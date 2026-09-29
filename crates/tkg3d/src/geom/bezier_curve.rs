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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_BezierCurve`: a rational or non-rational Bezier curve on `[0, 1]`.
use super::bounded_curve::{GeomBoundedCurve, dump_bounded_curve};
use super::curve::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use super::geometry::GeomGeometry;
use super::spline_kind::BezierState;
use crate::geom_eval::{RepCurveDesc, rep_utils};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::{at, check_one_based, copy_same_length};
use rustycad_tkg2d::geom2d::spline_data::bezier_knots;
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use std::io;

/// A Bezier curve; `Handle::new(GeomBezierCurve::new(&poles)?)`.
pub struct GeomBezierCurve {
    parent: StandardTransient,
    state: SharedState<BezierState>,
}
impl_transient!(
    GeomBezierCurve,
    "Geom_BezierCurve",
    dyn GeomBoundedCurve,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn GeomBoundedCurve]
);

impl GeomBezierCurve {
    fn from_state(state: BezierState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Non-rational curve (`Geom_BezierCurve(CurvePoles)`); 2 to MaxDegree()+1 poles;
    /// `GeomBezierCurve::new(&poles)?`.
    pub fn new(poles: &Array1<'_, Point3>) -> Result<Self, Failure> {
        Ok(Self::from_state(BezierState::checked(poles, None)?))
    }
    /// Rational curve (`Geom_BezierCurve(CurvePoles, PoleWeights)`); equal weights give a
    /// non-rational curve; `GeomBezierCurve::with_weights(&poles, &weights)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        Ok(Self::from_state(BezierState::checked(
            poles,
            Some(weights),
        )?))
    }
    /// The native copy constructor (`Geom_BezierCurve(const Geom_BezierCurve&)`): the
    /// evaluation representation is deep-copied; `GeomBezierCurve::copy_of(&curve)?`.
    pub fn copy_of(other: &GeomBezierCurve) -> Result<Self, Failure> {
        let mut copy: BezierState = other.state.snapshot();
        copy.eval_rep = rep_utils::clone_curve_desc(&copy.eval_rep)?;
        copy.reset_copied()?;
        Ok(Self::from_state(copy))
    }
    rustycad_tkg2d::eval_representation_members!(
        dyn RepCurveDesc,
        rep_utils::validate_curve_desc,
        "curve"
    );
    /// Raise the degree (`Increase`); `curve.increase(5)?`.
    pub fn increase(&self, degree: i32) -> Result<(), Failure> {
        self.state.write().increase(degree)
    }
    /// Insert a pole of weight 1 after position `index` in 0..=NbPoles
    /// (`InsertPoleAfter(Index, P)`); `curve.insert_pole_after(2, p)?`.
    pub fn insert_pole_after(&self, index: i32, pole: Point3) -> Result<(), Failure> {
        self.insert_pole_after_weighted(index, pole, 1.)
    }
    /// Insert a weighted pole after position `index` (`InsertPoleAfter(Index, P, Weight)`);
    /// `curve.insert_pole_after_weighted(2, p, 2.0)?`.
    pub fn insert_pole_after_weighted(
        &self,
        index: i32,
        pole: Point3,
        weight: f64,
    ) -> Result<(), Failure> {
        self.state.write().insert_pole_after(index, pole, weight)
    }
    /// Insert a pole of weight 1 before position `index` (`InsertPoleBefore(Index, P)`);
    /// `curve.insert_pole_before(1, p)?`.
    pub fn insert_pole_before(&self, index: i32, pole: Point3) -> Result<(), Failure> {
        self.insert_pole_after(index - 1, pole)
    }
    /// Insert a weighted pole before position `index` (`InsertPoleBefore(Index, P, Weight)`);
    /// `curve.insert_pole_before_weighted(1, p, 2.0)?`.
    pub fn insert_pole_before_weighted(
        &self,
        index: i32,
        pole: Point3,
        weight: f64,
    ) -> Result<(), Failure> {
        self.insert_pole_after_weighted(index - 1, pole, weight)
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
    pub fn set_pole(&self, index: i32, pole: Point3) -> Result<(), Failure> {
        self.state.write().set_pole(index, pole)
    }
    /// Replace a pole and its weight (`SetPole(Index, P, Weight)`); `curve.set_pole_weighted(2, p, 2.0)?`.
    pub fn set_pole_weighted(&self, index: i32, pole: Point3, weight: f64) -> Result<(), Failure> {
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
    pub fn pole(&self, index: i32) -> Result<Point3, Failure> {
        let state = self.state.read();
        check_one_based(index, state.poles.len(), "Geom_BezierCurve::Pole")?;
        at(&state.poles, index)
    }
    /// Copy the poles into `target` of equal length (deprecated `Poles(P)`);
    /// `curve.poles_into(&mut poles)?`.
    pub fn poles_into(&self, target: &mut Array1<'_, Point3>) -> Result<(), Failure> {
        let state = self.state.read();
        copy_same_length(&state.poles, target, "Geom_BezierCurve::Poles")
    }
    /// The poles, indexed from 1 (`Poles()`); `curve.poles()`.
    pub fn poles(&self) -> Array1<'static, Point3> {
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
    /// `BSplCLib::MaxDegree()` (`MaxDegree`); `GeomBezierCurve::max_degree() == 25`.
    pub fn max_degree() -> i32 {
        bsplclib::max_degree()
    }
    /// Parametric tolerance for a 3D tolerance, caching the inverse derivative bound
    /// (`Resolution`); `curve.resolution(1e-7)?`.
    pub fn resolution(&self, tolerance_3d: f64) -> Result<f64, Failure> {
        self.state.write().resolution(tolerance_3d)
    }
    /// The knots `{0, 1}` (`Knots`); `curve.knots()`.
    pub fn knots(&self) -> Array1<'static, f64> {
        bezier_knots()
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
impl GeomGeometry for GeomBezierCurve {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        self.state
            .write()
            .transform_poles(|pole| pole.transform(transform));
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(Self::copy_of(self)?).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        dump_class_name(stream, "Geom_BezierCurve")?;
        dump_base(stream, depth, dump_bounded_curve)?;
        state.dump_fields(stream)
    }
}
impl GeomCurve for GeomBezierCurve {
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
    rustycad_tkg2d::non_periodic_cn_curve!();
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        self.state.read().eval_d0(u)
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let (point, d1) = self.state.read().eval_d1(u)?;
        Ok(GeomCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let (point, d1, d2) = self.state.read().eval_d2(u)?;
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let (point, d1, d2, d3) = self.state.read().eval_d3(u)?;
        Ok(GeomCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        self.state.read().eval_dn(u, n)
    }
}
impl GeomBoundedCurve for GeomBezierCurve {
    fn end_point(&self) -> Result<Point3, Failure> {
        let state = self.state.read();
        at(&state.poles, state.poles.upper())
    }
    fn start_point(&self) -> Result<Point3, Failure> {
        at(&self.state.read().poles, 1)
    }
}
