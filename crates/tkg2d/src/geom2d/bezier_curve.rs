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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_BezierCurve`: a rational or non-rational Bezier curve on `[0, 1]`.
use super::arrays::at;
use super::bounded_curve::{Geom2dBoundedCurve, dump_bounded_curve};
use super::curve::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use super::geometry::{Geom2dGeometry, dump_base, dump_class_name};
use super::spline_kind::BezierState;
use crate::geom2d_eval::{RepCurveDesc, rep_utils};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};
use std::io;

/// A Bezier curve; `Handle::new(Geom2dBezierCurve::new(&poles)?)`.
pub struct Geom2dBezierCurve {
    parent: StandardTransient,
    state: SharedState<BezierState>,
}
impl_transient!(
    Geom2dBezierCurve,
    "Geom2d_BezierCurve",
    dyn Geom2dBoundedCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dBoundedCurve]
);

impl Geom2dBezierCurve {
    fn from_state(state: BezierState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Non-rational curve (`Geom2d_BezierCurve(CurvePoles)`); 2 to MaxDegree()+1 poles;
    /// `Geom2dBezierCurve::new(&poles)?`.
    pub fn new(poles: &Array1<'_, Point2>) -> Result<Self, Failure> {
        Ok(Self::from_state(BezierState::checked(poles, None)?))
    }
    /// Rational curve (`Geom2d_BezierCurve(CurvePoles, PoleWeights)`); equal weights give a
    /// non-rational curve; `Geom2dBezierCurve::with_weights(&poles, &weights)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point2>,
        weights: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        Ok(Self::from_state(BezierState::checked(
            poles,
            Some(weights),
        )?))
    }
    /// The native copy constructor with its failures (`Geom2d_BezierCurve(const Geom2d_BezierCurve&)`);
    /// `Geom2dBezierCurve::copy_of(&curve)?`.
    pub fn copy_of(other: &Geom2dBezierCurve) -> Result<Self, Failure> {
        let mut copy: BezierState = other.state.snapshot();
        copy.eval_rep = rep_utils::clone_curve_desc(&copy.eval_rep)?;
        copy.reset_copied()?;
        Ok(Self::from_state(copy))
    }
    /// Whether an evaluation representation is set (`HasEvalRepresentation`); `curve.has_eval_representation()`.
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
    /// Raise the degree (`Increase`); `curve.increase(5)?`.
    pub fn increase(&self, degree: i32) -> Result<(), Failure> {
        self.state.write().increase(degree)
    }
    /// Insert a pole after position `index` in 0..=NbPoles (`InsertPoleAfter`);
    /// `curve.insert_pole_after(2, p, 1.0)?`.
    pub fn insert_pole_after(&self, index: i32, pole: Point2, weight: f64) -> Result<(), Failure> {
        self.state.write().insert_pole_after(index, pole, weight)
    }
    /// Insert a pole before position `index` (`InsertPoleBefore`); `curve.insert_pole_before(1, p, 1.0)?`.
    pub fn insert_pole_before(&self, index: i32, pole: Point2, weight: f64) -> Result<(), Failure> {
        self.insert_pole_after(index - 1, pole, weight)
    }
    /// Remove the pole at `index` (`RemovePole`); `curve.remove_pole(2)?`.
    pub fn remove_pole(&self, index: i32) -> Result<(), Failure> {
        self.state.write().remove_pole(index)
    }
    /// Restrict to `[u1, u2]` and reparameterize on `[0, 1]` (`Segment`); `curve.segment(0.25, 0.75)?`.
    pub fn segment(&self, u1: f64, u2: f64) -> Result<(), Failure> {
        self.state.write().segment(u1, u2)
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
    /// Whether the weights differ (`IsRational`); `curve.is_rational()`.
    pub fn is_rational(&self) -> bool {
        self.state.read().rational
    }
    /// Number of poles minus one (`Degree`); `curve.degree()`.
    pub fn degree(&self) -> i32 {
        self.state.read().degree()
    }
    /// Number of poles (`NbPoles`); `curve.nb_poles()`.
    pub fn nb_poles(&self) -> i32 {
        self.state.read().poles.len() as i32
    }
    /// Pole at `index` (`Pole`); `curve.pole(1)?`.
    pub fn pole(&self, index: i32) -> Result<Point2, Failure> {
        let state = self.state.read();
        super::arrays::check_one_based(index, state.poles.len(), "Geom2d_BezierCurve::Pole")?;
        at(&state.poles, index)
    }
    /// Copy the poles into `target` of equal length (deprecated `Poles(P)`);
    /// `curve.poles_into(&mut poles)?`.
    pub fn poles_into(&self, target: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        let state = self.state.read();
        super::arrays::copy_same_length(&state.poles, target, "Geom2d_BezierCurve::Poles")
    }
    /// The poles, indexed from 1 (`Poles()`); `curve.poles()`.
    pub fn poles(&self) -> Array1<'static, Point2> {
        self.state.read().poles.clone()
    }
    /// Weight at `index`; 1 for a non-rational curve (`Weight`); `curve.weight(1)?`.
    pub fn weight(&self, index: i32) -> Result<f64, Failure> {
        self.state.read().weight(index)
    }
    /// Copy the weights into `target` of equal length, writing `target(1..=n)` for a
    /// non-rational curve (deprecated `Weights(W)`); `curve.weights_into(&mut w)?`.
    pub fn weights_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.state.read().weights_into(target)
    }
    /// The weights of a rational curve, `None` otherwise (`Weights()`); `curve.weights()`.
    pub fn weights(&self) -> Option<Array1<'static, f64>> {
        self.state.read().weights_view().cloned()
    }
    /// Read the stored weights in place, unit weights (a non-owning shared view) for a
    /// non-rational curve (`WeightsArray`, which natively returns a stable reference);
    /// `curve.weights_array(|weights| weights.len())`.
    pub fn weights_array<R>(&self, read: impl FnOnce(&Array1<'static, f64>) -> R) -> R {
        read(&self.state.read().weights)
    }
    /// `BSplCLib::MaxDegree()` (`MaxDegree`); `Geom2dBezierCurve::max_degree() == 25`.
    pub fn max_degree() -> i32 {
        bsplclib::max_degree()
    }
    /// Parametric tolerance for a 2D tolerance, caching the inverse derivative bound
    /// (`Resolution`); `curve.resolution(1e-7)?`.
    pub fn resolution(&self, tolerance_uv: f64) -> Result<f64, Failure> {
        self.state.write().resolution(tolerance_uv)
    }
    /// The knots `{0, 1}` (`Knots`); `curve.knots()`.
    pub fn knots(&self) -> Array1<'static, f64> {
        crate::geom2d::spline_data::bezier_knots()
    }
    /// The multiplicities `{Degree+1, Degree+1}` (`Multiplicities`); `curve.multiplicities()?`.
    pub fn multiplicities(&self) -> Result<Array1<'static, i32>, Failure> {
        self.state.read().multiplicities()
    }
    /// The flat knots (`KnotSequence`); `curve.knot_sequence()?`.
    pub fn knot_sequence(&self) -> Result<Array1<'static, f64>, Failure> {
        self.state.read().knot_sequence()
    }
}
impl Geom2dGeometry for Geom2dBezierCurve {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        self.state
            .write()
            .transform_poles(|pole| pole.transform(transform));
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(Self::copy_of(self)?).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        dump_class_name(stream, "Geom2d_BezierCurve")?;
        dump_base(stream, depth, dump_bounded_curve)?;
        state.dump_fields(stream)
    }
}
impl Geom2dCurve for Geom2dBezierCurve {
    fn reverse(&self) -> Result<(), Failure> {
        self.state.write().reverse();
        Ok(())
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(1. - u)
    }
    fn first_parameter(&self) -> f64 {
        0.0
    }
    fn last_parameter(&self) -> f64 {
        1.0
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        Ok(self.state.read().closed)
    }
    fn is_periodic(&self) -> bool {
        false
    }
    fn continuity(&self) -> Shape {
        Shape::CN
    }
    fn is_cn(&self, _n: i32) -> Result<bool, Failure> {
        Ok(true)
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        self.state.read().eval_d0(u)
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let (point, d1) = self.state.read().eval_d1(u)?;
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let (point, d1, d2) = self.state.read().eval_d2(u)?;
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let (point, d1, d2, d3) = self.state.read().eval_d3(u)?;
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        self.state.read().eval_dn(u, n)
    }
}
impl Geom2dBoundedCurve for Geom2dBezierCurve {
    fn end_point(&self) -> Result<Point2, Failure> {
        let state = self.state.read();
        at(&state.poles, state.poles.upper())
    }
    fn start_point(&self) -> Result<Point2, Failure> {
        at(&self.state.read().poles, 1)
    }
}
