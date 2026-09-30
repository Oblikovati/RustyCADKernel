// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_TBezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dEval_TBezierCurve`: a trigonometric Bezier curve over the basis
//! `{1, sin(alpha t), cos(alpha t), ..., sin(n alpha t), cos(n alpha t)}` on `[0, Pi/alpha]`.
//!
//! The members and the evaluation are shared with GeomEval through [`TBezierData`].
use super::eval_math::{check_derivative_order, unsupported};
use super::tbezier_data::TBezierData;
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
use std::f64::consts::PI;
use std::io;

const CLASS: &str = "Geom2dEval_TBezierCurve";

/// A rational or non-rational T-Bezier curve with 2n + 1 poles; immutable like every
/// eval curve (`Reverse` and `Transform` raise Standard_NotImplemented);
/// `Handle::new(Geom2dEvalTBezierCurve::new(&poles, 1.0)?)`.
pub struct Geom2dEvalTBezierCurve {
    parent: StandardTransient,
    data: TBezierData<Point2>,
}
impl_transient!(
    Geom2dEvalTBezierCurve,
    "Geom2dEval_TBezierCurve",
    dyn Geom2dBoundedCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dBoundedCurve]
);
impl Clone for Geom2dEvalTBezierCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}

impl Geom2dEvalTBezierCurve {
    /// Non-rational curve (`Geom2dEval_TBezierCurve(Poles, Alpha)`); Standard_ConstructionError
    /// unless the pole count is odd and >= 3 and `alpha > 0`;
    /// `Geom2dEvalTBezierCurve::new(&poles, 1.0)?`.
    pub fn new(poles: &Array1<'_, Point2>, alpha: f64) -> Result<Self, Failure> {
        Ok(Self::stored(TBezierData::checked(
            CLASS, poles, None, alpha,
        )?))
    }
    /// Rational curve (`Geom2dEval_TBezierCurve(Poles, Weights, Alpha)`); also refuses a
    /// weight count different from the pole count and weights <= 0. Equal weights still
    /// give a rational curve; `Geom2dEvalTBezierCurve::with_weights(&poles, &weights, 1.0)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point2>,
        weights: &Array1<'_, f64>,
        alpha: f64,
    ) -> Result<Self, Failure> {
        let data = TBezierData::checked(CLASS, poles, Some(weights), alpha)?;
        Ok(Self::stored(data))
    }
    fn stored(data: TBezierData<Point2>) -> Self {
        Self {
            parent: StandardTransient::default(),
            data,
        }
    }
    // Copy() reruns the matching constructor on valid stored values.
    fn copied(&self) -> Self {
        Self::stored(self.data.clone())
    }
    crate::tbezier_accessors!(Point2);
}
impl Geom2dGeometry for Geom2dEvalTBezierCurve {
    fn transform(&self, _transform: Transform2) -> Result<(), Failure> {
        Err(unsupported("Geom2dEval_TBezierCurve::Transform", ""))
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, CLASS)?;
        dump_base(stream, depth, dump_bounded_curve)?;
        self.data.dump_fields(stream)
    }
}
impl Geom2dCurve for Geom2dEvalTBezierCurve {
    crate::unsupported_curve_reversal!(unsupported, "Geom2dEval_TBezierCurve");
    fn first_parameter(&self) -> f64 {
        0.0
    }
    fn last_parameter(&self) -> f64 {
        PI / self.data.alpha
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let distance: f64 = self.start_point()?.distance(self.end_point()?);
        Ok(distance <= precision::confusion())
    }
    crate::non_periodic_cn_curve!();
    crate::tbezier_evaluators!(
        Point2,
        Vector2,
        Geom2dCurveD1,
        Geom2dCurveD2,
        Geom2dCurveD3,
        "Geom2dEval_TBezierCurve"
    );
}
impl Geom2dBoundedCurve for Geom2dEvalTBezierCurve {
    fn end_point(&self) -> Result<Point2, Failure> {
        self.eval_d0(PI / self.data.alpha)
    }
    fn start_point(&self) -> Result<Point2, Failure> {
        self.eval_d0(0.0)
    }
}
