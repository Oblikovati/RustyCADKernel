// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_TBezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_TBezierCurve`: a trigonometric Bezier curve over the basis
//! `{1, sin(alpha t), cos(alpha t), ..., sin(n alpha t), cos(n alpha t)}` on `[0, Pi/alpha]`.
//!
//! The evaluation is the Geom2dEval algorithm with `gp_XYZ` sums; both run
//! [`TBezierData`].
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
use rustycad_tkg2d::geom2d_eval::tbezier_data::TBezierData;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use std::f64::consts::PI;
use std::io;

const CLASS: &str = "GeomEval_TBezierCurve";

/// A rational or non-rational T-Bezier curve with 2n + 1 poles; immutable like every
/// eval curve (`Reverse` and `Transform` raise Standard_NotImplemented);
/// `Handle::new(GeomEvalTBezierCurve::new(&poles, 1.0)?)`.
pub struct GeomEvalTBezierCurve {
    parent: StandardTransient,
    data: TBezierData<Point3>,
}
impl_transient!(
    GeomEvalTBezierCurve,
    "GeomEval_TBezierCurve",
    dyn GeomBoundedCurve,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn GeomBoundedCurve]
);
impl Clone for GeomEvalTBezierCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}

impl GeomEvalTBezierCurve {
    /// Non-rational curve (`GeomEval_TBezierCurve(Poles, Alpha)`); Standard_ConstructionError
    /// unless the pole count is odd and >= 3 and `alpha > 0`;
    /// `GeomEvalTBezierCurve::new(&poles, 1.0)?`.
    pub fn new(poles: &Array1<'_, Point3>, alpha: f64) -> Result<Self, Failure> {
        Ok(Self::stored(TBezierData::checked(
            CLASS, poles, None, alpha,
        )?))
    }
    /// Rational curve (`GeomEval_TBezierCurve(Poles, Weights, Alpha)`); also refuses a
    /// weight count different from the pole count and weights <= 0. Equal weights still
    /// give a rational curve; `GeomEvalTBezierCurve::with_weights(&poles, &weights, 1.0)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
        alpha: f64,
    ) -> Result<Self, Failure> {
        let data = TBezierData::checked(CLASS, poles, Some(weights), alpha)?;
        Ok(Self::stored(data))
    }
    fn stored(data: TBezierData<Point3>) -> Self {
        Self {
            parent: StandardTransient::default(),
            data,
        }
    }
    // Copy() reruns the matching constructor on valid stored values.
    fn copied(&self) -> Self {
        Self::stored(self.data.clone())
    }
    rustycad_tkg2d::tbezier_accessors!(Point3);
}
impl GeomGeometry for GeomEvalTBezierCurve {
    fn transform(&self, _transform: Transform3) -> Result<(), Failure> {
        Err(unsupported("GeomEval_TBezierCurve::Transform", ""))
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, CLASS)?;
        dump_base(stream, depth, dump_bounded_curve)?;
        self.data.dump_fields(stream)
    }
}
impl GeomCurve for GeomEvalTBezierCurve {
    rustycad_tkg2d::unsupported_curve_reversal!(unsupported, "GeomEval_TBezierCurve");
    fn first_parameter(&self) -> f64 {
        0.0
    }
    fn last_parameter(&self) -> f64 {
        PI / self.data.alpha
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let distance: f64 = self.start_point()?.distance(&self.end_point()?);
        Ok(distance <= precision::confusion())
    }
    rustycad_tkg2d::non_periodic_cn_curve!();
    rustycad_tkg2d::tbezier_evaluators!(
        Point3,
        Vector3,
        GeomCurveD1,
        GeomCurveD2,
        GeomCurveD3,
        "GeomEval_TBezierCurve"
    );
}
impl GeomBoundedCurve for GeomEvalTBezierCurve {
    fn end_point(&self) -> Result<Point3, Failure> {
        self.eval_d0(PI / self.data.alpha)
    }
    fn start_point(&self) -> Result<Point3, Failure> {
        self.eval_d0(0.0)
    }
}
