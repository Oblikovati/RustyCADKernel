use super::PcurveGap;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkgeomalgo::geom2d_api::{ExtremaCurveCurve, ProjectPointOnCurve};
use rustycad_tkmath::gp::Point2;

pub(super) struct PairParameters {
    pub(super) point: Point2,
    pub(super) second_point: Point2,
    pub(super) first: f64,
    pub(super) second: f64,
}

pub(super) fn find(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_domain: (f64, f64),
    second_domain: (f64, f64),
) -> Result<Option<PairParameters>, Failure> {
    if let Some(candidate) = accepted_extrema(
        first,
        second,
        first_curve,
        second_curve,
        first_domain,
        second_domain,
    )? {
        return Ok(Some(candidate));
    }
    projection_candidate(
        first,
        second,
        first_curve,
        second_curve,
        first_domain,
        second_domain,
    )
}

fn accepted_extrema(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_domain: (f64, f64),
    second_domain: (f64, f64),
) -> Result<Option<PairParameters>, Failure> {
    let Some(candidate) = extrema_candidate(
        first,
        second,
        first_curve,
        second_curve,
        first_domain,
        second_domain,
    )?
    else {
        return Ok(None);
    };
    retain_if_acceptable(first, second, first_curve, second_curve, candidate)
}

fn retain_if_acceptable(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    candidate: PairParameters,
) -> Result<Option<PairParameters>, Failure> {
    let acceptable = accept_pair(
        first,
        second,
        first_curve,
        second_curve,
        candidate.first,
        candidate.second,
    )?;
    Ok(acceptable.then_some(candidate))
}

fn extrema_candidate(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_domain: (f64, f64),
    second_domain: (f64, f64),
) -> Result<Option<PairParameters>, Failure> {
    let extrema = curve_extrema(first_curve, second_curve, (first_domain, second_domain))?;
    let Some(parameters) = extrema_parameters(&extrema)? else {
        return Ok(None);
    };
    let parameters = adjust_pair(first, second, first_curve, second_curve, parameters)?;
    candidate_from_parameters(first_curve, second_curve, parameters)
}

fn curve_extrema(
    first: &Handle<dyn Geom2dCurve>,
    second: &Handle<dyn Geom2dCurve>,
    domains: ((f64, f64), (f64, f64)),
) -> Result<ExtremaCurveCurve, Failure> {
    ExtremaCurveCurve::new(
        first,
        second,
        domains.0.0,
        domains.0.1,
        domains.1.0,
        domains.1.1,
    )
}

fn extrema_parameters(extrema: &ExtremaCurveCurve) -> Result<Option<(f64, f64)>, Failure> {
    if extrema.nb_extrema() == 0 {
        return Ok(None);
    }
    Ok(Some(extrema.lower_distance_parameters()?))
}

fn projection_candidate(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_domain: (f64, f64),
    second_domain: (f64, f64),
) -> Result<Option<PairParameters>, Failure> {
    let pair = project_and_select_endpoints(
        first,
        second,
        first_curve,
        second_curve,
        first_domain,
        second_domain,
    )?;
    let parameters = adjust_pair(
        first,
        second,
        first_curve,
        second_curve,
        (pair.first, pair.second),
    )?;
    accepted_projection(first, second, first_curve, second_curve, parameters)
}

fn accepted_projection(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    parameters: (f64, f64),
) -> Result<Option<PairParameters>, Failure> {
    if !accept_pair(
        first,
        second,
        first_curve,
        second_curve,
        parameters.0,
        parameters.1,
    )? {
        return Ok(None);
    }
    candidate_from_parameters(first_curve, second_curve, parameters)
}

fn project_and_select_endpoints(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_domain: (f64, f64),
    second_domain: (f64, f64),
) -> Result<ProjectionPair, Failure> {
    let mut pair = project_endpoints(
        first,
        second,
        first_curve,
        second_curve,
        first_domain,
        second_domain,
    )?;
    adjust_initial_pair(first, second, first_curve, second_curve, &mut pair)?;
    select_projection_pair(
        first,
        second,
        first_curve,
        second_curve,
        pair,
        (first_domain, second_domain),
    )
}

