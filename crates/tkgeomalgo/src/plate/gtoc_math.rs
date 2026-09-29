// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of the computations that ModelingAlgorithms/TKGeomAlgo/Plate/
// Plate_GtoCConstraint.cxx and Plate_FreeGtoCConstraint.cxx repeat in every constructor, at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The G1, G2 and G3 matching terms of the GtoC constraints. The native constructors
//! repeat these blocks verbatim; the port writes each block once, with the native
//! operation order.
use super::{PlateD1, PlateD2, PlateD3};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3, Xyz};
use rustycad_tkmath::math::{Gauss, Matrix, Vector};

/// `NORMIN`: the smallest normal length (alr, 12/11/96).
pub(super) const NORMIN: f64 = 1.0e-10;
/// `COSMIN`: the smallest cosine between the surface normals.
pub(super) const COSMIN: f64 = 1.0e-2;

/// The unit normal `Du ^ Dv`, or `None` below `NORMIN`.
pub(super) fn unit_normal(d1: &PlateD1) -> Option<Xyz> {
    let mut normal: Xyz = d1.du ^ d1.dv;
    if normal.modulus() < NORMIN {
        return None;
    }
    normal
        .normalize()
        .unwrap_or_else(|_| unreachable!("the normal is longer than NORMIN"));
    Some(normal)
}

/// The G1 corrections `(du, dv)`: `dir * -(normale . D1S.D) * invcos`.
pub(super) fn g1_offsets(dir: Xyz, normale: Xyz, d1s: &PlateD1, invcos: f64) -> (Xyz, Xyz) {
    let du: Xyz = dir * -(normale * d1s.du) * invcos;
    let dv: Xyz = dir * -(normale * d1s.dv) * invcos;
    (du, dv)
}

/// The reparametrization of the target derivatives onto the corrected surface: the
/// 2x2 Gauss decomposition and its solutions `(a, b)`, `(c, d)`.
pub(super) struct G2Frame {
    pub(super) su: Xyz,
    pub(super) sv: Xyz,
    gauss: Gauss,
    pub(super) a: f64,
    pub(super) b: f64,
    pub(super) c: f64,
    pub(super) d: f64,
}

fn solve2(gauss: &Gauss, v0: f64, v1: f64) -> Result<(f64, f64), Failure> {
    let mut vec: Vector<'static> = Vector::with_bounds(0, 1)?;
    vec[0] = v0;
    vec[1] = v1;
    let mut sol: Vector<'static> = Vector::with_bounds(0, 1)?;
    gauss.solve_into(&vec, &mut sol)?;
    Ok((sol[0], sol[1]))
}

impl G2Frame {
    /// The frame of `Su = D1S.Du + du`, `Sv = D1S.Dv + dv` against the target first
    /// derivatives; `None` when the 2x2 system is singular.
    pub(super) fn new(
        d1s: &PlateD1,
        (du, dv): (Xyz, Xyz),
        d1t: &PlateD1,
    ) -> Result<Option<Self>, Failure> {
        let su: Xyz = d1s.du + du;
        let sv: Xyz = d1s.dv + dv;
        let mut mat: Matrix<'static> =
            Matrix::filled(0, 1, 0, 1, 0.0).map_err(errors::array_bounds)?;
        *mat.change_value(0, 0)? = su * d1t.du;
        *mat.change_value(0, 1)? = su * d1t.dv;
        *mat.change_value(1, 0)? = sv * d1t.du;
        *mat.change_value(1, 1)? = sv * d1t.dv;
        let gauss = Gauss::new(&mat)?;
        if !gauss.is_done() {
            return Ok(None);
        }
        let (a, b) = solve2(&gauss, su * su, su * sv)?;
        let (c, d) = solve2(&gauss, sv * su, sv * sv)?;
        Ok(Some(Self {
            su,
            sv,
            gauss,
            a,
            b,
            c,
            d,
        }))
    }

    /// The target second derivatives in the corrected parameters `(Suu, Suv, Svv)`.
    pub(super) fn second(&self, d2t: &PlateD2) -> (Xyz, Xyz, Xyz) {
        let (a, b, c, d) = (self.a, self.b, self.c, self.d);
        let suu: Xyz = d2t.duu * (a * a) + d2t.duv * (2.0 * a * b) + d2t.dvv * (b * b);
        let suv: Xyz = d2t.duu * (a * c) + d2t.duv * (a * d + b * c) + d2t.dvv * (b * d);
        let svv: Xyz = d2t.duu * (c * c) + d2t.duv * (2.0 * c * d) + d2t.dvv * (d * d);
        (suu, suv, svv)
    }

    /// The solution of the frame system for the residual `(D2S + d - S) . (Su, Sv)`.
    fn residual(&self, residual: Xyz) -> Result<(f64, f64), Failure> {
        solve2(&self.gauss, residual * self.su, residual * self.sv)
    }

