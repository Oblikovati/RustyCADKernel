// Rust translation of ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_BezierCurve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC2d_BezierCurve`: numerical extrema on a Bezier curve over a uniform
//! partition sized from its degree.
use super::{
    CurveEvaluator, Domain1D, ExtremaResult, GridEvaluator, SearchMode, Status,
    THE_BEZIER_DEGREE_MULTIPLIER, THE_BEZIER_MIN_SAMPLES, add_endpoint_extrema,
};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::Vector;

/// Point-Bezier extrema (`ExtremaPC2d_BezierCurve`).
///
/// The partition (max(24, 3 (degree + 1)) uniform samples) is rebuilt on every query,
/// so changes to the shared curve are observed; a null curve gives NotDone;
/// `BezierCurve::new(curve).perform(point, 1e-9, SearchMode::MinMax)?`.
pub struct BezierCurve {
    shape: BezierShape,
    domain: Domain1D,
    nb_samples: usize,
    evaluator: GridEvaluator,
}

/// The curve, evaluated by AddEndpointExtrema apart from the evaluator's result.
struct BezierShape(Handle<Geom2dBezierCurve>);

impl CurveEvaluator<Point2> for BezierShape {
    type Error = Failure;
    fn evaluate(&self, u: f64) -> Result<Point2, Failure> {
        spline_value(&self.0, u)
    }
}

/// `myCurve->Value(theU)`; natively a null curve is dereferenced, here it is
/// Standard_NullObject.
pub(crate) fn spline_value<T: Geom2dCurve + RuntimeTyped>(
    curve: &Handle<T>,
    u: f64,
) -> Result<Point2, Failure> {
    let curve = curve.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "ExtremaPC2d: Value on a null curve; expected a curve",
            "",
        )
    })?;
    curve.value(u)
}

/// `max(THE_BEZIER_MIN_SAMPLES, THE_BEZIER_DEGREE_MULTIPLIER * (degree + 1))`.
pub(crate) fn bezier_sample_count(degree: i32) -> usize {
    THE_BEZIER_MIN_SAMPLES.max(THE_BEZIER_DEGREE_MULTIPLIER * (degree + 1) as usize)
}

impl BezierCurve {
    /// Evaluator over the curve bounds (`ExtremaPC2d_BezierCurve(theCurve)`); a null
    /// curve keeps the domain `[0, 1]`; `BezierCurve::new(curve)`.
    pub fn new(curve: Handle<Geom2dBezierCurve>) -> Self {
        let domain: Domain1D = match curve.get() {
            Some(bezier) => Domain1D::new(bezier.first_parameter(), bezier.last_parameter()),
            None => Domain1D::new(0.0, 1.0),
        };
        Self::with_domain(curve, domain)
    }
    /// Evaluator over `domain` (`ExtremaPC2d_BezierCurve(theCurve, theDomain)`);
    /// `BezierCurve::with_domain(curve, Domain1D::new(0.0, 0.5))`.
    pub fn with_domain(curve: Handle<Geom2dBezierCurve>, domain: Domain1D) -> Self {
        let nb_samples: usize = match curve.get() {
            Some(bezier) => bezier_sample_count(bezier.degree()),
            None => THE_BEZIER_MIN_SAMPLES,
        };
        Self {
            shape: BezierShape(curve),
            domain,
            nb_samples,
            evaluator: GridEvaluator::default(),
        }
    }
    /// The curve point at `u` (`Value`); `extrema.value(0.5)?`.
    pub fn value(&self, u: f64) -> Result<Point2, Failure> {
        self.shape.evaluate(u)
    }
    /// Always true: a Bezier curve is bounded (`IsBounded`); `extrema.is_bounded()`.
    pub fn is_bounded(&self) -> bool {
        true
    }
    /// The domain (`Domain`); `extrema.domain()`.
    pub fn domain(&self) -> &Domain1D {
        &self.domain
    }
    /// The curve (`Curve`); `extrema.curve()`.
    pub fn curve(&self) -> &Handle<Geom2dBezierCurve> {
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
    // `buildParams`: the uniform partition of a valid finite domain.
    fn build_params(&mut self) -> Result<(), Failure> {
        let Some(bezier) = self.shape.0.get() else {
            return Ok(());
        };
        if !self.domain.is_valid() || !self.domain.is_finite(1.0e100) {
            return Ok(());
        }
        self.nb_samples = bezier_sample_count(bezier.degree());
        let params: Vector<'static> =
            GridEvaluator::build_uniform_params(self.domain.min, self.domain.max, self.nb_samples)?;
        self.evaluator.set_params(&params);
        Ok(())
    }
}

impl CurveEvaluator<Point2> for BezierCurve {
    type Error = Failure;
    fn evaluate(&self, u: f64) -> Result<Point2, Failure> {
        self.value(u)
    }
}
