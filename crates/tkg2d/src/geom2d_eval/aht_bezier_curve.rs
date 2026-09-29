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
// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_AHTBezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dEval_AHTBezierCurve`: an algebraic-hyperbolic-trigonometric Bezier curve
//! over `{1, t, ..., t^k, sinh(alpha t), cosh(alpha t), sin(beta t), cos(beta t)}` on `[0, 1]`.
//!
//! The members and the evaluation are shared with GeomEval through
//! [`AhtBezierData`].
use super::aht_bezier_data::AhtBezierData;
use super::eval_math::{check_derivative_order, unsupported};
use crate::geom2d::{
    Geom2dBoundedCurve, Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, Geom2dGeometry,
    dump_base, dump_bounded_curve, dump_class_name,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};
use std::io;

const CLASS: &str = "Geom2dEval_AHTBezierCurve";

/// A rational or non-rational AHT-Bezier curve; immutable like every eval curve
/// (`Reverse` and `Transform` raise Standard_NotImplemented);
/// `Handle::new(Geom2dEvalAHTBezierCurve::new(&poles, 1, 1.0, 1.0)?)`.
pub struct Geom2dEvalAHTBezierCurve {
    parent: StandardTransient,
    data: AhtBezierData<Point2>,
}
impl_transient!(
    Geom2dEvalAHTBezierCurve,
    "Geom2dEval_AHTBezierCurve",
    dyn Geom2dBoundedCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dBoundedCurve]
);
impl Clone for Geom2dEvalAHTBezierCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}

impl Geom2dEvalAHTBezierCurve {
    /// Non-rational curve (`Geom2dEval_AHTBezierCurve(Poles, AlgDegree, Alpha, Beta)`);
    /// Standard_ConstructionError unless `alg_degree >= 0`, `alpha >= 0`, `beta >= 0` and
    /// the pole count equals the basis dimension.
    ///
    /// The native constructor stores `myWeights(1, 1)`, one uninitialized double; the
    /// port stores 0.0 there; `Geom2dEvalAHTBezierCurve::new(&poles, 2, 0.0, 0.0)?`.
    pub fn new(
        poles: &Array1<'_, Point2>,
        alg_degree: i32,
        alpha: f64,
        beta: f64,
    ) -> Result<Self, Failure> {
        let data = AhtBezierData::checked(CLASS, poles, None, alg_degree, [alpha, beta])?;
        Ok(Self::stored(data))
    }
    /// Rational curve (`Geom2dEval_AHTBezierCurve(Poles, Weights, AlgDegree, Alpha, Beta)`);
    /// also refuses a weight count different from the pole count and weights <= 0;
    /// `Geom2dEvalAHTBezierCurve::with_weights(&poles, &weights, 2, 0.0, 0.0)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point2>,
        weights: &Array1<'_, f64>,
        alg_degree: i32,
        alpha: f64,
        beta: f64,
    ) -> Result<Self, Failure> {
        let frequencies: [f64; 2] = [alpha, beta];
        let data = AhtBezierData::checked(CLASS, poles, Some(weights), alg_degree, frequencies)?;
        Ok(Self::stored(data))
    }
    fn stored(data: AhtBezierData<Point2>) -> Self {
        Self {
            parent: StandardTransient::default(),
            data,
        }
    }
    // Copy() reruns the matching constructor on valid stored values.
    fn copied(&self) -> Self {
        Self::stored(self.data.clone())
    }
    crate::aht_bezier_accessors!(Point2);
}
impl Geom2dGeometry for Geom2dEvalAHTBezierCurve {
    fn transform(&self, _transform: Transform2) -> Result<(), Failure> {
        Err(unsupported("Geom2dEval_AHTBezierCurve::Transform", ""))
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, CLASS)?;
        dump_base(stream, depth, dump_bounded_curve)?;
        self.data
            .dump_fields(stream, depth, |pole, nested| pole.dump_json(nested))
    }
}
impl Geom2dCurve for Geom2dEvalAHTBezierCurve {
    crate::unsupported_curve_reversal!(unsupported, "Geom2dEval_AHTBezierCurve");
    fn first_parameter(&self) -> f64 {
        0.0
    }
    fn last_parameter(&self) -> f64 {
        1.0
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let distance: f64 = self.start_point()?.distance(self.end_point()?);
        Ok(distance <= precision::confusion())
    }
    crate::non_periodic_cn_curve!();
    crate::aht_bezier_evaluators!(
        Point2,
        Vector2,
        Geom2dCurveD1,
        Geom2dCurveD2,
        Geom2dCurveD3,
        "Geom2dEval_AHTBezierCurve"
    );
}
impl Geom2dBoundedCurve for Geom2dEvalAHTBezierCurve {
    fn end_point(&self) -> Result<Point2, Failure> {
        self.eval_d0(1.0)
    }
    fn start_point(&self) -> Result<Point2, Failure> {
        self.eval_d0(0.0)
    }
}