    /// The target third derivatives in the corrected parameters
    /// `(Suuu, Suuv, Suvv, Svvv)`, from the corrected second derivatives `d2` (the
    /// G2 corrections added) and the target `(Suu, Suv, Svv)`.
    pub(super) fn third(
        &self,
        d2s: &PlateD2,
        corrected: (Xyz, Xyz, Xyz),
        target: (Xyz, Xyz, Xyz),
        (d2t, d3t): (&PlateD2, &PlateD3),
    ) -> Result<[Xyz; 4], Failure> {
        let (duu, duv, dvv) = corrected;
        let (suu, suv, svv) = target;
        let (b0uu, b1uu) = self.residual(d2s.duu + duu - suu)?;
        let (b0uv, b1uv) = self.residual(d2s.duv + duv - suv)?;
        let (b0vv, b1vv) = self.residual(d2s.dvv + dvv - svv)?;
        let mut s: [Xyz; 4] = self.cubic_terms(d3t);
        let (a0u, a1u, a0v, a1v) = (self.a, self.b, self.c, self.d);
        s[0] += d2t.duu * (3.0 * a0u * b0uu)
            + d2t.duv * (3.0 * (a0u * b1uu + a1u * b0uu))
            + d2t.dvv * (3.0 * a1u * b1uu);
        s[1] += d2t.duu * (2.0 * a0u * b0uv + a0v * b0uu)
            + d2t.duv * (2.0 * (a0u * b1uv + a1u * b0uv) + a0v * b1uu + a1v * b0uu)
            + d2t.dvv * (2.0 * a1u * b1uv + a1v * b1uu);
        s[2] += d2t.duu * (a0u * b0vv + 2.0 * a0v * b0uv)
            + d2t.duv * (2.0 * (a0v * b1uv + a1v * b0uv) + a0u * b1vv + a1u * b0vv)
            + d2t.dvv * (2.0 * a1v * b1uv + a1u * b1vv);
        s[3] += d2t.duu * (3.0 * a0v * b0vv)
            + d2t.duv * (3.0 * (a0v * b1vv + a1v * b0vv))
            + d2t.dvv * (3.0 * a1v * b1vv);
        Ok(s)
    }

    /// The cubic part of the target third derivatives, before the second-order terms.
    fn cubic_terms(&self, d3t: &PlateD3) -> [Xyz; 4] {
        let (a, b, c, d) = (self.a, self.b, self.c, self.d);
        [
            d3t.duuu * (a * a * a)
                + d3t.duuv * (3.0 * a * a * b)
                + d3t.duvv * (3.0 * a * b * b)
                + d3t.dvvv * (b * b * b),
            d3t.duuu * (a * a * c)
                + d3t.duuv * (a * a * d + 2.0 * a * b * c)
                + d3t.duvv * (b * b * c + 2.0 * a * b * d)
                + d3t.dvvv * (b * b * d),
            d3t.duuu * (a * c * c)
                + d3t.duuv * (b * c * c + 2.0 * a * c * d)
                + d3t.duvv * (a * d * d + 2.0 * b * c * d)
                + d3t.dvvv * (b * d * d),
            d3t.duuu * (c * c * c)
                + d3t.duuv * (3.0 * c * c * d)
                + d3t.duvv * (3.0 * c * d * d)
                + d3t.dvvv * (d * d * d),
        ]
    }
}

/// `dir * (normale . (S - D)) * invcos`, the G2 and G3 corrections along `dir`.
pub(super) fn projected(dir: Xyz, normale: Xyz, difference: Xyz, invcos: f64) -> Xyz {
    dir * (normale * difference) * invcos
}

/// The rotation of the incremental load: from the source normal `n0` towards the
/// target normal `normale` (reversed when `orientation` asks it or the cosine is
/// negative), by `angle * (IncrementalLoad - 1)`; `None` when the normals are almost
/// opposite. Standard_ConstructionError when they are parallel (null rotation axis).
pub(super) fn incremental_rotation(
    n0: Xyz,
    normale: Xyz,
    incremental_load: f64,
    orientation: i32,
) -> Result<Option<Transform3>, Failure> {
    let mut n1: Xyz = normale;
    if orientation != 0 {
        n1 *= f64::from(orientation);
    }
    let mut c: f64 = n0 * n1;
    if orientation == 0 && c < 0.0 {
        c *= -1.0;
        n1 *= -1.0;
    }
    let s: f64 = n0.cross_magnitude(n1);
    if s < 1.0e-2 && c < 0.0 {
        return Ok(None);
    }
    let angle: f64 = c.atan2(s);
    let mut d: Xyz = n0 ^ n1;
    d.normalize()?;
    let axe = Axis1::new(Point3::new(0.0, 0.0, 0.0), Direction3::from_xyz(d)?);
    let mut rota = Transform3::default();
    rota.set_rotation(axe, angle * (incremental_load - 1.0))?;
    Ok(Some(rota))
}
