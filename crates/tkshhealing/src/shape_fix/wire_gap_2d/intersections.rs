use super::fallback::{native_end_parameter, native_start_parameter};
use super::{PcurveGap, domains, fallback};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::{geom2d::Geom2dCurve, geom2d_adaptor::Geom2dAdaptorCurve};
use rustycad_tkgeomalgo::{geom2d_int::GInter, int_res2d::Domain};
use rustycad_tkmath::gp::Point2;

#[derive(Clone, Copy)]
pub(super) struct LineIntersection {
    pub(super) point: Point2,
    pub(super) second_point: Point2,
    pub(super) first_parameter: f64,
    pub(super) second_parameter: f64,
}

impl From<fallback::PairParameters> for LineIntersection {
    fn from(candidate: fallback::PairParameters) -> Self {
        Self {
            point: candidate.point,
            second_point: candidate.second_point,
            first_parameter: candidate.first,
            second_parameter: candidate.second,
        }
    }
}

pub(super) fn curve_intersection(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
) -> Result<Option<LineIntersection>, Failure> {
    let bounds = domains::for_pair(first, second, first_curve, second_curve);
    let intersections = find_intersections(first_curve, second_curve, bounds)?;
    closest_line_intersection(first, second, first_curve, second_curve, &intersections)
}

fn find_intersections(
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    bounds: domains::SearchDomains,
) -> Result<GInter, Failure> {
    let first_domain = make_domain(first_curve, bounds.first)?;
    let second_domain = make_domain(second_curve, bounds.second)?;
    let first_adaptor = Geom2dAdaptorCurve::with_curve(first_curve)?;
    let second_adaptor = Geom2dAdaptorCurve::with_curve(second_curve)?;
    let intersections = GInter::with_domains(
        &first_adaptor,
        &first_domain,
        &second_adaptor,
        &second_domain,
        precision::p_confusion(),
        precision::p_confusion(),
    )?;
    Ok(intersections)
}

fn make_domain(curve: &Handle<dyn Geom2dCurve>, bounds: (f64, f64)) -> Result<Domain, Failure> {
    Ok(Domain::bounded(
        curve.value(bounds.0)?,
        bounds.0,
        precision::p_confusion(),
        curve.value(bounds.1)?,
        bounds.1,
        precision::p_confusion(),
    ))
}

fn closest_line_intersection(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    intersections: &GInter,
) -> Result<Option<LineIntersection>, Failure> {
    let candidates = normalize_candidates(
        first,
        second,
        first_curve,
        second_curve,
        intersection_candidates(intersections)?,
    )?;
    let (first_choice, second_choice) = endpoint_choices(first, second, candidates);
    Ok(select_closest_to_mean(
        first,
        second,
        first_choice,
        second_choice,
    ))
}

fn endpoint_choices(
    first: &PcurveGap,
    second: &PcurveGap,
    candidates: Vec<LineIntersection>,
) -> (Option<LineIntersection>, Option<LineIntersection>) {
    let mut choices = (None, None);
    for candidate in candidates {
        choose_endpoint_candidate(
            &mut choices.0,
            candidate,
            native_start_parameter(first),
            false,
        );
        choose_endpoint_candidate(
            &mut choices.1,
            candidate,
            native_end_parameter(second),
            true,
        );
    }
    choices
}

fn normalize_candidates(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    candidates: Vec<LineIntersection>,
) -> Result<Vec<LineIntersection>, Failure> {
    let mut normalized = Vec::with_capacity(candidates.len());
    for mut candidate in candidates {
        let parameters = fallback::adjust_pair(
            first,
            second,
            first_curve,
            second_curve,
            (candidate.first_parameter, candidate.second_parameter),
        )?;
        candidate.first_parameter = parameters.0;
        candidate.second_parameter = parameters.1;
        normalized.push(candidate);
    }
    Ok(normalized)
}

fn intersection_candidates(intersections: &GInter) -> Result<Vec<LineIntersection>, Failure> {
    let mut candidates = Vec::new();
    append_intersection_points(intersections, &mut candidates)?;
    append_intersection_segments(intersections, &mut candidates)?;
    Ok(candidates)
}

