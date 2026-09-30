// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Coons.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Coons`: the Coons filling of four boundary pole rows.
use super::GeomFillFilling;
use super::filling::{
    Rows, at, blend, blend_terms, coords, corner_terms, corners, domain_error, fill_four_sided,
    lengths, point_from, put2,
};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::plib;
use std::ops::{Deref, DerefMut};

/// The Coons filling (`GeomFill_Coons`): the inner poles blend the four boundary rows
/// with the cubic Hermite functions `F(t) = 1 - 3t^2 + 2t^3` and `G(t) = 3t^2 - 2t^3`
/// raised to the degree of each direction. The `GeomFill_Filling` result is reached
/// through `Deref`.
///
/// `let coons = GeomFillCoons::with_poles([&p1, &p2, &p3, &p4])?; coons.nb_v_poles()?`
#[derive(Clone, Default)]
pub struct GeomFillCoons {
    filling: GeomFillFilling,
}

impl Deref for GeomFillCoons {
    type Target = GeomFillFilling;
    fn deref(&self) -> &GeomFillFilling {
        &self.filling
    }
}

impl DerefMut for GeomFillCoons {
    fn deref_mut(&mut self) -> &mut GeomFillFilling {
        &mut self.filling
    }
}

impl GeomFillCoons {
    /// An empty filling (`GeomFill_Coons()`); `GeomFillCoons::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomFill_Coons(P1, P2, P3, P4)`: see `init`;
    /// `GeomFillCoons::with_poles([&p1, &p2, &p3, &p4])?`.
    pub fn with_poles(poles: Rows<'_, '_, Point3>) -> Result<Self, Failure> {
        let mut coons = Self::new();
        coons.init(poles)?;
        Ok(coons)
    }
    /// `GeomFill_Coons(P1, P2, P3, P4, W1, W2, W3, W4)`: see `init_weighted`;
    /// `GeomFillCoons::with_weights([&p1, &p2, &p3, &p4], [&w1, &w2, &w3, &w4])?`.
    pub fn with_weights(
        poles: Rows<'_, '_, Point3>,
        weights: Rows<'_, '_, f64>,
    ) -> Result<Self, Failure> {
        let mut coons = Self::new();
        coons.init_weighted(poles, weights)?;
        Ok(coons)
    }
    /// `Init(P1, P2, P3, P4)`: `P1` and `P3` are the first and last columns
    /// (`NbUPoles` values), `P2` and `P4` the first and last rows; opposite rows of
    /// different lengths raise Standard_DomainError. The rows are read with their own
    /// indices `1..=n`, as natively; `coons.init([&p1, &p2, &p3, &p4])?`.
    pub fn init(&mut self, poles: Rows<'_, '_, Point3>) -> Result<(), Failure> {
        let [p1, p2, p3, p4] = poles;
        if p1.len() != p3.len() || p2.len() != p4.len() {
            return Err(domain_error("GeomFill_Coons::Init", &lengths(poles)));
        }
        let (npol_u, npol_v) = (p1.len() as i32, p2.len() as i32);
        self.filling.is_rational = false;
        let grid: &mut Array2<'static, Point3> = self.filling.new_poles(npol_u, npol_v)?;
        for i in 1..=npol_u {
            put2(grid, i, 1, at(p1, i)?)?;
            put2(grid, i, npol_v, at(p3, i)?)?;
        }
        for i in 1..=npol_v {
            put2(grid, 1, i, at(p2, i)?)?;
            put2(grid, npol_u, i, at(p4, i)?)?;
        }
        let pole: Array1<'static, Point3> = hermite_poles()?;
        let (fu, gu) = blending_coefficients(&pole, npol_u)?;
        let (fv, gv) = blending_coefficients(&pole, npol_v)?;
        fill_inner(grid, (&fu, &gu), (&fv, &gv))
    }
    /// `Init(P1, P2, P3, P4, W1, W2, W3, W4)`: the poles of `init`, and the weights
    /// blended bilinearly with the corner correction; the weight rows must match the
    /// pole rows (Standard_DomainError); `coons.init_weighted([&p1, &p2, &p3, &p4], [&w1, &w2, &w3, &w4])?`.
    pub fn init_weighted(
        &mut self,
        poles: Rows<'_, '_, Point3>,
        weights: Rows<'_, '_, f64>,
    ) -> Result<(), Failure> {
        check_weights("GeomFill_Coons::Init", poles, weights)?;
        self.init(poles)?;
        self.filling.is_rational = true;
        let (npol_u, npol_v) = (weights[0].len() as i32, weights[1].len() as i32);
        let grid: &mut Array2<'static, f64> = self.filling.new_weights(npol_u, npol_v)?;
        fill_four_sided(grid, weights, false, |r, i, j| {
            Ok(blend(r, blend_terms(weights, i, j)?) - corners(r, corner_terms(weights)?))
        })
    }
}

