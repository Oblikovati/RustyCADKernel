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
use rustycad_tkg2d::geom2d::arrays::at;
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
    rustycad_tkg2d::bezier_curve_members!(Point3);
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
