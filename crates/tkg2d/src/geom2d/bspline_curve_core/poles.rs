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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve.cxx, Geom2d_BSplineCurve_1.cxx and ModelingData/TKG3d/Geom/Geom_BSplineCurve.cxx, Geom_BSplineCurve_1.cxx (poles, weights, moves) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Pole and weight edition, point moves, reversal and transformation.
use super::{BSplineCurveCore, BSplineCurveKind};
use crate::geom2d::arrays::{check_one_based, one_based, put, rebased};
use crate::geom2d::errors;
use crate::geom2d::spline_curve_kind::KindVector;
use crate::geom2d::spline_data::bspline_rational;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::{self, CurvePoint};
use rustycad_tkmath::gp::resolution;

/// `gp_Vec(first, second)` (or `gp_Vec2d`): the coordinate differences
/// `second - first`, one subtraction per coordinate as gp_XY and gp_XYZ compute them;
/// `between::<Point2>(a, b)`.
fn between<P: CurvePoint>(first: P, second: P) -> P::Vector {
    let mut coordinates: [f64; 3] = [0.0; 3];
    for (k, value) in coordinates.iter_mut().enumerate().take(P::DIMENSION) {
        *value = second.pole_coordinate(k) - first.pole_coordinate(k);
    }
    P::vector_from(&coordinates[..P::DIMENSION])
}

/// The componentwise difference `a - b` of two derivative vectors, as MovePointAndTangent
/// builds its `DeltaDerivative`; `vector_difference::<Point3>(&t, &d1)`.
fn vector_difference<P: CurvePoint>(a: &P::Vector, b: &P::Vector) -> P::Vector {
    let mut coordinates: [f64; 3] = [0.0; 3];
    for (k, value) in coordinates.iter_mut().enumerate().take(P::DIMENSION) {
        *value = P::vector_coordinate(a, k) - P::vector_coordinate(b, k);
    }
    P::vector_from(&coordinates[..P::DIMENSION])
}

impl<K: BSplineCurveKind> BSplineCurveCore<K> {
    /// `Reverse`: reversed knots, multiplicities, poles and weights; `core.reverse()?`.
    pub fn reverse(&mut self) -> Result<(), Failure> {
        self.clear_eval_rep_on_edit();
        bsplclib::reverse_knots(&mut self.knots)?;
        bsplclib::reverse_mults(&mut self.mults);
        let last: i32 = if self.periodic {
            self.flat_knots.upper() - self.deg - 1
        } else {
            self.poles.upper()
        };
        bsplclib::reverse_poles(&mut self.poles, last)?;
        if self.rational {
            bsplclib::reverse_poles(&mut self.weights, last)?;
        }
        self.update_knots()
    }
    /// `SetPole(Index, P)`; `core.set_pole(2, p)?`.
    pub fn set_pole(&mut self, index: i32, pole: K::Point) -> Result<(), Failure> {
        check_one_based(
            index,
            self.poles.len(),
            "BSpline curve: SetPole: index and #pole mismatch",
        )?;
        self.clear_eval_rep_on_edit();
        put(&mut self.poles, index, pole)?;
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `SetWeight`: equal weights make the curve non-rational again;
    /// `core.set_weight(2, 2.0)?`.
    pub fn set_weight(&mut self, index: i32, weight: f64) -> Result<(), Failure> {
        check_one_based(
            index,
            self.poles.len(),
            "BSpline curve: SetWeight: Index and #pole mismatch",
        )?;
        if weight <= resolution() {
            return Err(errors::construction(&format!(
                "BSpline curve: SetWeight: Weight = {weight} too small; expected > gp::Resolution()"
            )));
        }
        self.clear_eval_rep_on_edit();
        let mut rational_now: bool = self.rational || (weight - 1.).abs() > resolution();
        if rational_now {
            if !self.rational {
                // Detach the shared unit weights before writing, like the native copy.
                self.weights = rebased(&self.weights)?;
            }
            put(&mut self.weights, index, weight)?;
            if self.rational {
                rational_now = bspline_rational(&self.weights)?;
                if !rational_now {
                    self.reset_unit_weights()?;
                }
            }
            self.rational = rational_now;
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `MovePoint`: returns `(FirstModifiedPole, LastModifiedPole)`, both 0 when nothing
    /// moved; `core.move_point(0.5, p, 2, 3)?`.
    pub fn move_point(
        &mut self,
        u: f64,
        point: K::Point,
        index1: i32,
        index2: i32,
    ) -> Result<(i32, i32), Failure> {
        let nbpoles: i32 = self.poles.len() as i32;
        if index1 < 1 || index1 > nbpoles || index2 < 1 || index2 > nbpoles || index1 > index2 {
            return Err(errors::out_of_range(&format!(
                "BSpline curve: MovePoint: Index1 = {index1}, Index2 = {index2}; \
                 expected 1 <= Index1 <= Index2 <= {nbpoles}"
            )));
        }
        let mut new_poles: Array1<'static, K::Point> = one_based(nbpoles)?;
        let current: K::Point = self.eval_d0(u)?;
        let displacement: KindVector<K> = between(current, point);
        let moved: (i32, i32) = bsplclib::move_point(
            u,
            displacement,
            index1,
            index2,
            self.deg,
            &self.poles,
            self.weights_view(),
            &self.flat_knots,
            &mut new_poles,
        )?;
        if moved.0 != 0 {
            self.clear_eval_rep_on_edit();
            self.poles = new_poles;
            self.max_deriv_inv_ok = false;
        }
        Ok(moved)
    }
    /// `MovePointAndTangent`: returns `ErrorStatus`, 0 on success; a periodic curve is
    /// made non-periodic first, whatever the outcome;
    /// `core.move_point_and_tangent(0.5, p, t, 1e-7, (1, 1))?`.
    pub fn move_point_and_tangent(
        &mut self,
        u: f64,
        point: K::Point,
        tangent: KindVector<K>,
        tolerance: f64,
        conditions: (i32, i32),
    ) -> Result<i32, Failure> {
        if self.periodic {
            self.set_not_periodic()?;
        }
        let mut new_poles: Array1<'static, K::Point> = one_based(self.poles.len() as i32)?;
        let (current_point, current_d1) = self.eval_d1(u)?;
        let delta: KindVector<K> = between(current_point, point);
        let delta_derivative: KindVector<K> = vector_difference::<K::Point>(&tangent, &current_d1);
        let mut error_status: i32 = 0;
        bsplclib::move_point_and_tangent(
            u,
            delta,
            delta_derivative,
            tolerance,
            self.deg,
            conditions.0,
            conditions.1,
            &self.poles,
            self.weights_view(),
            &self.flat_knots,
            &mut new_poles,
            &mut error_status,
        )?;
        if error_status == 0 {
            self.clear_eval_rep_on_edit();
            self.poles = new_poles;
            self.max_deriv_inv_ok = false;
        }
        Ok(error_status)
    }
    /// `Transform`: `apply` transforms each pole in place (`gp_Pnt2d::Transform` or
    /// `gp_Pnt::Transform`); `core.transform_poles(|pole| pole.transform(t))`.
    pub fn transform_poles(&mut self, apply: impl Fn(&mut K::Point)) {
        self.clear_eval_rep_on_edit();
        for pole in self.poles.as_mut_slice() {
            apply(pole);
        }
        self.max_deriv_inv_ok = false;
    }
}
