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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve.hxx/.cxx, Geom2d_BSplineCurve_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_BSplineCurve`: rational or non-rational, periodic or not, B-spline curves.
mod access;
mod poles;
mod traits;
use super::arrays::at;
use super::bounded_curve::Geom2dBoundedCurve;
use super::bspline_curve_core::GivenWeights;
use super::curve::Geom2dCurve;
use super::errors;
use super::geometry::Geom2dGeometry;
use super::spline_data::check_curve_data;
use super::spline_kind::BSplineState;
use crate::geom2d_eval::{RepCurveDesc, rep_utils};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point2, Vector2, resolution};

/// A B-spline curve; `Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, 3, false)?)`.
pub struct Geom2dBSplineCurve {
    parent: StandardTransient,
    state: SharedState<BSplineState>,
}
impl_transient!(
    Geom2dBSplineCurve,
    "Geom2d_BSplineCurve",
    dyn Geom2dBoundedCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dBoundedCurve]
);

// The weight checks of the rational constructor.
fn check_weights(weights: &Array1<'_, f64>, nbpoles: usize) -> Result<(), Failure> {
    if weights.len() != nbpoles {
        return Err(errors::construction(&format!(
            "Geom2d_BSplineCurve: {} weights for {nbpoles} poles; Weights and Poles array size mismatch",
            weights.len()
        )));
    }
    for i in weights.lower()..=weights.upper() {
        let weight: f64 = at(weights, i)?;
        if weight <= resolution() {
            return Err(errors::construction(&format!(
                "Geom2d_BSplineCurve: Weights({i}) = {weight}; Weights values too small"
            )));
        }
    }
    Ok(())
}

impl Geom2dBSplineCurve {
    fn from_state(state: BSplineState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Non-rational curve (`Geom2d_BSplineCurve(Poles, Knots, Multiplicities, Degree, Periodic)`);
    /// Standard_ConstructionError for inconsistent data;
    /// `Geom2dBSplineCurve::new(&poles, &knots, &mults, 3, false)?`.
    pub fn new(
        poles: &Array1<'_, Point2>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
    ) -> Result<Self, Failure> {
        check_curve_data(poles, knots, mults, degree, periodic)?;
        let state = BSplineState::from_arrays(poles, None, knots, mults, degree, periodic)?;
        Ok(Self::from_state(state))
    }
    /// Rational curve (`Geom2d_BSplineCurve(Poles, Weights, Knots, Multiplicities, Degree, Periodic)`);
    /// equal weights give a non-rational curve;
    /// `Geom2dBSplineCurve::with_weights(&poles, &weights, &knots, &mults, 2, false)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point2>,
        weights: &Array1<'_, f64>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        degree: i32,
        periodic: bool,
    ) -> Result<Self, Failure> {
        check_curve_data(poles, knots, mults, degree, periodic)?;
        check_weights(weights, poles.len())?;
        let given = GivenWeights {
            weights,
            check_rational: true,
        };
        let state = BSplineState::from_arrays(poles, Some(given), knots, mults, degree, periodic)?;
        Ok(Self::from_state(state))
    }
    /// The native copy constructor: the evaluation representation is deep-copied, the
    /// cached resolution value is kept but marked stale
    /// (`Geom2d_BSplineCurve(const Geom2d_BSplineCurve&)`); `Geom2dBSplineCurve::copy_of(&curve)?`.
    pub fn copy_of(other: &Geom2dBSplineCurve) -> Result<Self, Failure> {
        let mut copy: BSplineState = other.state.snapshot();
        copy.eval_rep = rep_utils::clone_curve_desc(&copy.eval_rep)?;
        if !copy.rational {
            copy.reset_unit_weights()?;
        }
        copy.max_deriv_inv_ok = false;
        Ok(Self::from_state(copy))
    }
    /// Whether an evaluation representation is set (`HasEvalRepresentation`);
    /// `curve.has_eval_representation()`.
    pub fn has_eval_representation(&self) -> bool {
        self.state.read().eval_rep.is_some()
    }
    /// The evaluation representation, possibly null (`EvalRepresentation`); `curve.eval_representation()`.
    pub fn eval_representation(&self) -> Handle<dyn RepCurveDesc> {
        self.state.read().eval_rep.clone()
    }
    /// Validate and set the evaluation representation (`SetEvalRepresentation`);
    /// `curve.set_eval_representation(desc)?`.
    pub fn set_eval_representation(&self, desc: Handle<dyn RepCurveDesc>) -> Result<(), Failure> {
        rep_utils::validate_curve_desc(&desc, std::ptr::from_ref(self).cast())?;
        self.state.write().eval_rep = desc;
        Ok(())
    }
    /// Remove the evaluation representation (`ClearEvalRepresentation`); `curve.clear_eval_representation()`.
    pub fn clear_eval_representation(&self) {
        self.state.write().eval_rep = Handle::default();
    }
    /// Raise the degree (`IncreaseDegree`); `curve.increase_degree(5)?`.
    pub fn increase_degree(&self, degree: i32) -> Result<(), Failure> {
        self.state.write().increase_degree(degree)
    }
    /// Raise the multiplicity of knot `index` to `mult` (`IncreaseMultiplicity(Index, M)`);
    /// `curve.increase_multiplicity(2, 3)?`.
    pub fn increase_multiplicity(&self, index: i32, mult: i32) -> Result<(), Failure> {
        self.state.write().increase_multiplicity(index, mult)
    }
    /// Raise the multiplicities of knots `i1..=i2` to `mult` (`IncreaseMultiplicity(I1, I2, M)`);
    /// `curve.increase_multiplicity_range(2, 3, 3)?`.
    pub fn increase_multiplicity_range(&self, i1: i32, i2: i32, mult: i32) -> Result<(), Failure> {
        self.state.write().raise_multiplicities(i1, i2, mult, false)
    }
    /// Add `step` to the multiplicities of knots `i1..=i2` (`IncrementMultiplicity`);
    /// `curve.increment_multiplicity(2, 3, 1)?`.
    pub fn increment_multiplicity(&self, i1: i32, i2: i32, step: i32) -> Result<(), Failure> {
        self.state.write().raise_multiplicities(i1, i2, step, true)
    }
    /// Insert knot `u` with multiplicity `mult` (`InsertKnot(U, M, ParametricTolerance)`);
    /// `curve.insert_knot(0.5, 1, 0.0)?`.
    pub fn insert_knot(&self, u: f64, mult: i32, parametric_tolerance: f64) -> Result<(), Failure> {
        let mut knots: Array1<'static, f64> = super::arrays::one_based(1)?;
        super::arrays::put(&mut knots, 1, u)?;
        let mut mults: Array1<'static, i32> = super::arrays::one_based(1)?;
        super::arrays::put(&mut mults, 1, mult)?;
        self.insert_knots(&knots, &mults, parametric_tolerance, false)
    }
    /// Insert knots (`InsertKnots(Knots, Mults, ParametricTolerance, Add)`); with `add` the
    /// multiplicities of existing knots are increased by `mults`, otherwise raised to them;
    /// `curve.insert_knots(&knots, &mults, 0.0, false)?`.
    pub fn insert_knots(
        &self,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        parametric_tolerance: f64,
        add: bool,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .insert_knots(knots, mults, parametric_tolerance, add)
    }
    /// Reduce knot `index` to multiplicity `mult` if the curve moves less than
    /// `tolerance`; false when it would (`RemoveKnot`); `curve.remove_knot(2, 0, 1e-7)?`.
    pub fn remove_knot(&self, index: i32, mult: i32, tolerance: f64) -> Result<bool, Failure> {
        self.state.write().remove_knot(index, mult, tolerance)
    }
    /// Insert a pole after `index` on a uniform or quasi-uniform curve (`InsertPoleAfter`);
    /// `curve.insert_pole_after(2, p, 1.0)?`.
    pub fn insert_pole_after(&self, index: i32, pole: Point2, weight: f64) -> Result<(), Failure> {
        self.state.write().insert_pole_after(index, pole, weight)
    }
    /// Insert a pole before `index` (`InsertPoleBefore`); `curve.insert_pole_before(1, p, 1.0)?`.
    pub fn insert_pole_before(&self, index: i32, pole: Point2, weight: f64) -> Result<(), Failure> {
        self.insert_pole_after(index - 1, pole, weight)
    }
    /// Remove the pole at `index` on a uniform or quasi-uniform curve (`RemovePole`);
    /// `curve.remove_pole(2)?`.
    pub fn remove_pole(&self, index: i32) -> Result<(), Failure> {
        self.state.write().remove_pole(index)
    }
    /// Restrict to `[u1, u2]` (`Segment`); `curve.segment(0.2, 0.8, precision::p_confusion())?`.
    pub fn segment(&self, u1: f64, u2: f64, tolerance: f64) -> Result<(), Failure> {
        self.state.write().segment(u1, u2, tolerance)
    }
    /// Move knot `index` strictly between its neighbours (`SetKnot(Index, K)`); `curve.set_knot(2, 0.4)?`.
    pub fn set_knot(&self, index: i32, knot: f64) -> Result<(), Failure> {
        self.state.write().set_knot(index, knot)
    }
    /// Replace all knot values (`SetKnots`); `curve.set_knots(&knots)?`.
    pub fn set_knots(&self, knots: &Array1<'_, f64>) -> Result<(), Failure> {
        self.state.write().set_knots(knots)
    }
    /// Raise the multiplicity of knot `index` to `mult`, then move it (`SetKnot(Index, K, M)`);
    /// `curve.set_knot_with_multiplicity(2, 0.4, 2)?`.
    pub fn set_knot_with_multiplicity(
        &self,
        index: i32,
        knot: f64,
        mult: i32,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.increase_multiplicity(index, mult)?;
        state.set_knot(index, knot)
    }
    /// Shift a parameter of a periodic curve into its range (`PeriodicNormalization`);
    /// `curve.periodic_normalization(7.0)?`.
    pub fn periodic_normalization(&self, u: f64) -> Result<f64, Failure> {
        self.state.read().periodic_normalization(u)
    }
    /// Make the curve periodic (`SetPeriodic`); `curve.set_periodic()?`.
    pub fn set_periodic(&self) -> Result<(), Failure> {
        self.state.write().set_periodic()
    }
    /// Make knot `index` the origin of a periodic curve (`SetOrigin`); `curve.set_origin(2)?`.
    pub fn set_origin(&self, index: i32) -> Result<(), Failure> {
        self.state.write().set_origin(index)
    }
    /// Make the curve non-periodic (`SetNotPeriodic`); `curve.set_not_periodic()?`.
    pub fn set_not_periodic(&self) -> Result<(), Failure> {
        self.state.write().set_not_periodic()
    }
    /// Replace a pole (`SetPole(Index, P)`); `curve.set_pole(2, p)?`.
    pub fn set_pole(&self, index: i32, pole: Point2) -> Result<(), Failure> {
        self.state.write().set_pole(index, pole)
    }
    /// Replace a pole and its weight (`SetPole(Index, P, Weight)`); `curve.set_pole_weighted(2, p, 2.0)?`.
    pub fn set_pole_weighted(&self, index: i32, pole: Point2, weight: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.set_pole(index, pole)?;
        state.set_weight(index, weight)
    }
    /// Replace a weight (`SetWeight`); `curve.set_weight(2, 2.0)?`.
    pub fn set_weight(&self, index: i32, weight: f64) -> Result<(), Failure> {
        self.state.write().set_weight(index, weight)
    }
    /// Move the point of parameter `u` to `point` by changing poles `index1..=index2`; returns
    /// `(FirstModifiedPole, LastModifiedPole)`, `(0, 0)` when nothing moved (`MovePoint`);
    /// `curve.move_point(0.5, p, 2, 3)?`.
    pub fn move_point(
        &self,
        u: f64,
        point: Point2,
        index1: i32,
        index2: i32,
    ) -> Result<(i32, i32), Failure> {
        self.state.write().move_point(u, point, index1, index2)
    }
    /// Move the point and tangent of parameter `u`; returns `ErrorStatus`, 0 on success
    /// (`MovePointAndTangent`); `curve.move_point_and_tangent(0.5, p, t, 1e-7, 1, 1)?`.
    pub fn move_point_and_tangent(
        &self,
        u: f64,
        point: Point2,
        tangent: Vector2,
        tolerance: f64,
        starting_condition: i32,
        ending_condition: i32,
    ) -> Result<i32, Failure> {
        let conditions: (i32, i32) = (starting_condition, ending_condition);
        self.state
            .write()
            .move_point_and_tangent(u, point, tangent, tolerance, conditions)
    }
    /// Tangent continuity within `ang_tol` on `[tf, tl]` (`IsG1`); `curve.is_g1(0.0, 1.0, 1e-12)?`.
    pub fn is_g1(&self, tf: f64, tl: f64, ang_tol: f64) -> Result<bool, Failure> {
        self.state.read().is_g1(tf, tl, ang_tol)
    }
    /// `BSplCLib::MaxDegree()` (`MaxDegree`); `Geom2dBSplineCurve::max_degree() == 25`.
    pub fn max_degree() -> i32 {
        bsplclib::max_degree()
    }
    /// Parametric tolerance for a 2D tolerance, caching the inverse derivative bound
    /// (`Resolution`); `curve.resolution(1e-7)?`.
    pub fn resolution(&self, tolerance_uv: f64) -> Result<f64, Failure> {
        self.state.write().resolution(tolerance_uv)
    }
}
