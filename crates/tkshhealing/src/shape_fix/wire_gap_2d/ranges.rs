use super::{
    Face, Failure, PcurveGap, ShapeFixWire, clipping, convert, domains, fallback, replace,
};
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkg2d::{
    geom2d::{Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dTrimmedCurve},
    geom2d_adaptor::Geom2dAdaptorCurve,
};
use rustycad_tkgeomalgo::{geom2d_int::GInter, int_res2d::Domain};
use rustycad_tkmath::gp::Point2;
use std::f64::consts::PI;

#[derive(Clone, Copy)]
struct LineIntersection {
    point: Point2,
    second_point: Point2,
    first_parameter: f64,
    second_parameter: f64,
}

struct RangeCurves {
    first: Handle<dyn Geom2dCurve>,
    second: Handle<dyn Geom2dCurve>,
}

impl RangeCurves {
    fn from_entries(first: &PcurveGap, second: &PcurveGap) -> Result<Self, Failure> {
        Ok(Self {
            first: convert::basic_gap_curve(&first.curve)?,
            second: convert::basic_gap_curve(&second.curve)?,
        })
    }
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

impl From<clipping::ClippedPair> for fallback::PairParameters {
    fn from(candidate: clipping::ClippedPair) -> Self {
        Self {
            point: candidate.first_point,
            second_point: candidate.second_point,
            first: candidate.first_parameter,
            second: candidate.second_parameter,
        }
    }
}

pub(super) fn repair_range(
    tool: &ShapeFixWire,
    face: &Face,
    first: &PcurveGap,
    second: &PcurveGap,
) -> Result<bool, Failure> {
    let curves = RangeCurves::from_entries(first, second)?;
    if same_edge_conic(first, second, &curves.first) {
        return extend_same_edge_conic(tool, face, first, &curves.first);
    }
    let candidate = candidate_for_pair(first, second, &curves)?;
    let Some(candidate) = candidate else {
        return Ok(false);
    };
    let candidate = clipped_candidate(face, first, second, &curves, candidate)?;
    apply_range_candidate(tool, face, first, second, &curves, candidate)
}

fn candidate_for_pair(
    first: &PcurveGap,
    second: &PcurveGap,
    curves: &RangeCurves,
) -> Result<Option<fallback::PairParameters>, Failure> {
    let bounds = domains::for_pair(first, second, &curves.first, &curves.second);
    let intersection = curve_intersection(first, second, &curves.first, &curves.second)?;
    choose_candidate(
        first,
        second,
        curves,
        intersection,
        (bounds.first, bounds.second),
    )
}

fn choose_candidate(
    first: &PcurveGap,
    second: &PcurveGap,
    curves: &RangeCurves,
    intersection: Option<LineIntersection>,
    bounds: ((f64, f64), (f64, f64)),
) -> Result<Option<fallback::PairParameters>, Failure> {
    if let Some(candidate) = intersection
        && acceptable(first, second, &candidate, &curves.first, &curves.second)
    {
        return Ok(Some(pair_parameters(candidate)));
    }
    fallback::find(
        first,
        second,
        &curves.first,
        &curves.second,
        bounds.0,
        bounds.1,
    )
}

fn pair_parameters(candidate: LineIntersection) -> fallback::PairParameters {
    fallback::PairParameters {
        point: candidate.point,
        second_point: candidate.second_point,
        first: candidate.first_parameter,
        second: candidate.second_parameter,
    }
}

fn apply_range_candidate(
    tool: &ShapeFixWire,
    face: &Face,
    first: &PcurveGap,
    second: &PcurveGap,
    curves: &RangeCurves,
    candidate: fallback::PairParameters,
) -> Result<bool, Failure> {
    replace_intersection(tool, face, first, second, curves, &candidate.into())?;
    tool.set_last_fix_status(ShapeExtendStatus::Done1);
    Ok(true)
}

fn same_edge_conic(first: &PcurveGap, second: &PcurveGap, curve: &Handle<dyn Geom2dCurve>) -> bool {
    first.index == second.index
        && (!curve.cast::<Geom2dCircle>().is_null() || !curve.cast::<Geom2dEllipse>().is_null())
}

fn extend_same_edge_conic(
    tool: &ShapeFixWire,
    face: &Face,
    entry: &PcurveGap,
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<bool, Failure> {
    let difference = PI - (native_end_parameter(entry) - native_start_parameter(entry)).abs() * 0.5;
    let range = (entry.first - difference, entry.last + difference);
    replace::replace_single_pcurve_range(tool, face, entry, curve, range)?;
    tool.set_last_fix_status(ShapeExtendStatus::Done1);
    Ok(true)
}

fn clipped_candidate(
    face: &Face,
    first: &PcurveGap,
    second: &PcurveGap,
    curves: &RangeCurves,
    candidate: fallback::PairParameters,
) -> Result<fallback::PairParameters, Failure> {
    if !requires_surface_clip(first, second, &candidate) {
        return Ok(candidate);
    }
    let parameters = clipped_parameters(face, first, second, curves, candidate)?;
    Ok(parameters)
}

fn clipped_parameters(
    face: &Face,
    first: &PcurveGap,
    second: &PcurveGap,
    curves: &RangeCurves,
    candidate: fallback::PairParameters,
) -> Result<fallback::PairParameters, Failure> {
    let clipped = clipping::apply(
        face,
        first,
        second,
        &curves.first,
        &curves.second,
        (candidate.first, candidate.second),
    )?;
    Ok(clipped.into())
}

fn curve_intersection(
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

fn acceptable(
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

fn requires_surface_clip(
    first: &PcurveGap,
    second: &PcurveGap,
    candidate: &fallback::PairParameters,
) -> bool {
    parameter_outside(first, candidate.first) || parameter_outside(second, candidate.second)
}

fn parameter_outside(entry: &PcurveGap, parameter: f64) -> bool {
    parameter < entry.first || parameter > entry.last
}

fn replace_intersection(
    tool: &ShapeFixWire,
    face: &Face,
    first: &PcurveGap,
    second: &PcurveGap,
    curves: &RangeCurves,
    intersection: &LineIntersection,
) -> Result<(), Failure> {
    let first_range = updated_range(first, intersection.first_parameter, true);
    let second_range = updated_range(second, intersection.second_parameter, false);
    let repairs = range_repairs(first, second, curves, (first_range, second_range))?;
    replace::replace_pcurves_with_ranges(tool, face, (first, second), (&repairs.0, &repairs.1))
}

fn range_repairs(
    first: &PcurveGap,
    second: &PcurveGap,
    curves: &RangeCurves,
    ranges: ((f64, f64), (f64, f64)),
) -> Result<(Option<replace::CurveRepair>, Option<replace::CurveRepair>), Failure> {
    let first_repair = range_repair(first, &curves.first, ranges.0, true)?;
    let second_repair = range_repair(second, &curves.second, ranges.1, false)?;
    Ok((first_repair, second_repair))
}

fn updated_range(entry: &PcurveGap, parameter: f64, first_edge: bool) -> (f64, f64) {
    let (mut first, mut last) = (entry.first, entry.last);
    let unchanged = unchanged_parameter(entry, first_edge);
    if (parameter - unchanged).abs() > precision::p_confusion() {
        if first_edge == entry.reversed {
            first = parameter;
        } else {
            last = parameter;
        }
    }
    (first, last)
}

fn range_repair(
    entry: &PcurveGap,
    curve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    first_edge: bool,
) -> Result<Option<replace::CurveRepair>, Failure> {
    let unchanged = unchanged_parameter(entry, first_edge);
    let parameter = if first_edge == entry.reversed {
        range.0
    } else {
        range.1
    };
    if (parameter - unchanged).abs() <= precision::p_confusion() {
        return Ok(None);
    }
    let curve = ranged_curve(entry, curve, range)?;
    Ok(Some(replace::CurveRepair {
        curve,
        first: range.0,
        last: range.1,
    }))
}

fn ranged_curve(
    entry: &PcurveGap,
    curve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if !entry.trimmed {
        return Ok(curve.clone());
    }
    Ok(Handle::new(Geom2dTrimmedCurve::new(
        curve, range.0, range.1, true, true,
    )?)
    .cast())
}

fn unchanged_parameter(entry: &PcurveGap, first_edge: bool) -> f64 {
    if first_edge == entry.reversed {
        entry.first
    } else {
        entry.last
    }
}
