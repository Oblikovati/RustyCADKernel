// Rust translation of ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_BSplineCurve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC2d_BSplineCurve`: numerical extrema on a BSpline curve over a knot-aware
//! partition.
use super::bezier_curve::spline_value;
use super::{
    CurveEvaluator, Domain1D, ExtremaResult, GridEvaluator, SearchMode, Status,
    THE_BSPLINE_FALLBACK_SAMPLES, add_endpoint_extrema,
};
use crate::extrema_pc_core::knot_params::{KnotGrid, knot_aware_params};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::Vector;

/// Point-BSpline extrema (`ExtremaPC2d_BSplineCurve`).
///
/// Each knot span of the domain (every period of it for a periodic curve) gets
/// 2 (degree + 1) samples; the partition is rebuilt on every query, and a grid too
/// large for an `int` count is Standard_RangeError;
/// `BSplineCurve::new(curve).perform(point, 1e-9, SearchMode::MinMax)?`.
pub struct BSplineCurve {
    shape: BSplineShape,
    domain: Domain1D,
    evaluator: GridEvaluator,
}

/// The curve, evaluated by AddEndpointExtrema apart from the evaluator's result.
struct BSplineShape(Handle<Geom2dBSplineCurve>);

impl CurveEvaluator<Point2> for BSplineShape {
    type Error = Failure;
    fn evaluate(&self, u: f64) -> Result<Point2, Failure> {
        spline_value(&self.0, u)
    }
}

impl BSplineCurve {
    /// Evaluator over the curve bounds (`ExtremaPC2d_BSplineCurve(theCurve)`); a null
    /// curve keeps the domain `[0, 1]`; `BSplineCurve::new(curve)`.
    pub fn new(curve: Handle<Geom2dBSplineCurve>) -> Self {
        let domain: Domain1D = match curve.get() {
            Some(bspline) => Domain1D::new(bspline.first_parameter(), bspline.last_parameter()),
            None => Domain1D::new(0.0, 1.0),
        };
        Self::with_domain(curve, domain)
    }
    /// Evaluator over `domain` (`ExtremaPC2d_BSplineCurve(theCurve, theDomain)`);
    /// `BSplineCurve::with_domain(curve, Domain1D::new(0.0, 0.5))`.
    pub fn with_domain(curve: Handle<Geom2dBSplineCurve>, domain: Domain1D) -> Self {
        Self {
            shape: BSplineShape(curve),
            domain,
            evaluator: GridEvaluator::default(),
        }
    }
    /// The curve point at `u` (`Value`); `extrema.value(0.5)?`.
    pub fn value(&self, u: f64) -> Result<Point2, Failure> {
        self.shape.evaluate(u)
    }
    /// Always true: a BSpline curve is bounded (`IsBounded`); `extrema.is_bounded()`.
    pub fn is_bounded(&self) -> bool {
        true
    }
    /// The domain (`Domain`); `extrema.domain()`.
    pub fn domain(&self) -> &Domain1D {
        &self.domain
    }
    /// The curve (`Curve`); `extrema.curve()`.
    pub fn curve(&self) -> &Handle<Geom2dBSplineCurve> {
        &self.shape.0
    }
    /// The interior extrema (`Perform`); `extrema.perform(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform(
        &mut self,
        point: Point2,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        if self.shape.0.is_null() {
            let result: &mut ExtremaResult = self.evaluator.result();
            result.clear();
            result.status = Status::NotDone;
            return Ok(result);
        }
        self.build_params()?;
        let adaptor = Geom2dAdaptorCurve::with_curve(&self.shape.0.cast())?;
        Ok(self
            .evaluator
            .perform(&adaptor, &point, &self.domain, tol, mode))
    }
    /// The interior extrema and the extremal ends (`PerformWithEndpoints`);
    /// `extrema.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point2,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        self.perform(point, tol, mode)?;
        let status: Status = self.evaluator.result().status;
        if status == Status::OK || status == Status::NoSolution {
            let adaptor = Geom2dAdaptorCurve::with_curve(&self.shape.0.cast())?;
            let is_closed: bool = GridEvaluator::is_closed_domain(&adaptor, &self.domain);
            let result: &mut ExtremaResult = self.evaluator.result();
            add_endpoint_extrema(result, &point, &self.domain, &self.shape, mode, is_closed)?;
            if !result.extrema.is_empty() {
                result.status = Status::OK;
            }
        }
        Ok(self.evaluator.result())
    }
    // `buildParams`: the knot-aware partition of a valid finite domain.
    fn build_params(&mut self) -> Result<(), Failure> {
        if self.shape.0.is_null() || !self.domain.is_valid() || !self.domain.is_finite(1.0e100) {
            return Ok(());
        }
        let params: Vector<'static> = self.build_knot_aware_params()?;
        self.evaluator.set_params(&params);
        Ok(())
    }
    // `buildKnotAwareParams`.
    fn build_knot_aware_params(&self) -> Result<Vector<'static>, Failure> {
        let Some(bspline) = self.shape.0.get() else {
            // Fallback to uniform sampling
            return GridEvaluator::build_uniform_params(
                self.domain.min,
                self.domain.max,
                THE_BSPLINE_FALLBACK_SAMPLES,
            );
        };
        let grid = KnotGrid {
            knots: bspline.knots().iter().copied().collect(),
            degree: bspline.degree(),
            period: if bspline.is_periodic() {
                Some(bspline.period()?)
            } else {
                None
            },
            class_name: "ExtremaPC2d_BSplineCurve",
        };
        knot_aware_params(&grid, &self.domain)
    }
}

impl CurveEvaluator<Point2> for BSplineCurve {
    type Error = Failure;
    fn evaluate(&self, u: f64) -> Result<Point2, Failure> {
        self.value(u)
    }
}
