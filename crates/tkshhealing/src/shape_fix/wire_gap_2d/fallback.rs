use super::{PcurveGap, projection};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkgeomalgo::geom2d_api::ExtremaCurveCurve;
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
    let parameters = projection::project_and_select_parameters(
        first,
        second,
        first_curve,
        second_curve,
        first_domain,
        second_domain,
    )?;
    let parameters = adjust_pair(first, second, first_curve, second_curve, parameters)?;
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

pub(super) fn accept_pair(
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

pub(super) fn native_start_parameter(entry: &PcurveGap) -> f64 {
    if entry.reversed {
        entry.last
    } else {
        entry.first
    }
}

pub(super) fn native_end_parameter(entry: &PcurveGap) -> f64 {
    if entry.reversed {
        entry.first
    } else {
        entry.last
    }
}
