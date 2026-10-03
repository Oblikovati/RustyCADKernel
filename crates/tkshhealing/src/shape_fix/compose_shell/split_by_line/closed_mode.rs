use super::super::ShapeFixComposeShell;
use super::SplitIntersection;
use super::helpers::{
    IOR_BOTH, IOR_POS, ITP_BEGSEG, ITP_ENDSEG, ITP_INTER, ITP_TANG, oriented_range_endpoint,
    point_code, shared_tangential_point,
};
use crate::shape_fix::ShapeFixWireSegment;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Line2;

#[derive(Clone, Copy)]
pub(super) enum PeriodicAxis {
    U { line_coordinate: f64, period: f64 },
    V { line_coordinate: f64, period: f64 },
}

pub(super) fn periodic_cut_direction(
    composer: &ShapeFixComposeShell,
    line: Line2,
) -> Option<PeriodicAxis> {
    if !composer.closed_mode() {
        return None;
    }
    let fields = composer.fields.read();
    let direction = line.direction();
    let location = line.location();
    if fields.u_closed && direction.x().abs() < precision::p_confusion() {
        return Some(PeriodicAxis::U {
            line_coordinate: location.x(),
            period: fields.u_period,
        });
    }
    if fields.v_closed && direction.y().abs() < precision::p_confusion() {
        return Some(PeriodicAxis::V {
            line_coordinate: location.y(),
            period: fields.v_period,
        });
    }
    None
}

pub(super) fn remove_duplicates(
    composer: &ShapeFixComposeShell,
    wire: &ShapeFixWireSegment,
    intersections: &mut Vec<SplitIntersection>,
) -> Result<(), Failure> {
    if !composer.closed_mode() || intersections.len() < 2 {
        return Ok(());
    }
    let mut index = 0;
    let mut previous = intersections.len() - 1;
    while index < intersections.len() {
        if index == previous {
            break;
        }
        if duplicate_at_pair(composer, wire, intersections, index, previous)? {
            intersections.remove(index);
            if previous > index {
                previous -= 1;
            }
        } else {
            previous = index;
            index += 1;
        }
    }
    Ok(())
}

fn duplicate_at_pair(
    composer: &ShapeFixComposeShell,
    wire: &ShapeFixWireSegment,
    intersections: &[SplitIntersection],
    index: usize,
    previous: usize,
) -> Result<bool, Failure> {
    let first = intersections[index];
    let tail = intersections[previous];
    if first.edge_index == tail.edge_index {
        return Ok((first.edge_parameter - tail.edge_parameter).abs() < precision::p_confusion());
    }
    adjacent_endpoint_duplicate(composer, wire, first, tail)
}

fn adjacent_endpoint_duplicate(
    composer: &ShapeFixComposeShell,
    wire: &ShapeFixWireSegment,
    first: SplitIntersection,
    tail: SplitIntersection,
) -> Result<bool, Failure> {
    let edge_count = wire.nb_edges();
    if edge_count != 1 && first.edge_index != tail.edge_index % edge_count + 1 {
        return Ok(false);
    }
    let face = composer.fields.read().face.clone();
    let first_edge = wire.edge(first.edge_index)?;
    let tail_edge = wire.edge(tail.edge_index)?;
    let tail_range = BRepTool::range_on_face(&tail_edge, &face)?;
    let first_range = BRepTool::range_on_face(&first_edge, &face)?;
    let tail_end = oriented_range_endpoint(&tail_edge, tail_range, false);
    let first_start = oriented_range_endpoint(&first_edge, first_range, true);
    Ok(
        (tail.edge_parameter - tail_end).abs() < precision::p_confusion()
            && (first.edge_parameter - first_start).abs() < precision::p_confusion(),
    )
}

pub(super) fn classify_output(
    composer: &ShapeFixComposeShell,
    orientation: Orientation,
    segment_codes: &[i32],
    intersections: &[SplitIntersection],
) -> (Vec<i32>, Vec<i32>) {
    if !composer.closed_mode() {
        return (
            classify_open_points(orientation, segment_codes, intersections),
            segment_codes.to_vec(),
        );
    }
    if orientation == Orientation::External {
        return (
            vec![ITP_TANG | IOR_BOTH; segment_codes.len()],
            segment_codes.to_vec(),
        );
    }
    classify_closed_points(orientation, segment_codes, intersections)
}

