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
    let extrema = extrema_candidate(
        first,
        second,
        first_curve,
        second_curve,
        first_domain,
        second_domain,
    )?;
    if let Some(candidate) = extrema
        && acceptable(
            first,
            second,
            first_curve,
            second_curve,
            candidate.first,
            candidate.second,
        )?
    {
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

fn extrema_candidate(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_domain: (f64, f64),
    second_domain: (f64, f64),
) -> Result<Option<PairParameters>, Failure> {
    let extrema = ExtremaCurveCurve::new(
        first_curve,
        second_curve,
        first_domain.0,
        first_domain.1,
        second_domain.0,
        second_domain.1,
    )?;
    if extrema.nb_extrema() == 0 {
        return Ok(None);
    }
    let parameters = extrema.lower_distance_parameters()?;
    let parameters = adjust_pair(first, second, first_curve, second_curve, parameters)?;
    candidate_from_parameters(first_curve, second_curve, parameters)
}

fn projection_candidate(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_domain: (f64, f64),
    second_domain: (f64, f64),
) -> Result<Option<PairParameters>, Failure> {
    let mut pair = project_endpoints(
        first,
        second,
        first_curve,
        second_curve,
        first_domain,
        second_domain,
    )?;
    adjust_initial_pair(first, second, first_curve, second_curve, &mut pair)?;
    if projections_cross_original_ends(first, second, pair.first, pair.second) {
        pair = project_midpoint(
            first,
            second,
            first_curve,
            second_curve,
            first_domain,
            second_domain,
        )?;
    } else {
        retain_nearer_endpoint(first, second, &mut pair);
    }
    let parameters = adjust_pair(
        first,
        second,
        first_curve,
        second_curve,
        (pair.first, pair.second),
    )?;
    if !acceptable(
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
    let mut pair = ProjectionPair {
        first_point: first.point,
        second_point: second.point,
        first: native_end_parameter(first),
        second: native_start_parameter(second),
    };
    project_point(
        second.point,
        first_curve,
        first_domain,
        &mut pair.first_point,
        &mut pair.first,
    )?;
    project_point(
        first.point,
        second_curve,
        second_domain,
        &mut pair.second_point,
        &mut pair.second,
    )?;
    Ok(pair)
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
    let first_end = native_end_parameter(first);
    let second_start = native_start_parameter(second);
    let first_inside = if first.reversed {
        first_parameter > first_end
    } else {
        first_parameter < first_end
    };
    let second_inside = if second.reversed {
        second_parameter < second_start
    } else {
        second_parameter > second_start
    };
    let first_outside = if first.reversed {
        first_parameter < first_end
    } else {
        first_parameter > first_end
    };
    let second_outside = if second.reversed {
        second_parameter > second_start
    } else {
        second_parameter < second_start
    };
    first_inside && second_inside || first_outside && second_outside
}

fn project_midpoint(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_domain: (f64, f64),
    second_domain: (f64, f64),
) -> Result<ProjectionPair, Failure> {
    let midpoint = Point2::new(
        (first.point.x() + second.point.x()) * 0.5,
        (first.point.y() + second.point.y()) * 0.5,
    );
    let mut pair = ProjectionPair {
        first_point: first.point,
        second_point: second.point,
        first: native_end_parameter(first),
        second: native_start_parameter(second),
    };
    project_point(
        midpoint,
        first_curve,
        first_domain,
        &mut pair.first_point,
        &mut pair.first,
    )?;
    project_point(
        midpoint,
        second_curve,
        second_domain,
        &mut pair.second_point,
        &mut pair.second,
    )?;
    Ok(pair)
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

fn acceptable(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    first_parameter: f64,
    second_parameter: f64,
) -> Result<bool, Failure> {
    let point1 = first_curve.value(first_parameter)?;
    let point2 = second_curve.value(second_parameter)?;
    let gap = first.point.distance(second.point);
    let endpoints_moved = (native_start_parameter(first) - first_parameter).abs()
        > precision::p_confusion()
        && (native_end_parameter(second) - second_parameter).abs() > precision::p_confusion();
    let interior = strictly_inside((first.first, first.last), first_parameter)
        || strictly_inside((second.first, second.last), second_parameter);
    Ok(point1.distance(point2) <= gap
        && endpoints_moved
        && (interior
            || first.point.distance(point1) <= gap
            || second.point.distance(point2) <= gap))
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