/// The two Standard_DomainError checks of the weighted constructors.
pub(super) fn check_weights(
    member: &str,
    poles: Rows<'_, '_, Point3>,
    weights: Rows<'_, '_, f64>,
) -> Result<(), Failure> {
    let [w1, w2, w3, w4] = weights.map(|w| w.len());
    let lengths = [w1, w2, w3, w4];
    if w1 != w3 || w2 != w4 {
        return Err(domain_error(member, &lengths));
    }
    if (0..4).any(|k| weights[k].len() != poles[k].len()) {
        return Err(domain_error(member, &lengths));
    }
    Ok(())
}

/// `PLib::CoefficientsPoles` of the Hermite coefficients `(1, 0, -3, 2)` in X and
/// `(0, 0, 3, -2)` in Y: the cubic Bezier poles of `F` and `G`.
fn hermite_poles() -> Result<Array1<'static, Point3>, Failure> {
    let mut coef: Array1<'static, Point3> = one_based(4)?;
    for (k, (x, y)) in (1..).zip([(1., 0.), (0., 0.), (-3., 3.), (2., -2.)]) {
        coef[k] = Point3::new(x, y, 0.);
    }
    let mut pole: Array1<'static, Point3> = one_based(4)?;
    plib::coefficients_poles(&coef, None, &mut pole, None)?;
    Ok(pole)
}

/// `FU(i)` and `GU(i)` for `i` in `2..NPol` (arrays of bounds `2..NPol - 1`): the
/// Hermite poles raised to degree `NPol - 1` when there are more than four poles,
/// the cubic ones otherwise (native behavior, which reads the cubic poles directly).
/// Fewer than two poles give a negative array size, a huge allocation natively and
/// Standard_RangeError here.
fn blending_coefficients(
    pole: &Array1<'static, Point3>,
    npol: i32,
) -> Result<(Array1<'static, f64>, Array1<'static, f64>), Failure> {
    let mut coef: Array1<'static, Point3> = one_based(npol)?;
    if npol > 4 {
        bsplclib::increase_degree_bezier(npol - 1, pole, None, &mut coef, None)?;
    } else {
        coef.assign(pole);
    }
    let mut f: Array1<'static, f64> =
        Array1::with_bounds(2, npol - 1).map_err(errors::array_bounds)?;
    let mut g: Array1<'static, f64> =
        Array1::with_bounds(2, npol - 1).map_err(errors::array_bounds)?;
    for i in 2..npol {
        let c: Point3 = at(&coef, i)?;
        (f[i], g[i]) = (c.x(), c.y());
    }
    Ok((f, g))
}

/// The inner poles of the Coons patch, `FV(j) P(i,1) + GV(j) P(i,NV) + FU(i) P(1,j)
/// + GU(i) P(NU,j)` minus the four corner terms, per coordinate in native order.
fn fill_inner(
    grid: &mut Array2<'static, Point3>,
    (fu, gu): (&Array1<'static, f64>, &Array1<'static, f64>),
    (fv, gv): (&Array1<'static, f64>, &Array1<'static, f64>),
) -> Result<(), Failure> {
    let (npol_u, npol_v) = (grid.col_length(), grid.row_length());
    let value = |grid: &Array2<'static, Point3>, i: i32, j: i32| coords(grid[(i, j)]);
    let corner = [
        value(grid, 1, 1),
        value(grid, 1, npol_v),
        value(grid, npol_u, 1),
        value(grid, npol_u, npol_v),
    ];
    for j in 2..npol_v {
        let (fvj, gvj) = (fv[j], gv[j]);
        for i in 2..npol_u {
            let (fui, gui) = (fu[i], gu[i]);
            let edges = [
                value(grid, i, 1),
                value(grid, i, npol_v),
                value(grid, 1, j),
                value(grid, npol_u, j),
            ];
            let p: Point3 = point_from(|k| {
                fvj * edges[0][k] + gvj * edges[1][k] + fui * edges[2][k] + gui * edges[3][k]
                    - fui * fvj * corner[0][k]
                    - fui * gvj * corner[1][k]
                    - gui * fvj * corner[2][k]
                    - gui * gvj * corner[3][k]
            });
            put2(grid, i, j, p)?;
        }
    }
    Ok(())
}