fn append_intersection_points(
    intersections: &GInter,
    candidates: &mut Vec<LineIntersection>,
) -> Result<(), Failure> {
    for index in 1..=intersections.nb_points()? {
        let point = intersections.point(index)?;
        candidates.push(LineIntersection {
            point: point.value(),
            second_point: point.value(),
            first_parameter: point.param_on_first(),
            second_parameter: point.param_on_second(),
        });
    }
    Ok(())
}

fn append_intersection_segments(
    intersections: &GInter,
    candidates: &mut Vec<LineIntersection>,
) -> Result<(), Failure> {
    for index in 1..=intersections.nb_segments()? {
        let segment = intersections.segment(index)?;
        if segment.has_first_point() {
            candidates.push(from_intersection_point(segment.first_point()?));
        }
        if segment.has_last_point() {
            candidates.push(from_intersection_point(segment.last_point()?));
        }
    }
    Ok(())
}

fn from_intersection_point(
    point: rustycad_tkgeomalgo::int_res2d::IntersectionPoint,
) -> LineIntersection {
    LineIntersection {
        point: point.value(),
        second_point: point.value(),
        first_parameter: point.param_on_first(),
        second_parameter: point.param_on_second(),
    }
}

fn choose_endpoint_candidate(
    best: &mut Option<LineIntersection>,
    candidate: LineIntersection,
    parameter: f64,
    second_curve: bool,
) {
    let candidate_parameter = curve_parameter(&candidate, second_curve);
    let distance = (candidate_parameter - parameter).abs();
    if best
        .as_ref()
        .is_none_or(|current| distance < (curve_parameter(current, second_curve) - parameter).abs())
    {
        *best = Some(candidate);
    }
}

fn curve_parameter(candidate: &LineIntersection, second_curve: bool) -> f64 {
    if second_curve {
        return candidate.second_parameter;
    }
    candidate.first_parameter
}

fn select_closest_to_mean(
    first: &PcurveGap,
    second: &PcurveGap,
    first_choice: Option<LineIntersection>,
    second_choice: Option<LineIntersection>,
) -> Option<LineIntersection> {
    let first_choice = first_choice?;
    let second_choice = second_choice?;
    if first_choice.first_parameter == second_choice.first_parameter
        && first_choice.second_parameter == second_choice.second_parameter
    {
        return Some(first_choice);
    }
    let midpoint = Point2::new(
        (first.point.x() + second.point.x()) * 0.5,
        (first.point.y() + second.point.y()) * 0.5,
    );
    if second_choice.point.distance(midpoint) < first_choice.point.distance(midpoint) {
        return Some(second_choice);
    }
    Some(first_choice)
}

pub(super) fn acceptable(
    first: &PcurveGap,
    second: &PcurveGap,
    candidate: &LineIntersection,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
) -> bool {
    let gap = first.point.distance(second.point);
    candidate_points_within_gap(first_curve, second_curve, candidate, gap)
        && candidate_moves_both(first, second, candidate)
        && candidate_is_acceptable(first, second, candidate, gap)
}

fn candidate_points_within_gap(
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
    candidate: &LineIntersection,
    gap: f64,
) -> bool {
    let Ok(first) = first_curve.value(candidate.first_parameter) else {
        return false;
    };
    let Ok(second) = second_curve.value(candidate.second_parameter) else {
        return false;
    };
    first.distance(second) <= gap
}

fn candidate_moves_both(
    first: &PcurveGap,
    second: &PcurveGap,
    candidate: &LineIntersection,
) -> bool {
    let threshold = precision::p_confusion();
    (native_start_parameter(first) - candidate.first_parameter).abs() > threshold
        && (native_end_parameter(second) - candidate.second_parameter).abs() > threshold
}

fn candidate_is_acceptable(
    first: &PcurveGap,
    second: &PcurveGap,
    candidate: &LineIntersection,
    gap: f64,
) -> bool {
    parameter_inside(first, candidate.first_parameter)
        || parameter_inside(second, candidate.second_parameter)
        || first.point.distance(candidate.point) <= gap
        || second.point.distance(candidate.point) <= gap
}

fn parameter_inside(entry: &PcurveGap, parameter: f64) -> bool {
    parameter >= entry.first.min(entry.last) && parameter <= entry.first.max(entry.last)
}