fn classify_open_points(
    orientation: Orientation,
    segment_codes: &[i32],
    intersections: &[SplitIntersection],
) -> Vec<i32> {
    if orientation == Orientation::External {
        return vec![64 | IOR_BOTH; segment_codes.len()];
    }
    let is_internal = orientation == Orientation::Internal;
    (0..segment_codes.len())
        .map(|index| point_code(index, segment_codes, intersections, is_internal))
        .collect()
}

fn classify_closed_points(
    orientation: Orientation,
    segment_codes: &[i32],
    intersections: &[SplitIntersection],
) -> (Vec<i32>, Vec<i32>) {
    let mut points = Vec::with_capacity(segment_codes.len());
    let mut segments = Vec::with_capacity(segment_codes.len());
    let internal = orientation == Orientation::Internal;
    for index in 0..segment_codes.len() {
        let (point, next) =
            closed_point_and_segments(index, segment_codes, intersections, internal);
        points.push(point);
        segments.extend(next);
    }
    (points, segments)
}

fn closed_point_and_segments(
    index: usize,
    codes: &[i32],
    intersections: &[SplitIntersection],
    internal: bool,
) -> (i32, Vec<i32>) {
    let previous = (index + codes.len() - 1) % codes.len();
    let before = normalize_both(codes[previous], false);
    let after = normalize_both(codes[index], true);
    let point = closed_point_code(index, before, after, codes.len(), intersections, internal);
    let mut segments = vec![after];
    if intersections[index].edge_index == intersections[previous].edge_index {
        segments.push(before);
    }
    (point, segments)
}

fn normalize_both(code: i32, current: bool) -> i32 {
    if code & IOR_BOTH != IOR_BOTH {
        return code;
    }
    let positive = code & IOR_POS != 0;
    match (current, positive) {
        (false, true) | (true, false) => 2,
        _ => 1,
    }
}

fn closed_point_code(
    index: usize,
    before: i32,
    after: i32,
    count: usize,
    intersections: &[SplitIntersection],
    internal: bool,
) -> i32 {
    let code = before | after;
    if before == 0 {
        return tangential_start_code(index, count, intersections, code);
    }
    if after == 0 {
        return tangential_end_code(index, count, intersections, code);
    }
    if index == (index + count - 1) % count {
        return if code & IOR_BOTH == IOR_BOTH && !internal {
            code | ITP_INTER
        } else {
            code | ITP_TANG
        };
    }
    code | if internal || before == after {
        ITP_TANG
    } else {
        ITP_INTER
    }
}

fn tangential_start_code(
    index: usize,
    count: usize,
    points: &[SplitIntersection],
    code: i32,
) -> i32 {
    let previous = (index + count - 1) % count;
    code | if points[index].line_parameter > points[previous].line_parameter {
        ITP_ENDSEG
    } else {
        ITP_BEGSEG
    }
}

fn tangential_end_code(index: usize, count: usize, points: &[SplitIntersection], code: i32) -> i32 {
    let next = (index + 1) % count;
    code | if points[next].line_parameter > points[index].line_parameter {
        ITP_BEGSEG
    } else {
        ITP_ENDSEG
    }
}

pub(super) fn mergeable_tangential_point(
    closed_mode: bool,
    intersections: &[SplitIntersection],
    codes: &[i32],
) -> Option<usize> {
    if !closed_mode {
        return shared_tangential_point(codes);
    }
    let count = codes.len();
    (0..count).find(|index| {
        let previous = (index + count - 1) % count;
        codes[previous] == 0 && codes[*index] == 0 && !retain_tangency(*index, intersections)
    })
}

fn retain_tangency(index: usize, points: &[SplitIntersection]) -> bool {
    let count = points.len();
    let previous = (index + count - 1) % count;
    let next = (index + 1) % count;
    let current = points[index].line_parameter;
    (points[previous].line_parameter - current) * (points[next].line_parameter - current) <= 0.0
}
