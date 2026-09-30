// Rust translation of the PerformWithEndpoints members of the Line, Circle, Ellipse,
// Hyperbola and Parabola evaluators of ModelingData/TKGeomBase/ExtremaPC2d and
// ModelingData/TKGeomBase/ExtremaPC at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `PerformWithEndpoints` flow the ten analytical evaluators share: validate,
//! handle a degenerate domain by its single point, compute the interior extrema, then
//! add the ends of a bounded domain.
use super::endpoints::{CurveEvaluator, NeighborStep, add_endpoint_extrema_with};
use super::result::{PcResult, SearchMode, Status};
use super::space::PcPoint;
use super::validity::{is_closed_periodic_domain, is_valid_tolerance};
use rustycad_tkernel::precision;
use rustycad_tkmath::math_utils::Domain1D;
use std::convert::Infallible;
use std::f64::consts::PI;

/// An analytical evaluator: an infallible curve with an optional domain.
pub(crate) trait AnalyticEvaluator<P: PcPoint>:
    CurveEvaluator<P, Error = Infallible>
{
    /// Whether a domain spanning whole periods of 2*PI is closed (circle, ellipse).
    const IS_PERIODIC: bool;
    /// The neighbor step of the package's AddEndpointExtrema.
    const NEIGHBOR_STEP: NeighborStep;
    /// The failure of `Perform` (`Infallible` in 2D; a projection failure in 3D).
    type PerformError;
    /// The domain fixed at construction, `None` for the whole curve.
    fn fixed_domain(&self) -> Option<Domain1D>;
    /// The reusable result.
    fn result_slot(&mut self) -> &mut PcResult<P>;
    /// `Perform`: the interior extrema into the result.
    fn perform_interior(
        &mut self,
        point: P,
        tol: f64,
        mode: SearchMode,
    ) -> Result<(), Self::PerformError>;
}

/// The InvalidInput test of every evaluator: a bad tolerance, a non-finite point or an
/// invalid domain.
pub(crate) fn is_invalid_input<P: PcPoint>(tol: f64, point: &P, domain: Option<&Domain1D>) -> bool {
    !is_valid_tolerance(tol) || !point.is_finite_point() || domain.is_some_and(|d| !d.is_valid())
}

/// `myResult.Status = Extrema.IsEmpty() ? NoSolution : OK`.
pub(crate) fn found_status<P>(result: &PcResult<P>) -> Status {
    if result.extrema.is_empty() {
        Status::NoSolution
    } else {
        Status::OK
    }
}

/// The shared `PerformWithEndpoints`.
pub(crate) fn perform_with_endpoints<P: PcPoint, E: AnalyticEvaluator<P>>(
    evaluator: &mut E,
    point: P,
    tol: f64,
    mode: SearchMode,
) -> Result<&PcResult<P>, E::PerformError> {
    let domain: Option<Domain1D> = evaluator.fixed_domain();
    if is_invalid_input(tol, &point, domain.as_ref()) {
        let result: &mut PcResult<P> = evaluator.result_slot();
        result.clear();
        result.status = Status::InvalidInput;
        return Ok(result);
    }
    if let Some(domain) = domain.filter(|domain| domain.min == domain.max) {
        let mut result: PcResult<P> = std::mem::take(evaluator.result_slot());
        result.clear();
        add_ends(evaluator, &mut result, &point, &domain, (mode, false));
        result.status = found_status(&result);
        *evaluator.result_slot() = result;
        return Ok(evaluator.result_slot());
    }
    evaluator.perform_interior(point, tol, mode)?;
    let mut result: PcResult<P> = std::mem::take(evaluator.result_slot());
    if let Some(domain) = domain
        && (result.status == Status::OK || result.status == Status::NoSolution)
    {
        let is_closed: bool =
            E::IS_PERIODIC && is_closed_periodic_domain(&domain, 2.0 * PI, precision::angular());
        add_ends(evaluator, &mut result, &point, &domain, (mode, is_closed));
        if !result.extrema.is_empty() {
            result.status = Status::OK;
        }
    }
    *evaluator.result_slot() = result;
    Ok(evaluator.result_slot())
}

// `AddEndpointExtrema` on an infallible evaluator.
fn add_ends<P: PcPoint, E: AnalyticEvaluator<P>>(
    evaluator: &E,
    result: &mut PcResult<P>,
    point: &P,
    domain: &Domain1D,
    (mode, is_closed): (SearchMode, bool),
) {
    let Ok(()) = add_endpoint_extrema_with(
        result,
        point,
        domain,
        evaluator,
        (mode, is_closed),
        E::NEIGHBOR_STEP,
    );
}
