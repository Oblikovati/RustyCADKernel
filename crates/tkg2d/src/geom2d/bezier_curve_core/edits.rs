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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BezierCurve.cxx and ModelingData/TKG3d/Geom/Geom_BezierCurve.cxx (degree elevation, pole edits, Segment, Transform, Resolution) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edits of a Bezier curve; each one ends in the native `init` or cache reset.
use super::{BezierCurveCore, BezierCurveKind};
use crate::geom2d::arrays::{
    at, check_one_based, insert_at, one_based, put, rebased, remove_at, stored_or_unit_weights,
};
use crate::geom2d::errors;
use crate::geom2d::spline_data::{bezier_knots, bezier_rational};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::resolution;
use rustycad_tkmath::plib;

impl<K: BezierCurveKind> BezierCurveCore<K> {
    /// `Increase(Degree)`: degree elevation, then `init`; `core.increase(5)?`.
    pub fn increase(&mut self, degree: i32) -> Result<(), Failure> {
        if degree == self.degree() {
            return Ok(());
        }
        if degree < self.degree() || degree > bsplclib::max_degree() {
            return Err(errors::construction(&format!(
                "{}::Increase: Degree = {degree}; expected in [{}, {}]",
                K::BEZIER_CLASS,
                self.degree(),
                bsplclib::max_degree()
            )));
        }
        let mut new_poles: Array1<'static, K::Point> = one_based(degree + 1)?;
        let (knots, mults) = (bezier_knots(), self.multiplicities()?);
        let (mut new_knots, mut new_mults) = (one_based::<f64>(2)?, one_based::<i32>(2)?);
        let mut new_weights: Array1<'static, f64> = one_based(degree + 1)?;
        let target_weights = self.rational.then_some(&mut new_weights);
        let (degree_now, weights) = (self.degree(), self.weights_view());
        bsplclib::increase_degree(
            degree_now,
            degree,
            false,
            &self.poles,
            weights,
            &knots,
            &mults,
            &mut new_poles,
            target_weights,
            &mut new_knots,
            &mut new_mults,
        )?;
        let weights_arg: Option<&Array1<'_, f64>> = self.rational.then_some(&new_weights);
        self.init(&new_poles, weights_arg)
    }
    /// `InsertPoleAfter(Index, P, Weight)`, then `init`; `core.insert_pole_after(2, p, 1.0)?`.
    pub fn insert_pole_after(
        &mut self,
        index: i32,
        pole: K::Point,
        weight: f64,
    ) -> Result<(), Failure> {
        let nbpoles: i32 = self.poles.len() as i32;
        if nbpoles >= bsplclib::max_degree() || weight <= resolution() {
            return Err(errors::construction(&format!(
                "{}::InsertPoleAfter: NbPoles = {nbpoles}, Weight = {weight}; \
                 expected NbPoles < MaxDegree() and Weight > gp::Resolution()",
                K::BEZIER_CLASS
            )));
        }
        if index < 0 || index > nbpoles {
            return Err(errors::out_of_range(&format!(
                "{}::InsertPoleAfter: Index = {index}; expected in [0, {nbpoles}]",
                K::BEZIER_CLASS
            )));
        }
        let mut new_poles: Array1<'static, K::Point> = one_based(nbpoles + 1)?;
        insert_at(&self.poles, &mut new_poles, index, pole)?;
        if !(self.rational || (weight - 1.).abs() > resolution()) {
            return self.init(&new_poles, None);
        }
        let mut new_weights: Array1<'static, f64> = one_based(nbpoles + 1)?;
        let old_weights: Array1<'static, f64> =
            stored_or_unit_weights(self.rational, &self.weights, self.poles.len() as i32)?;
        insert_at(&old_weights, &mut new_weights, index, weight)?;
        self.init(&new_poles, Some(&new_weights))
    }
    /// `RemovePole(Index)`, then `init`; `core.remove_pole(2)?`.
    pub fn remove_pole(&mut self, index: i32) -> Result<(), Failure> {
        let nbpoles: i32 = self.poles.len() as i32;
        if nbpoles <= 2 {
            return Err(errors::construction(&format!(
                "{}::RemovePole: NbPoles = {nbpoles}; expected > 2",
                K::BEZIER_CLASS
            )));
        }
        let member: String = format!("{}::RemovePole", K::BEZIER_CLASS);
        check_one_based(index, self.poles.len(), &member)?;
        let mut new_poles: Array1<'static, K::Point> = one_based(nbpoles - 1)?;
        remove_at(&self.poles, &mut new_poles, index)?;
        if !self.rational {
            return self.init(&new_poles, None);
        }
        let mut new_weights: Array1<'static, f64> = one_based(nbpoles - 1)?;
        remove_at(&self.weights, &mut new_weights, index)?;
        self.init(&new_poles, Some(&new_weights))
    }
    /// `Reverse`: the poles and weights in reverse order; `core.reverse()`.
    pub fn reverse(&mut self) {
        self.poles.as_mut_slice().reverse();
        if self.rational {
            self.weights.as_mut_slice().reverse();
        }
        self.max_deriv_inv_ok = false;
    }
    /// `Segment(U1, U2)`: the poles of the arc through the power basis;
    /// `core.segment(0.2, 0.8)?`.
    pub fn segment(&mut self, u1: f64, u2: f64) -> Result<(), Failure> {
        let (p1, p2) = (self.eval_d0(u1)?, self.eval_d0(u2)?);
        self.closed = K::point_distance(p1, p2).abs() <= K::closure_tolerance();
        let count: i32 = self.poles.len() as i32;
        let flat: Array1<'static, f64> = self.knot_sequence()?;
        let mut coeffs: Array1<'static, K::Point> = one_based(count)?;
        let mut wcoeffs: Array1<'static, f64> = one_based(count)?;
        let rational: bool = self.rational;
        bsplclib::build_cache(
            0.0,
            1.0,
            false,
            self.degree(),
            &flat,
            &self.poles,
            self.weights_view(),
            &mut coeffs,
            rational.then_some(&mut wcoeffs),
        )?;
        plib::trimming(u1, u2, &mut coeffs, rational.then_some(&mut wcoeffs))?;
        let target_weights = rational.then_some(&mut self.weights);
        plib::coefficients_poles(
            &coeffs,
            rational.then_some(&wcoeffs),
            &mut self.poles,
            target_weights,
        )?;
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `SetPole(Index, P)`, updating the closure flag at an end pole;
    /// `core.set_pole(1, p)?`.
    pub fn set_pole(&mut self, index: i32, pole: K::Point) -> Result<(), Failure> {
        let member: String = format!("{}::SetPole", K::BEZIER_CLASS);
        check_one_based(index, self.poles.len(), &member)?;
        put(&mut self.poles, index, pole)?;
        if index == 1 || index == self.poles.len() as i32 {
            let (first, last) = (
                at(&self.poles, 1)?,
                at(&self.poles, self.poles.len() as i32)?,
            );
            self.closed = K::point_distance(first, last) <= K::closure_tolerance();
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `SetWeight(Index, Weight)`: equal weights make the curve non-rational again;
    /// `core.set_weight(2, 2.0)?`.
    pub fn set_weight(&mut self, index: i32, weight: f64) -> Result<(), Failure> {
        let member: String = format!("{}::SetWeight", K::BEZIER_CLASS);
        check_one_based(index, self.poles.len(), &member)?;
        if weight <= resolution() {
            return Err(errors::construction(&format!(
                "{}::SetWeight: Weight = {weight}; expected > gp::Resolution()",
                K::BEZIER_CLASS
            )));
        }
        let was_rational: bool = self.rational;
        if !was_rational {
            if (weight - 1.).abs() <= resolution() {
                return Ok(());
            }
            // Detach the shared unit weights before writing, like the native copy.
            self.weights = rebased(&self.weights)?;
        }
        put(&mut self.weights, index, weight)?;
        self.rational = !was_rational || bezier_rational(&self.weights)?;
        if !self.rational {
            self.weights = bsplclib::unit_weights(self.poles.len() as i32)?;
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    /// `Transform`: `apply` transforms each pole in place and the evaluation
    /// representation is cleared; `core.transform_poles(|pole| pole.transform(t))`.
    pub fn transform_poles(&mut self, apply: impl Fn(&mut K::Point)) {
        for pole in self.poles.as_mut_slice() {
            apply(pole);
        }
        self.eval_rep = K::EvalRep::default();
        self.max_deriv_inv_ok = false;
    }
    /// `Resolution`: the cached inverse derivative bound times `tolerance`;
    /// `core.resolution(1e-7)?`.
    pub fn resolution(&mut self, tolerance: f64) -> Result<f64, Failure> {
        if !self.max_deriv_inv_ok {
            let flat: Array1<'static, f64> = self.knot_sequence()?;
            let count: i32 = self.poles.len() as i32;
            let weights = self.weights_view();
            self.max_deriv_inv =
                bsplclib::resolution(&self.poles, weights, count, &flat, self.degree(), 1.)?;
            self.max_deriv_inv_ok = true;
        }
        Ok(tolerance * self.max_deriv_inv)
    }
}
