// Created on: 1997-05-13
// Created by: Stagiaire Francois DUMONT
// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_DenominatorMultiplier.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib_DenominatorMultiplier`: the law a(u, v) that cancels the U derivative
//! of the denominator of a rational B-spline surface on its U boundaries.
use super::live;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::bsplslib::{self, SurfaceSpan};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::Matrix;

/// The denominator multiplier of a rational B-spline surface: the cubic (in U)
/// B-spline law on the flat knots `KnotVector` (8, 9 or 10 knots on [0, 1]) that
/// meets the boundary denominators and cancels their U derivatives.
///
/// `let law = DenominatorMultiplier::new(&surface, &flat_knots)?; let a = law.value(0.5, 0.5)?;`
#[derive(Clone)]
pub struct DenominatorMultiplier {
    my_surface: Handle<GeomBSplineSurface>,
    my_knot_flat_vector: Array1<'static, f64>,
}

impl DenominatorMultiplier {
    /// The law of `surface` on the flat knots `knot_vector`, read from index 1 to its
    /// length as natively; Standard_OutOfRange for an index outside `knot_vector`
    /// (`GeomLib_DenominatorMultiplier(Surface, KnotVector)`);
    /// `DenominatorMultiplier::new(&surface, &flat_knots)?`.
    pub fn new(
        surface: &Handle<GeomBSplineSurface>,
        knot_vector: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        let mut my_knot_flat_vector: Array1<'static, f64> = one_based(knot_vector.length())?;
        for i in 1..=knot_vector.length() {
            my_knot_flat_vector[i] = *knot_vector
                .value(i)
                .map_err(|e| crate::native_failure::array_failure(e, "KnotVector"))?;
        }
        Ok(Self {
            my_surface: surface.clone(),
            my_knot_flat_vector,
        })
    }

    /// `a(u, v)`: the law poles built from the denominators D and their U derivatives
    /// on both U boundaries at `v` (the last ones scaled by the square of the
    /// first-to-last weight ratio of the first row), times the cubic basis at `u`.
    /// The combination depends on the flat knot count (8, 9 or 10); any other count
    /// gives 0 (`Value(UParameter, VParameter)`); `law.value(0.5, 0.5)?`.
    pub fn value(&self, u_parameter: f64, v_parameter: f64) -> Result<f64, Failure> {
        let surface: &GeomBSplineSurface = live(&self.my_surface, "GeomLib_DenominatorMultiplier")?;
        let (dumaxv, d_dduumaxv) = boundary_denominator(
            surface,
            surface.u_knot(surface.last_u_knot_index()?)?,
            v_parameter,
        )?;
        let (duminv, d_dduuminv) = boundary_denominator(surface, surface.u_knot(1)?, v_parameter)?;
        let lambda: f64 = surface.weight(1, 1)? / surface.weight(surface.nb_u_poles(), 1)?;
        let b1prim0: f64 = self.basis_derivative(0.0, 2)?;
        let bprelastprim1: f64 = self.basis_derivative(1.0, 3)?;
        let mut basis_value: Matrix<'static> = crate::array_access::matrix(1, 1, 1, 4, 0.0)?;
        let mut index: i32 = 0;
        bsplclib::eval_bspline_basis(
            0,
            4,
            &self.my_knot_flat_vector,
            u_parameter,
            &mut index,
            &mut basis_value,
            false,
        )?;
        let b = |col: i32| -> Result<f64, Failure> { Ok(*basis_value.value(1, col)?) };
        // Poles of a(u, v).
        let mut polesenv: [f64; 6] = [0.0; 6];
        polesenv[0] = duminv;
        polesenv[1] = duminv - d_dduuminv / b1prim0;
        polesenv[4] = lambda * lambda * (dumaxv - d_dduumaxv / bprelastprim1);
        polesenv[5] = lambda * lambda * dumaxv;
        let mut value: [f64; 6] = [0.0; 6];
        match self.my_knot_flat_vector.length() {
            8 => value = [b(1)?, b(2)?, 0.0, 0.0, b(3)?, b(4)?],
            9 => {
                value = if index == 1 {
                    [b(1)?, b(2)?, b(3)?, 0.0, b(4)?, 0.0]
                } else {
                    [0.0, b(1)?, b(2)?, 0.0, b(3)?, b(4)?]
                };
                polesenv[2] = 0.5 * (polesenv[0] + polesenv[5]);
            }
            10 => {
                value = ten_knot_values(index, &b, value)?;
                polesenv[2] = 0.5 * (polesenv[0] + polesenv[5]);
                polesenv[3] = polesenv[2];
            }
            _ => {}
        }
        let mut buv: f64 = 0.0;
        for i in 0..6 {
            buv += polesenv[i] * value[i];
        }
        Ok(buv)
    }

    // The first derivative (row 2) of basis function `column` at `parameter`.
    fn basis_derivative(&self, parameter: f64, column: i32) -> Result<f64, Failure> {
        let mut deriv: Matrix<'static> = crate::array_access::matrix(1, 2, 1, 4, 0.0)?;
        let mut index: i32 = 0;
        bsplclib::eval_bspline_basis(
            1,
            4,
            &self.my_knot_flat_vector,
            parameter,
            &mut index,
            &mut deriv,
            false,
        )?;
        Ok(*deriv.value(2, column)?)
    }
}

// The 10-knot case: the basis values shifted by the span index 1, 2 or 3; another
// index keeps the values as they are (zeros), as natively.
fn ten_knot_values(
    index: i32,
    b: &dyn Fn(i32) -> Result<f64, Failure>,
    value: [f64; 6],
) -> Result<[f64; 6], Failure> {
    Ok(match index {
        1 => [b(1)?, b(2)?, b(3)?, b(4)?, 0.0, 0.0],
        2 => [0.0, b(1)?, b(2)?, b(3)?, b(4)?, 0.0],
        3 => [0.0, 0.0, b(1)?, b(2)?, b(3)?, b(4)?],
        _ => value,
    })
}

// BSplSLib::HomogeneousD1 at (u, v) with no derivative request: the denominator and
// its U derivative.
fn boundary_denominator(
    surface: &GeomBSplineSurface,
    u: f64,
    v: f64,
) -> Result<(f64, f64), Failure> {
    let (u_knots, v_knots) = (surface.u_knots(), surface.v_knots());
    let (u_mults, v_mults) = (surface.u_multiplicities(), surface.v_multiplicities());
    let poles: Array2<'static, Point3> = surface.poles();
    let weights: Option<Array2<'static, f64>> = surface.weights();
    let span = SurfaceSpan {
        u,
        v,
        u_index: 0,
        v_index: 0,
        u_knots: &u_knots,
        v_knots: &v_knots,
        u_mults: Some(&u_mults),
        v_mults: Some(&v_mults),
        u_degree: surface.u_degree(),
        v_degree: surface.v_degree(),
        u_rat: surface.is_u_rational(),
        v_rat: surface.is_v_rational(),
        u_per: surface.is_u_periodic(),
        v_per: surface.is_v_periodic(),
    };
    let (_, _, _, d, du, _) = bsplslib::homogeneous_d1(&span, &poles, weights.as_ref())?;
    Ok((d, du))
}
