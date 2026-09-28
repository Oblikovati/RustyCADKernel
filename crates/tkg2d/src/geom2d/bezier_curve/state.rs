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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BezierCurve.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fields of Geom2d_BezierCurve and the native algorithms on them.
use crate::geom2d::arrays::{any_weight_step, at, insert_at, one_based, put, rebased, remove_at};
use crate::geom2d::errors;
use crate::geom2d::{Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use crate::geom2d_eval::{RepCurveDesc, rep_utils};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bsplclib::{self, CurveSpan};
use rustycad_tkmath::gp::{Point2, Transform2, Vector2, resolution};
use rustycad_tkmath::plib;

#[derive(Clone)]
pub(super) struct BezierState {
    pub(super) poles: Array1<'static, Point2>,
    pub(super) weights: Array1<'static, f64>,
    pub(super) eval_rep: Handle<dyn RepCurveDesc>,
    pub(super) rational: bool,
    pub(super) closed: bool,
    pub(super) max_deriv_inv: f64,
    pub(super) max_deriv_inv_ok: bool,
}

// The file-static `Rational(W)` of Geom2d_BezierCurve.cxx: it reads W(1)..W(n) with
// one-based indices whatever the lower bound, so other bounds raise Standard_OutOfRange.
pub(super) fn rational(weights: &Array1<'_, f64>) -> Result<bool, Failure> {
    any_weight_step(weights, 1, weights.len() as i32)
}

impl BezierState {
    /// `init(thePoles, theWeights)` on a fresh state; `BezierState::initialized(&p, None)?`.
    pub(super) fn initialized(
        poles: &Array1<'_, Point2>,
        weights: Option<&Array1<'_, f64>>,
    ) -> Result<Self, Failure> {
        let mut state = BezierState {
            poles: Array1::default(),
            weights: Array1::default(),
            eval_rep: Handle::default(),
            rational: false,
            closed: false,
            max_deriv_inv: 0.0,
            max_deriv_inv_ok: false,
        };
        state.init(poles, weights)?;
        Ok(state)
    }
    // Geom2d_BezierCurve::init: closure flag, one-based copies, rationality, cache and
    // evaluation representation reset.
    pub(super) fn init(
        &mut self,
        poles: &Array1<'_, Point2>,
        weights: Option<&Array1<'_, f64>>,
    ) -> Result<(), Failure> {
        let (first, last) = (at(poles, poles.lower())?, at(poles, poles.upper())?);
        self.closed = first.distance(last) <= resolution();
        self.poles = rebased(poles)?;
        self.rational = false;
        self.weights = bsplclib::unit_weights(poles.len() as i32)?;
        if let Some(weights) = weights {
            let copied: Array1<'static, f64> = rebased(weights)?;
            self.rational = rational(&copied)?;
            if self.rational {
                self.weights = copied;
            }
        }
        self.max_deriv_inv = 0.0;
        self.max_deriv_inv_ok = false;
        self.eval_rep = Handle::default();
        Ok(())
    }
    pub(super) fn degree(&self) -> i32 {
        self.poles.len() as i32 - 1
    }
    /// `Weights()`: the weights of a rational curve, `None` otherwise.
    pub(super) fn weights_view(&self) -> Option<&Array1<'static, f64>> {
        self.rational.then_some(&self.weights)
    }
    fn check_pole_index(&self, index: i32, member: &str) -> Result<(), Failure> {
        if index < 1 || index > self.poles.len() as i32 {
            return Err(errors::out_of_range(&format!(
                "Geom2d_BezierCurve::{member}: Index = {index}; expected in [1, {}]",
                self.poles.len()
            )));
        }
        Ok(())
    }
    pub(super) fn increase(&mut self, degree: i32) -> Result<(), Failure> {
        if degree == self.degree() {
            return Ok(());
        }
        if degree < self.degree() || degree > bsplclib::max_degree() {
            return Err(errors::construction(&format!(
                "Geom2d_BezierCurve::Increase: Degree = {degree}; expected in [{}, {}]",
                self.degree(),
                bsplclib::max_degree()
            )));
        }
        let mut new_poles: Array1<'static, Point2> = one_based(degree + 1)?;
        let (knots, mults) = (knots(), multiplicities(self.degree())?);
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
    pub(super) fn insert_pole_after(
        &mut self,
        index: i32,
        pole: Point2,
        weight: f64,
    ) -> Result<(), Failure> {
        let nbpoles: i32 = self.poles.len() as i32;
        if nbpoles >= bsplclib::max_degree() || weight <= resolution() {
            return Err(errors::construction(&format!(
                "Geom2d_BezierCurve::InsertPoleAfter: NbPoles = {nbpoles}, Weight = {weight}; \
                 expected NbPoles < MaxDegree() and Weight > gp::Resolution()"
            )));
        }
        if index < 0 || index > nbpoles {
            return Err(errors::out_of_range(&format!(
                "Geom2d_BezierCurve::InsertPoleAfter: Index = {index}; expected in [0, {nbpoles}]"
            )));
        }
        let mut new_poles: Array1<'static, Point2> = one_based(nbpoles + 1)?;
        insert_at(&self.poles, &mut new_poles, index, pole)?;
        if !(self.rational || (weight - 1.).abs() > resolution()) {
            return self.init(&new_poles, None);
        }
        let mut new_weights: Array1<'static, f64> = one_based(nbpoles + 1)?;
        let old_weights: Array1<'static, f64> = self.weights_or_units()?;
        insert_at(&old_weights, &mut new_weights, index, weight)?;
        self.init(&new_poles, Some(&new_weights))
    }
    // Stored weights of a rational curve, unit weights otherwise (the native `IsRational() ? W(i) : 1.`).
    fn weights_or_units(&self) -> Result<Array1<'static, f64>, Failure> {
        if self.rational {
            return Ok(self.weights.clone());
        }
        bsplclib::unit_weights(self.poles.len() as i32)
    }
    pub(super) fn remove_pole(&mut self, index: i32) -> Result<(), Failure> {
        let nbpoles: i32 = self.poles.len() as i32;
        if nbpoles <= 2 {
            return Err(errors::construction(&format!(
                "Geom2d_BezierCurve::RemovePole: NbPoles = {nbpoles}; expected > 2"
            )));
        }
        self.check_pole_index(index, "RemovePole")?;
        let mut new_poles: Array1<'static, Point2> = one_based(nbpoles - 1)?;
        remove_at(&self.poles, &mut new_poles, index)?;
        if !self.rational {
            return self.init(&new_poles, None);
        }
        let mut new_weights: Array1<'static, f64> = one_based(nbpoles - 1)?;
        remove_at(&self.weights, &mut new_weights, index)?;
        self.init(&new_poles, Some(&new_weights))
    }
    pub(super) fn reverse(&mut self) {
        self.poles.as_mut_slice().reverse();
        if self.rational {
            self.weights.as_mut_slice().reverse();
        }
        self.max_deriv_inv_ok = false;
    }
    pub(super) fn segment(&mut self, u1: f64, u2: f64) -> Result<(), Failure> {
        let (p1, p2) = (self.eval_d0(u1)?, self.eval_d0(u2)?);
        self.closed = p1.distance(p2).abs() <= resolution();
        let count: i32 = self.poles.len() as i32;
        let flat: Array1<'static, f64> = knot_sequence(self.degree())?;
        let mut coeffs: Array1<'static, Point2> = one_based(count)?;
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
    pub(super) fn set_pole(&mut self, index: i32, pole: Point2) -> Result<(), Failure> {
        self.check_pole_index(index, "SetPole")?;
        put(&mut self.poles, index, pole)?;
        if index == 1 || index == self.poles.len() as i32 {
            let (first, last) = (
                at(&self.poles, 1)?,
                at(&self.poles, self.poles.len() as i32)?,
            );
            self.closed = first.distance(last) <= resolution();
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    pub(super) fn set_weight(&mut self, index: i32, weight: f64) -> Result<(), Failure> {
        self.check_pole_index(index, "SetWeight")?;
        if weight <= resolution() {
            return Err(errors::construction(&format!(
                "Geom2d_BezierCurve::SetWeight: Weight = {weight}; expected > gp::Resolution()"
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
        self.rational = !was_rational || rational(&self.weights)?;
        if !self.rational {
            self.weights = bsplclib::unit_weights(self.poles.len() as i32)?;
        }
        self.max_deriv_inv_ok = false;
        Ok(())
    }
    pub(super) fn transform(&mut self, transform: Transform2) {
        for pole in self.poles.as_mut_slice() {
            pole.transform(transform);
        }
        self.eval_rep = Handle::default();
        self.max_deriv_inv_ok = false;
    }
    pub(super) fn resolution(&mut self, tolerance_uv: f64) -> Result<f64, Failure> {
        if !self.max_deriv_inv_ok {
            let flat: Array1<'static, f64> = knot_sequence(self.degree())?;
            let count: i32 = self.poles.len() as i32;
            let weights = self.weights_view();
            self.max_deriv_inv =
                bsplclib::resolution(&self.poles, weights, count, &flat, self.degree(), 1.)?;
            self.max_deriv_inv_ok = true;
        }
        Ok(tolerance_uv * self.max_deriv_inv)
    }
    pub(super) fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        if let Some(point) = rep_utils::try_eval_curve_d0(&self.eval_rep, u)? {
            return Ok(point);
        }
        bsplclib::bezier_d0(u, &self.poles, self.weights_view())
    }
    pub(super) fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        if let Some(result) = rep_utils::try_eval_curve_d1(&self.eval_rep, u)? {
            return Ok(result);
        }
        let (point, d1) = bsplclib::bezier_d1(u, &self.poles, self.weights_view())?;
        Ok(Geom2dCurveD1 { point, d1 })
    }
    pub(super) fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        if let Some(result) = rep_utils::try_eval_curve_d2(&self.eval_rep, u)? {
            return Ok(result);
        }
        let (point, d1, d2) = bsplclib::bezier_d2(u, &self.poles, self.weights_view())?;
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    pub(super) fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        if let Some(result) = rep_utils::try_eval_curve_d3(&self.eval_rep, u)? {
            return Ok(result);
        }
        let (point, d1, d2, d3) = bsplclib::bezier_d3(u, &self.poles, self.weights_view())?;
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    pub(super) fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Geom2d_BezierCurve::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        if let Some(vector) = rep_utils::try_eval_curve_dn(&self.eval_rep, u, n)? {
            return Ok(vector);
        }
        let (knots, mults) = (knots(), multiplicities(self.degree())?);
        let span = CurveSpan {
            u,
            index: 0,
            degree: self.degree(),
            periodic: false,
            knots: &knots,
            mults: Some(&mults),
        };
        bsplclib::dn(&span, n, &self.poles, self.weights_view())
    }
}

/// The knots `{0, 1}` (`Geom2d_BezierCurve::Knots`); `knots()[2] == 1.0`.
pub(super) fn knots() -> Array1<'static, f64> {
    static THE_DATA: [f64; 2] = [0.0, 1.0];
    Array1::from_shared(&THE_DATA, 1)
}

/// The multiplicities `{Degree+1, Degree+1}` (`Multiplicities`); `multiplicities(3)?[1] == 4`.
pub(super) fn multiplicities(degree: i32) -> Result<Array1<'static, i32>, Failure> {
    if degree < 0 {
        return Err(errors::program(
            "Geom2d_BezierCurve: empty poles; expected at least 2 poles",
        ));
    }
    let mut mults: Array1<'static, i32> = one_based(2)?;
    mults.init(degree + 1);
    Ok(mults)
}

/// The flat knots `Degree+1` zeros then `Degree+1` ones (`KnotSequence`);
/// `knot_sequence(2)?.len() == 6`.
pub(super) fn knot_sequence(degree: i32) -> Result<Array1<'static, f64>, Failure> {
    if degree < 0 {
        return Err(errors::program(
            "Geom2d_BezierCurve: empty poles; expected at least 2 poles",
        ));
    }
    let flat: &'static [f64] = bsplclib::flat_bezier_knots(degree)?;
    Ok(Array1::from_shared(&flat[..2 * (degree as usize + 1)], 1))
}
