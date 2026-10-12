//! OCCT 3d097a0's limit expansion and C1 sampling preserve the last-native-interval endpoint rule.
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::{ncollection::Array1, precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::gcpnts::{TangentialCriteria, TangentialDeflection};
use rustycad_tkmath::{
    geom_abs::{CurveType, Shape},
    gp::Point3,
};
pub(super) fn brep_selection_curve_points(
    curve: &BRepAdaptorCurve,
    deflection: f64,
    angle: f64,
    edge_points: i32,
    max_parameter: f64,
) -> Result<Vec<Point3>, Failure> {
    let (first, last, limited) = brep_selection_curve_limits(curve, max_parameter)?;
    if limited {
        return brep_selection_c1_points(curve, first, last, deflection, angle);
    }
    brep_selection_uniform_points(curve, first, last, edge_points)
}
struct BrepSelectionLimitRange {
    first: f64,
    last: f64,
    negative: bool,
    positive: bool,
}
pub(super) fn brep_selection_curve_limits(
    curve: &dyn Adaptor3dCurve,
    limit: f64,
) -> Result<(f64, f64, bool), Failure> {
    let first = curve.first_parameter()?;
    let last = curve.last_parameter()?;
    let range = BrepSelectionLimitRange {
        first,
        last,
        negative: precision::is_negative_infinite(first),
        positive: precision::is_positive_infinite(last),
    };
    if !range.negative && !range.positive {
        return Ok((first, last, true));
    }
    expand_brep_selection_limits(curve, range, limit)
}
#[allow(clippy::neg_cmp_op_on_partial_ord)] // Native NaN distance exits; replacing !< with >= changes behavior.
fn expand_brep_selection_limits(
    curve: &dyn Adaptor3dCurve,
    mut range: BrepSelectionLimitRange,
    limit: f64,
) -> Result<(f64, f64, bool), Failure> {
    let finite = finite_brep_selection_endpoint(curve, &range)?;
    let mut delta = 1.;
    for _ in 0..100000 {
        delta *= 2.;
        advance_brep_selection_limits(&mut range, delta);
        let distance = brep_selection_limit_distance(curve, &range, finite)?;
        if !(distance < limit) {
            return Ok((range.first, range.last, true));
        }
    }
    Ok((range.first, range.last, false))
}
fn finite_brep_selection_endpoint(
    curve: &dyn Adaptor3dCurve,
    range: &BrepSelectionLimitRange,
) -> Result<Option<Point3>, Failure> {
    // Native evaluates a finite endpoint once, before expanding the opposite bound.
    if !range.negative {
        return Ok(Some(curve.value(range.first)?));
    }
    if !range.positive {
        return Ok(Some(curve.value(range.last)?));
    }
    Ok(None)
}

fn advance_brep_selection_limits(range: &mut BrepSelectionLimitRange, delta: f64) {
    if range.negative {
        range.first = if range.positive {
            -delta
        } else {
            range.last - delta
        };
    }
    if range.positive {
        range.last = if range.negative {
            delta
        } else {
            range.first + delta
        };
    }
}
fn brep_selection_limit_distance(
    curve: &dyn Adaptor3dCurve,
    range: &BrepSelectionLimitRange,
    finite: Option<Point3>,
) -> Result<f64, Failure> {
    if !range.negative {
        return Ok(finite
            .expect("finite first endpoint")
            .distance(&curve.value(range.last)?));
    }
    let first = curve.value(range.first)?;
    let last = if let Some(point) = finite {
        point
    } else {
        curve.value(range.last)?
    };
    Ok(first.distance(&last))
}

struct BrepSelectionC1Range<'a> {
    curve: &'a BRepAdaptorCurve,
    first: f64,
    last: f64,
    criteria: TangentialCriteria,
}
fn brep_selection_c1_points(
    curve: &BRepAdaptorCurve,
    first: f64,
    last: f64,
    deflection: f64,
    angle: f64,
) -> Result<Vec<Point3>, Failure> {
    let intervals = brep_selection_c1_intervals(curve)?;
    let range = BrepSelectionC1Range {
        curve,
        first,
        last,
        criteria: TangentialCriteria::new(angle, deflection),
    };
    collect_brep_selection_c1_points(&range, &intervals)
}
fn collect_brep_selection_c1_points(
    range: &BrepSelectionC1Range<'_>,
    intervals: &Array1<'_, f64>,
) -> Result<Vec<Point3>, Failure> {
    let count = intervals.len() as i32 - 1;
    let mut points = Vec::new();
    for index in 1..=count {
        append_clipped_brep_selection_interval(
            range,
            brep_selection_interval_bounds(intervals, index)?,
            index == count,
            &mut points,
        )?;
    }
    Ok(points)
}
fn brep_selection_c1_intervals(curve: &BRepAdaptorCurve) -> Result<Array1<'static, f64>, Failure> {
    let count = curve.nb_intervals(Shape::C1)?;
    let mut intervals = Array1::with_bounds(1, count + 1)
        .map_err(|error| error.into_failure("StdSelect_BRepSelectionTool C1 intervals"))?;
    curve.intervals(&mut intervals, Shape::C1)?;
    Ok(intervals)
}
fn brep_selection_interval_bounds(
    intervals: &Array1<'_, f64>,
    index: i32,
) -> Result<(f64, f64), Failure> {
    let low = *intervals
        .value(index)
        .map_err(|error| error.into_failure("StdSelect_BRepSelectionTool C1 interval"))?;
    let high = *intervals
        .value(index + 1)
        .map_err(|error| error.into_failure("StdSelect_BRepSelectionTool C1 interval"))?;
    Ok((low, high))
}
fn append_clipped_brep_selection_interval(
    range: &BrepSelectionC1Range<'_>,
    bounds: (f64, f64),
    is_last: bool,
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    let (low, high) = bounds;
    if !(high > range.first && low < range.last) {
        return Ok(());
    }
    // Native std::max/min keep the first argument on unordered comparisons.
    let first = if low < range.first { range.first } else { low };
    let last = if range.last < high { range.last } else { high };
    append_brep_selection_interval(range.curve, first, last, range.criteria, is_last, points)
}

fn append_brep_selection_interval(
    curve: &BRepAdaptorCurve,
    first: f64,
    last: f64,
    criteria: TangentialCriteria,
    is_last: bool,
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    let adaptor: &dyn Adaptor3dCurve = curve;
    let algorithm = TangentialDeflection::with_range(adaptor, first, last, criteria)?;
    let count = algorithm.nb_points();
    for index in 1..count {
        points.push(algorithm.value(index)?);
    }
    if count > 0 && is_last {
        points.push(algorithm.value(count)?);
    }
    Ok(())
}
fn brep_selection_uniform_points(
    curve: &BRepAdaptorCurve,
    first: f64,
    last: f64,
    edge_points: i32,
) -> Result<Vec<Point3>, Failure> {
    let intervals = if curve.get_type()? == CurveType::BSplineCurve {
        (curve.nb_knots()? - 1) / 3
    } else {
        1
    };
    let count = (edge_points * intervals.max(1)).max(2);
    let step = (last - first) / f64::from(count - 1);
    let mut points = Vec::with_capacity(count as usize);
    for index in 0..count {
        points.push(curve.value(first + step * f64::from(index))?);
    }
    Ok(points)
}
