use super::{
    Face, Failure, PcurveGap, ShapeFixWire, clipping, convert, domains,
    fallback::{self, native_end_parameter, native_start_parameter},
    intersections::{self, LineIntersection},
    replace,
};
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dTrimmedCurve};
use std::f64::consts::PI;

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
    let intersection =
        intersections::curve_intersection(first, second, &curves.first, &curves.second)?;
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
        && intersections::acceptable(first, second, &candidate, &curves.first, &curves.second)
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
