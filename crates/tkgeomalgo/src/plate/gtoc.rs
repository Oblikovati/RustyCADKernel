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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Plate/Plate_GtoCConstraint.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Plate_GtoCConstraint`: pinpoint constraints that make the plate match a target
//! surface to G1, G2 or G3 at a point.
use super::gtoc_math::{COSMIN, G2Frame, NORMIN, g1_offsets, projected, unit_normal};
use super::{PlateD1, PlateD2, PlateD3, PlatePinpointConstraint};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::{Xy, Xyz};

/// G1 (then G2, G3) matching of the surface `S` onto the target `T` at a point, as up
/// to nine derivative constraints (`Plate_GtoCConstraint`). A constructor that meets a
/// degenerate normal or nearly orthogonal normals stops with the constraints built so
/// far, as natively.
///
/// `let c = PlateGtoCConstraint::g1(uv, &d1s, &d1t)?; c.nb_ppc()`.
#[derive(Clone, Debug)]
pub struct PlateGtoCConstraint {
    my_ppc: [PlatePinpointConstraint; 9],
    my_d1_surf_init: PlateD1,
    pnt2d: Xy,
    nb_pp_constraints: i32,
}

/// The G1 set-up shared by the constructors: the target normal, the direction of the
/// corrections (the surface normal, projected off `nP` when given) and `1 / cos`.
struct G1Basis {
    normale: Xyz,
    dir: Xyz,
    invcos: f64,
}

fn g1_basis(d1s: &PlateD1, d1t: &PlateD1, n_p: Option<Xyz>) -> Option<G1Basis> {
    let normale: Xyz = unit_normal(d1t)?;
    let normale_s: Xyz = unit_normal(d1s)?;
    let dir: Xyz = match n_p {
        None => normale_s,
        Some(np) => {
            let mut nsp: Xyz = normale_s - np * (np * normale_s);
            if nsp.modulus() < NORMIN {
                return None;
            }
            nsp.normalize()
                .unwrap_or_else(|_| unreachable!("the projection is longer than NORMIN"));
            nsp
        }
    };
    let cos_normales: f64 = normale * dir;
    if cos_normales.abs() < COSMIN {
        return None;
    }
    Some(G1Basis {
        normale,
        dir,
        invcos: 1.0 / cos_normales,
    })
}

impl PlateGtoCConstraint {
    fn empty(point2d: Xy, d1s: &PlateD1) -> Self {
        Self {
            my_ppc: [PlatePinpointConstraint::default(); 9],
            my_d1_surf_init: *d1s,
            pnt2d: point2d,
            nb_pp_constraints: 0,
        }
    }

    fn push(&mut self, value: Xyz, iu: i32, iv: i32) {
        let slot: usize = self.nb_pp_constraints as usize;
        self.my_ppc[slot] = PlatePinpointConstraint::new(self.pnt2d, value, iu, iv);
        self.nb_pp_constraints += 1;
    }

    /// The G1 constraints; returns the basis and the corrections when they were built.
    fn build_g1(
        &mut self,
        d1s: &PlateD1,
        d1t: &PlateD1,
        n_p: Option<Xyz>,
    ) -> Option<(G1Basis, (Xyz, Xyz))> {
        let basis: G1Basis = g1_basis(d1s, d1t, n_p)?;
        let (du, dv) = g1_offsets(basis.dir, basis.normale, d1s, basis.invcos);
        self.push(du, 1, 0);
        self.push(dv, 0, 1);
        Some((basis, (du, dv)))
    }

    /// The G2 constraints after G1; returns the frame and the second-order terms.
    fn build_g2(
        &mut self,
        (d1s, d1t): (&PlateD1, &PlateD1),
        (d2s, d2t): (&PlateD2, &PlateD2),
        (basis, g1): (&G1Basis, (Xyz, Xyz)),
    ) -> Result<Option<SecondOrder>, Failure> {
        let Some(frame) = G2Frame::new(d1s, g1, d1t)? else {
            return Ok(None);
        };
        let target = frame.second(d2t);
        let corrected = (
            projected(basis.dir, basis.normale, target.0 - d2s.duu, basis.invcos),
            projected(basis.dir, basis.normale, target.1 - d2s.duv, basis.invcos),
            projected(basis.dir, basis.normale, target.2 - d2s.dvv, basis.invcos),
        );
        self.push(corrected.0, 2, 0);
        self.push(corrected.1, 1, 1);
        self.push(corrected.2, 0, 2);
        Ok(Some(SecondOrder {
            frame,
            target,
            corrected,
        }))
    }

    /// G1 matching (`Plate_GtoCConstraint(point2d, D1S, D1T)`).
    /// `PlateGtoCConstraint::g1(uv, &d1s, &d1t)`.
    pub fn g1(point2d: Xy, d1s: &PlateD1, d1t: &PlateD1) -> Self {
        let mut constraint = Self::empty(point2d, d1s);
        constraint.build_g1(d1s, d1t, None);
        constraint
    }

    /// G1 matching with corrections orthogonal to `n_p`
    /// (`Plate_GtoCConstraint(point2d, D1S, D1T, nP)`). `PlateGtoCConstraint::g1_projected(uv, &d1s, &d1t, np)`.
    pub fn g1_projected(point2d: Xy, d1s: &PlateD1, d1t: &PlateD1, n_p: Xyz) -> Self {
        let mut constraint = Self::empty(point2d, d1s);
        constraint.build_g1(d1s, d1t, Some(n_p));
        constraint
    }

    fn g2_with(
        point2d: Xy,
        d1: (&PlateD1, &PlateD1),
        d2: (&PlateD2, &PlateD2),
        n_p: Option<Xyz>,
    ) -> Result<Self, Failure> {
        let mut constraint = Self::empty(point2d, d1.0);
        if let Some(g1) = constraint.build_g1(d1.0, d1.1, n_p) {
            constraint.build_g2(d1, d2, (&g1.0, g1.1))?;
        }
        Ok(constraint)
    }

    /// G2 matching (`Plate_GtoCConstraint(point2d, D1S, D1T, D2S, D2T)`).
    /// `PlateGtoCConstraint::g2(uv, &d1s, &d1t, &d2s, &d2t)?`.
    pub fn g2(
        point2d: Xy,
        d1s: &PlateD1,
        d1t: &PlateD1,
        d2s: &PlateD2,
        d2t: &PlateD2,
    ) -> Result<Self, Failure> {
        Self::g2_with(point2d, (d1s, d1t), (d2s, d2t), None)
    }

    /// G2 matching with corrections orthogonal to `n_p`
    /// (`Plate_GtoCConstraint(point2d, D1S, D1T, D2S, D2T, nP)`).
    /// `PlateGtoCConstraint::g2_projected(uv, (&d1s, &d1t), (&d2s, &d2t), np)?`.
    pub fn g2_projected(
        point2d: Xy,
        (d1s, d1t): (&PlateD1, &PlateD1),
        (d2s, d2t): (&PlateD2, &PlateD2),
        n_p: Xyz,
    ) -> Result<Self, Failure> {
        Self::g2_with(point2d, (d1s, d1t), (d2s, d2t), Some(n_p))
    }

    /// G3 matching (`Plate_GtoCConstraint(point2d, D1S, D1T, D2S, D2T, D3S, D3T)`).
    /// `PlateGtoCConstraint::g3(uv, (&d1s, &d1t), (&d2s, &d2t), (&d3s, &d3t))?`.
    pub fn g3(
        point2d: Xy,
        d1: (&PlateD1, &PlateD1),
        d2: (&PlateD2, &PlateD2),
        (d3s, d3t): (&PlateD3, &PlateD3),
    ) -> Result<Self, Failure> {
        let mut constraint = Self::empty(point2d, d1.0);
        let Some(g1) = constraint.build_g1(d1.0, d1.1, None) else {
            return Ok(constraint);
        };
        let Some(second) = constraint.build_g2(d1, d2, (&g1.0, g1.1))? else {
            return Ok(constraint);
        };
        let s = second
            .frame
            .third(d2.0, second.corrected, second.target, (d2.1, d3t))?;
        let differences = [
            s[0] - d3s.duuu,
            s[1] - d3s.duuv,
            s[2] - d3s.duvv,
            s[3] - d3s.dvvv,
        ];
        for (difference, (iu, iv)) in differences
            .into_iter()
            .zip([(3, 0), (2, 1), (1, 2), (0, 3)])
        {
            constraint.push(
                projected(g1.0.dir, g1.0.normale, difference, g1.0.invcos),
                iu,
                iv,
            );
        }
        Ok(constraint)
    }

    /// G3 matching; the native constructor ignores `nP`, so this is `g3`
    /// (`Plate_GtoCConstraint(point2d, D1S, D1T, D2S, D2T, D3S, D3T, nP)`).
    /// `PlateGtoCConstraint::g3_projected(uv, d1, d2, d3, np)?`.
    pub fn g3_projected(
        point2d: Xy,
        d1: (&PlateD1, &PlateD1),
        d2: (&PlateD2, &PlateD2),
        d3: (&PlateD3, &PlateD3),
        _n_p: Xyz,
    ) -> Result<Self, Failure> {
        Self::g3(point2d, d1, d2, d3)
    }

    /// The number of built constraints: 0, 2, 5 or 9 (`nb_PPC`); `c.nb_ppc()`.
    pub const fn nb_ppc(&self) -> i32 {
        self.nb_pp_constraints
    }

    /// Constraint `index` of the fixed array of nine (`GetPPC`; entries past `nb_PPC`
    /// are default constraints); Standard_OutOfRange outside `0..9`, where the native
    /// array read is undefined. `c.get_ppc(0)?`.
    pub fn get_ppc(&self, index: i32) -> Result<PlatePinpointConstraint, Failure> {
        let slot = usize::try_from(index)
            .ok()
            .filter(|&i| i < self.my_ppc.len());
        slot.map(|i| self.my_ppc[i]).ok_or_else(|| {
            errors::out_of_range(&format!(
                "Plate_GtoCConstraint::GetPPC: Index = {index}; expected in [0, 8]"
            ))
        })
    }

    /// The initial surface derivatives (`D1SurfInit`); `c.d1_surf_init()`.
    pub const fn d1_surf_init(&self) -> PlateD1 {
        self.my_d1_surf_init
    }
}

/// The G2 results that G3 builds on.
struct SecondOrder {
    frame: G2Frame,
    target: (Xyz, Xyz, Xyz),
    corrected: (Xyz, Xyz, Xyz),
}
