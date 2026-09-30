// Rust translation of ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_OffsetCurve.hxx/.cxx and
// ExtremaPC2d_OtherCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC2d_OffsetCurve` and `ExtremaPC2d_OtherCurve`: numerical extrema through an
//! adaptor over a curve-aware partition (C2 intervals refined by
//! `Extrema_Curve2dTool::DeflCurvIntervals`). The two native classes have the same
//! code; [`AdaptorExtrema`] holds it and each public type wraps it.
use super::{
    CurveEvaluator, Domain1D, ExtremaResult, GridEvaluator, SearchMode, Status,
    add_endpoint_extrema,
};
use crate::extrema_pc_core::{PcCurveRef, Space2d};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::Vector;

/// The adaptor reference of the evaluators.
pub type CurveRef2d<'a> = PcCurveRef<'a, Space2d>;

/// The shared body of the offset and general-curve evaluators.
pub(crate) struct AdaptorExtrema<'a> {
    shape: AdaptorShape<'a>,
    domain: Domain1D,
    evaluator: GridEvaluator,
}

/// The adaptor, evaluated by AddEndpointExtrema apart from the evaluator's result.
struct AdaptorShape<'a>(CurveRef2d<'a>);

impl CurveEvaluator<Point2> for AdaptorShape<'_> {
    type Error = Failure;
    fn evaluate(&self, u: f64) -> Result<Point2, Failure> {
        let curve = self.0.get().ok_or_else(null_adaptor)?;
        curve.value(u)
    }
}

// Natively a null adaptor pointer is dereferenced.
fn null_adaptor() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ExtremaPC2d: Value on a null adaptor; expected a curve",
        "",
    )
}

impl<'a> AdaptorExtrema<'a> {
    fn new(curve: CurveRef2d<'a>, domain: Domain1D) -> Self {
        Self {
            shape: AdaptorShape(curve),
            domain,
            evaluator: GridEvaluator::default(),
        }
    }
    // The constructor without a domain: the adaptor bounds.
    fn over_bounds(curve: CurveRef2d<'a>) -> Result<Self, Failure> {
        let adaptor = curve.get().ok_or_else(null_adaptor)?;
        let domain = Domain1D::new(adaptor.first_parameter()?, adaptor.last_parameter()?);
        Ok(Self::new(curve, domain))
    }
    fn perform(
        &mut self,
        point: Point2,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        let Some(curve) = self.shape.0.get() else {
            let result: &mut ExtremaResult = self.evaluator.result();
            result.clear();
            result.status = Status::NotDone;
            return Ok(result);
        };
        // buildParams: the curve-aware partition of a valid finite domain.
        if self.domain.is_valid() && self.domain.is_finite(1.0e100) {
            let params: Vector<'static> =
                GridEvaluator::build_curve_aware_params(curve, &self.domain)?;
            self.evaluator.set_params(&params);
        }
        Ok(self
            .evaluator
            .perform(curve, &point, &self.domain, tol, mode))
    }
    fn perform_with_endpoints(
        &mut self,
        point: Point2,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        // Refresh the curve-aware partition and compute interior extrema.
        self.perform(point, tol, mode)?;
        let Some(curve) = self.shape.0.get() else {
            return Ok(self.evaluator.result());
        };
        let status: Status = self.evaluator.result().status;
        if status == Status::OK || status == Status::NoSolution {
            let is_closed: bool = GridEvaluator::is_closed_domain(curve, &self.domain);
            let result: &mut ExtremaResult = self.evaluator.result();
            add_endpoint_extrema(result, &point, &self.domain, &self.shape, mode, is_closed)?;
            if !result.extrema.is_empty() {
                result.status = Status::OK;
            }
        }
        Ok(self.evaluator.result())
    }
}

// The public offset and general-curve evaluators: the same members around
// AdaptorExtrema, with the native class names.
macro_rules! adaptor_evaluator {
    ($(#[$doc:meta])* $name:ident, $native:literal) => {
        $(#[$doc])*
        pub struct $name<'a>(AdaptorExtrema<'a>);

        impl<'a> $name<'a> {
            #[doc = concat!("Evaluator over the adaptor bounds (`", $native, "(theCurve)`); ")]
            #[doc = "the adaptor must outlive it; `new(&adaptor)?`."]
            pub fn new(curve: &'a dyn Adaptor2dCurve2d) -> Result<Self, Failure> {
                AdaptorExtrema::over_bounds(PcCurveRef::Borrowed(curve)).map(Self)
            }
            #[doc = concat!("Evaluator over `domain` (`", $native, "(theCurve, theDomain)`); ")]
            #[doc = "`with_domain(&adaptor, Domain1D::new(0.0, 1.0))`."]
            pub fn with_domain(curve: &'a dyn Adaptor2dCurve2d, domain: Domain1D) -> Self {
                Self(AdaptorExtrema::new(PcCurveRef::Borrowed(curve), domain))
            }
            /// Evaluator over a borrowed or shared adaptor, with the adaptor bounds when
            /// `domain` is `None`; `from_curve_ref(PcCurveRef::Shared(handle), None)?`.
            pub fn from_curve_ref(curve: CurveRef2d<'a>, domain: Option<Domain1D>) -> Result<Self, Failure> {
                match domain {
                    Some(domain) => Ok(Self(AdaptorExtrema::new(curve, domain))),
                    None => AdaptorExtrema::over_bounds(curve).map(Self),
                }
            }
            /// The adaptor point at `u` (`Value`); `extrema.value(0.5)?`.
            pub fn value(&self, u: f64) -> Result<Point2, Failure> {
                self.0.shape.evaluate(u)
            }
            /// True for a finite domain (`IsBounded`); `extrema.is_bounded()`.
            pub fn is_bounded(&self) -> bool {
                self.0.domain.is_finite(1.0e100)
            }
            /// The domain (`Domain`); `extrema.domain()`.
            pub fn domain(&self) -> &Domain1D {
                &self.0.domain
            }
            /// The interior extrema; InvalidInput for an infinite domain, NotDone
            /// without an adaptor (`Perform`); `extrema.perform(point, 1e-9, SearchMode::MinMax)?`.
            pub fn perform(&mut self, point: Point2, tol: f64, mode: SearchMode) -> Result<&ExtremaResult, Failure> {
                self.0.perform(point, tol, mode)
            }
            /// The interior extrema and the extremal ends (`PerformWithEndpoints`);
            /// `extrema.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)?`.
            pub fn perform_with_endpoints(&mut self, point: Point2, tol: f64, mode: SearchMode) -> Result<&ExtremaResult, Failure> {
                self.0.perform_with_endpoints(point, tol, mode)
            }
        }

        impl CurveEvaluator<Point2> for $name<'_> {
            type Error = Failure;
            fn evaluate(&self, u: f64) -> Result<Point2, Failure> {
                self.value(u)
            }
        }
    };
}

adaptor_evaluator!(
    /// Point-offset-curve extrema through an adaptor (`ExtremaPC2d_OffsetCurve`).
    OffsetCurve,
    "ExtremaPC2d_OffsetCurve"
);
adaptor_evaluator!(
    /// Point-curve extrema through any adaptor (`ExtremaPC2d_OtherCurve`).
    OtherCurve,
    "ExtremaPC2d_OtherCurve"
);
