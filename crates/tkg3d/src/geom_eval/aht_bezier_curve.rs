// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_AHTBezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_AHTBezierCurve`: an algebraic-hyperbolic-trigonometric Bezier curve
//! over `{1, t, ..., t^k, sinh(alpha t), cosh(alpha t), sin(beta t), cos(beta t)}` on `[0, 1]`.
//!
//! The evaluation is the Geom2dEval algorithm with `gp_XYZ` sums; both run
//! [`AhtBezierData`].
use super::curve_support::check_derivative_order;
use super::eval_failures::unsupported;
use crate::geom::{
    GeomBoundedCurve, GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, GeomGeometry,
    dump_bounded_curve,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkg2d::geom2d_eval::aht_bezier_data::AhtBezierData;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use std::io;

const CLASS: &str = "GeomEval_AHTBezierCurve";

/// A rational or non-rational AHT-Bezier curve; immutable like every eval curve
/// (`Reverse` and `Transform` raise Standard_NotImplemented);
/// `Handle::new(GeomEvalAHTBezierCurve::new(&poles, 1, 1.0, 1.0)?)`.
pub struct GeomEvalAHTBezierCurve {
    parent: StandardTransient,
    data: AhtBezierData<Point3>,
}
impl_transient!(
    GeomEvalAHTBezierCurve,
    "GeomEval_AHTBezierCurve",
    dyn GeomBoundedCurve,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn GeomBoundedCurve]
);
impl Clone for GeomEvalAHTBezierCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}

impl GeomEvalAHTBezierCurve {
    /// Non-rational curve (`GeomEval_AHTBezierCurve(Poles, AlgDegree, Alpha, Beta)`);
    /// Standard_ConstructionError unless `alg_degree >= 0`, `alpha >= 0`, `beta >= 0` and
    /// the pole count equals the basis dimension.
    ///
    /// The native constructor stores `myWeights(1, 1)`, one uninitialized double; the
    /// port stores 0.0 there; `GeomEvalAHTBezierCurve::new(&poles, 2, 0.0, 0.0)?`.
    pub fn new(
        poles: &Array1<'_, Point3>,
        alg_degree: i32,
        alpha: f64,
        beta: f64,
    ) -> Result<Self, Failure> {
        let data = AhtBezierData::checked(CLASS, poles, None, alg_degree, [alpha, beta])?;
        Ok(Self::stored(data))
    }
    /// Rational curve (`GeomEval_AHTBezierCurve(Poles, Weights, AlgDegree, Alpha, Beta)`);
    /// also refuses a weight count different from the pole count and weights <= 0;
    /// `GeomEvalAHTBezierCurve::with_weights(&poles, &weights, 2, 0.0, 0.0)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
        alg_degree: i32,
        alpha: f64,
        beta: f64,
    ) -> Result<Self, Failure> {
        let frequencies: [f64; 2] = [alpha, beta];
        let data = AhtBezierData::checked(CLASS, poles, Some(weights), alg_degree, frequencies)?;
        Ok(Self::stored(data))
    }
    fn stored(data: AhtBezierData<Point3>) -> Self {
        Self {
            parent: StandardTransient::default(),
            data,
        }
    }
    // Copy() reruns the matching constructor on valid stored values.
    fn copied(&self) -> Self {
        Self::stored(self.data.clone())
    }
    rustycad_tkg2d::aht_bezier_accessors!(Point3);
}
impl GeomGeometry for GeomEvalAHTBezierCurve {
    fn transform(&self, _transform: Transform3) -> Result<(), Failure> {
        Err(unsupported("GeomEval_AHTBezierCurve::Transform", ""))
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, CLASS)?;
        dump_base(stream, depth, dump_bounded_curve)?;
        self.data
            .dump_fields(stream, depth, |pole, nested| pole.dump_json(nested))
    }
}
impl GeomCurve for GeomEvalAHTBezierCurve {
    rustycad_tkg2d::unsupported_curve_reversal!(unsupported, "GeomEval_AHTBezierCurve");
    fn first_parameter(&self) -> f64 {
        0.0
    }
    fn last_parameter(&self) -> f64 {
        1.0
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let distance: f64 = self.start_point()?.distance(&self.end_point()?);
        Ok(distance <= precision::confusion())
    }
    rustycad_tkg2d::non_periodic_cn_curve!();
    rustycad_tkg2d::aht_bezier_evaluators!(
        Point3,
        Vector3,
        GeomCurveD1,
        GeomCurveD2,
        GeomCurveD3,
        "GeomEval_AHTBezierCurve"
    );
}
impl GeomBoundedCurve for GeomEvalAHTBezierCurve {
    fn end_point(&self) -> Result<Point3, Failure> {
        self.eval_d0(1.0)
    }
    fn start_point(&self) -> Result<Point3, Failure> {
        self.eval_d0(0.0)
    }
}
