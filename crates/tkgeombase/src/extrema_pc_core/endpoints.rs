// Rust translation of AddEndpointExtrema of ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d.hxx
// and ExtremaPC/ExtremaPC.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AddEndpointExtrema`: the finite ends of a bounded curve are local extrema of the
//! distance when a neighboring point is farther (minimum) or nearer (maximum). The
//! 2D and 3D templates differ only in the neighbor step, which each package passes.
use super::result::{PcExtremum, PcResult, SearchMode};
use super::space::PcPoint;
use super::validity::{THE_PARAM_TOLERANCE, is_finite_parameter};
use rustycad_tkmath::math_utils::Domain1D;

/// A curve evaluated by `AddEndpointExtrema` (the `CurveEvaluator` template parameter:
/// a type with `Value(double)`). Every evaluator of the package implements it; an
/// infallible evaluation uses `std::convert::Infallible` as its error;
/// `impl CurveEvaluator<Point2> for Linear { type Error = Infallible; ... }`.
pub trait CurveEvaluator<P> {
    /// The failure of an evaluation (a native exception), or `Infallible`.
    type Error;
    /// The curve point at `u` (`Value(theU)`); `evaluator.evaluate(0.5)?`.
    fn evaluate(&self, u: f64) -> Result<P, Self::Error>;
}

/// The neighbor step of one endpoint: `(parameter, span, has_finite_span) -> step`.
pub(crate) type NeighborStep = fn(f64, f64, bool) -> f64;

/// The duplicate test of the template: an existing extremum within THE_PARAM_TOLERANCE.
fn is_duplicate<P>(result: &PcResult<P>, u: f64) -> bool {
    result
        .extrema
        .iter()
        .any(|extremum| (extremum.parameter - u).abs() < THE_PARAM_TOLERANCE)
}

/// The shared body of both `AddEndpointExtrema` templates.
pub(crate) fn add_endpoint_extrema_with<P: PcPoint, E: CurveEvaluator<P> + ?Sized>(
    result: &mut PcResult<P>,
    query: &P,
    domain: &Domain1D,
    evaluator: &E,
    (mode, is_closed): (SearchMode, bool),
    step: NeighborStep,
) -> Result<(), E::Error> {
    if !domain.is_valid() {
        return Ok(());
    }
    let (u_min, u_max) = (domain.min, domain.max);
    let has_finite_min: bool = is_finite_parameter(u_min);
    let has_finite_max: bool = is_finite_parameter(u_max);
    if !has_finite_min && !has_finite_max {
        return Ok(());
    }
    if u_min == u_max {
        if has_finite_min && !is_duplicate(result, u_min) {
            let point: P = evaluator.evaluate(u_min)?;
            let square_distance: f64 = query.square_distance_to(&point);
            let extremum = PcExtremum::new(
                u_min,
                point,
                square_distance,
                mode != SearchMode::Max,
                mode != SearchMode::Min,
            );
            result.extrema.push(extremum);
        }
        return Ok(());
    }
    if is_closed {
        return Ok(());
    }
    let endpoint = Endpoint {
        query,
        evaluator,
        mode,
        span: (u_max - u_min, has_finite_min && has_finite_max),
        step,
    };
    if has_finite_min {
        endpoint.add(result, u_min, true)?;
    }
    if has_finite_max {
        endpoint.add(result, u_max, false)?;
    }
    Ok(())
}

/// The `addEndpoint` lambda and what it captures.
struct Endpoint<'a, P, E: ?Sized> {
    query: &'a P,
    evaluator: &'a E,
    mode: SearchMode,
    span: (f64, bool),
    step: NeighborStep,
}

impl<P: PcPoint, E: CurveEvaluator<P> + ?Sized> Endpoint<'_, P, E> {
    // Classifies one end against its inner neighbor and appends it in the search mode.
    fn add(
        &self,
        result: &mut PcResult<P>,
        parameter: f64,
        is_lower: bool,
    ) -> Result<(), E::Error> {
        if is_duplicate(result, parameter) {
            return Ok(());
        }
        let step: f64 = (self.step)(parameter, self.span.0, self.span.1);
        let point: P = self.evaluator.evaluate(parameter)?;
        let neighbor: P = self
            .evaluator
            .evaluate(parameter + if is_lower { step } else { -step })?;
        let square_distance: f64 = self.query.square_distance_to(&point);
        let neighbor_distance: f64 = self.query.square_distance_to(&neighbor);
        let is_minimum: bool = square_distance <= neighbor_distance;
        let is_maximum: bool = square_distance >= neighbor_distance;
        let wants_min: bool = self.mode == SearchMode::Min || self.mode == SearchMode::MinMax;
        let wants_max: bool = self.mode == SearchMode::Max || self.mode == SearchMode::MinMax;
        if wants_min && is_minimum {
            let extremum = PcExtremum::new(parameter, point, square_distance, true, false);
            result.extrema.push(extremum);
        } else if wants_max && is_maximum {
            let extremum = PcExtremum::new(parameter, point, square_distance, false, true);
            result.extrema.push(extremum);
        }
        Ok(())
    }
}

/// A closure as a [`CurveEvaluator`], for the evaluators that pass their geometry
/// apart from their own result; `ValueFn(|u| Ok::<_, Infallible>(line.value(u)))`.
pub(crate) struct ValueFn<F>(pub F);

impl<P, E, F: Fn(f64) -> Result<P, E>> CurveEvaluator<P> for ValueFn<F> {
    type Error = E;
    fn evaluate(&self, u: f64) -> Result<P, E> {
        (self.0)(u)
    }
}