fn select_projection_pair(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    mut pair: ProjectionPair,
    domains: ((f64, f64), (f64, f64)),
) -> Result<ProjectionPair, Failure> {
    if projections_cross_original_ends(first, second, pair.first, pair.second) {
        return project_midpoint(first, second, first_curve, second_curve, domains);
    }
    retain_nearer_endpoint(first, second, &mut pair);
    Ok(pair)
}

#[derive(Clone, Copy)]
struct ProjectionPair {
    first_point: Point2,
    second_point: Point2,
    first: f64,
    second: f64,
}

fn project_endpoints(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_domain: (f64, f64),
    second_domain: (f64, f64),
) -> Result<ProjectionPair, Failure> {
    project_pair(
        first.point,
        second.point,
        first,
        second,
        first_curve,
        second_curve,
        (first_domain, second_domain),
    )
}

fn project_pair(
    first_point: Point2,
    second_point: Point2,
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    domains: ((f64, f64), (f64, f64)),
) -> Result<ProjectionPair, Failure> {
    let mut pair = initial_projection_pair(first_point, second_point, first, second);
    project_endpoints_into_pair(
        &mut pair,
        (first, second),
        (first_curve, second_curve),
        domains,
    )?;
    Ok(pair)
}

fn initial_projection_pair(
    first_point: Point2,
    second_point: Point2,
    first: &PcurveGap,
    second: &PcurveGap,
) -> ProjectionPair {
    ProjectionPair {
        first_point,
        second_point,
        first: native_end_parameter(first),
        second: native_start_parameter(second),
    }
}

fn project_endpoints_into_pair(
    pair: &mut ProjectionPair,
    entries: (&PcurveGap, &PcurveGap),
    curves: (&Handle<dyn Geom2dCurve>, &Handle<dyn Geom2dCurve>),
    domains: ((f64, f64), (f64, f64)),
) -> Result<(), Failure> {
    project_point(
        entries.1.point,
        curves.0,
        domains.0,
        &mut pair.first_point,
        &mut pair.first,
    )?;
    project_point(
        entries.0.point,
        curves.1,
        domains.1,
        &mut pair.second_point,
        &mut pair.second,
    )
}

fn project_point(
    point: Point2,
    curve: &Handle<dyn Geom2dCurve>,
    domain: (f64, f64),
    projected: &mut Point2,
    parameter: &mut f64,
) -> Result<(), Failure> {
    let projection = ProjectPointOnCurve::with_range(point, curve, domain.0, domain.1)?;
    if projection.nb_points() == 0 {
        return Ok(());
    }
    *projected = projection.nearest_point()?;
    *parameter = projection.lower_distance_parameter()?;
    Ok(())
}

fn adjust_initial_pair(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    pair: &mut ProjectionPair,
) -> Result<(), Failure> {
    (pair.first, pair.second) = adjust_pair(
        first,
        second,
        first_curve,
        second_curve,
        (pair.first, pair.second),
    )?;
    Ok(())
}

fn projections_cross_original_ends(
    first: &PcurveGap,
    second: &PcurveGap,
    first_parameter: f64,
    second_parameter: f64,
) -> bool {
    let first_inside = on_gap_side(first, first_parameter, native_end_parameter(first), true);
    let second_inside = on_gap_side(
        second,
        second_parameter,
        native_start_parameter(second),
        false,
    );
    let first_outside = off_gap_side(first, first_parameter, native_end_parameter(first), true);
    let second_outside = off_gap_side(
        second,
        second_parameter,
        native_start_parameter(second),
        false,
    );
    first_inside && second_inside || first_outside && second_outside
}

fn on_gap_side(entry: &PcurveGap, parameter: f64, endpoint: f64, first_edge: bool) -> bool {
    if entry.reversed == first_edge {
        return parameter > endpoint;
    }
    parameter < endpoint
}

fn off_gap_side(entry: &PcurveGap, parameter: f64, endpoint: f64, first_edge: bool) -> bool {
    parameter != endpoint && !on_gap_side(entry, parameter, endpoint, first_edge)
}

fn project_midpoint(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    domains: ((f64, f64), (f64, f64)),
) -> Result<ProjectionPair, Failure> {
    let midpoint = midpoint(first.point, second.point);
    let mut pair = initial_projection_pair(first.point, second.point, first, second);
    project_point(
        midpoint,
        first_curve,
        domains.0,
        &mut pair.first_point,
        &mut pair.first,
    )?;
    project_point(
        midpoint,
        second_curve,
        domains.1,
        &mut pair.second_point,
        &mut pair.second,
    )?;
    Ok(pair)
}

fn midpoint(first: Point2, second: Point2) -> Point2 {
    Point2::new(
        (first.x() + second.x()) * 0.5,
        (first.y() + second.y()) * 0.5,
    )
}

fn retain_nearer_endpoint(first: &PcurveGap, second: &PcurveGap, pair: &mut ProjectionPair) {
    if first.point.distance(pair.second_point) < second.point.distance(pair.first_point) {
        pair.first = native_end_parameter(first);
    } else {
        pair.second = native_start_parameter(second);
    }
}

pub(super) fn adjust_pair(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    parameters: (f64, f64),
) -> Result<(f64, f64), Failure> {
    Ok((
        adjust_periodic(
            first_curve,
            first.reversed,
            (first.first, first.last),
            parameters.0,
        )?,
        adjust_periodic(
            second_curve,
            !second.reversed,
            (second.first, second.last),
            parameters.1,
        )?,
    ))
}

pub(super) fn adjust_periodic(
    curve: &Handle<dyn Geom2dCurve>,
    take_first: bool,
    bounds: (f64, f64),
    parameter: f64,
) -> Result<f64, Failure> {
    let Some(curve) = curve.get() else {
        return Ok(parameter);
    };
    if !curve.is_periodic() {
        return Ok(parameter);
    }
    let period = curve.period()?;
    let shift = periodic_shift(bounds.0, period);
    let (first, last) = (bounds.0 + shift, bounds.1 + shift);
    if take_first && parameter > last && parameter > first {
        return Ok(parameter - period - shift);
    }
    if !take_first && parameter < last && parameter < first {
        return Ok(parameter + period - shift);
    }
    Ok(parameter)
}

fn periodic_shift(first: f64, period: f64) -> f64 {
    let mut shift = -(first / period).trunc() * period;
    if first < 0.0 {
        shift += period;
    }
    shift
}

fn accept_pair(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_parameter: f64,
    second_parameter: f64,
) -> Result<bool, Failure> {
    let points = (
        first_curve.value(first_parameter)?,
        second_curve.value(second_parameter)?,
    );
    let limits = (first.point.distance(second.point), precision::p_confusion());
    Ok(close_enough(
        first,
        second,
        points,
        limits,
        (first_parameter, second_parameter),
    ))
}

fn close_enough(
    first: &PcurveGap,
    second: &PcurveGap,
    points: (Point2, Point2),
    limits: (f64, f64),
    parameters: (f64, f64),
) -> bool {
    let moved = (native_start_parameter(first) - parameters.0).abs() > limits.1
        && (native_end_parameter(second) - parameters.1).abs() > limits.1;
    let interior = strictly_inside((first.first, first.last), parameters.0)
        || strictly_inside((second.first, second.last), parameters.1);
    points.0.distance(points.1) <= limits.0
        && moved
        && (interior
            || first.point.distance(points.0) <= limits.0
            || second.point.distance(points.1) <= limits.0)
}

fn strictly_inside(bounds: (f64, f64), parameter: f64) -> bool {
    parameter > bounds.0 && parameter < bounds.1
}

fn candidate_from_parameters(
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    parameters: (f64, f64),
) -> Result<Option<PairParameters>, Failure> {
    Ok(Some(PairParameters {
        point: first_curve.value(parameters.0)?,
        second_point: second_curve.value(parameters.1)?,
        first: parameters.0,
        second: parameters.1,
    }))
}

fn native_start_parameter(entry: &PcurveGap) -> f64 {
    if entry.reversed {
        entry.last
    } else {
        entry.first
    }
}

fn native_end_parameter(entry: &PcurveGap) -> f64 {
    if entry.reversed {
        entry.first
    } else {
        entry.last
    }
}
