//! Bounded collinear overlap repairs for nonadjacent p-curves.
#[path = "segment/triple.rs"]
mod triple;
use super::super::ShapeFixIntersectionTool;
use super::super::boxes::EdgeBox;
use super::super::point_on_edge;
use super::{PairAction, PairCurves, RepairCounts};
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::int_res2d::IntersectionSegment;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point3;

#[allow(clippy::too_many_arguments)] // Mirrors native segment repair inputs.
pub(super) fn repair(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    first_index: i32,
    second_index: i32,
    first: &Edge,
    second: &Edge,
    first_box: &BoundingBox2,
    second_box: &BoundingBox2,
    curves: &PairCurves,
    segment: IntersectionSegment,
    max_vertex_tolerance: &mut f64,
    counts: &mut RepairCounts<'_>,
    boxes: &mut Vec<EdgeBox>,
) -> Result<PairAction, Failure> {
    let samples = segment_points(segment, first, second, face, curves)?;
    if separated_overlap(&samples, *max_vertex_tolerance) {
        return Ok(PairAction::Next);
    }
    let first_selected = select_existing_endpoint(
        first,
        curves.ranges[0],
        samples.first_parameters,
        &samples.first,
        *max_vertex_tolerance,
    )?;
    let (first_modified, first_selected) = cut_selected_endpoint(
        tool,
        first,
        face,
        curves.ranges[0],
        samples.first_parameters,
        first_selected,
        counts,
    )?;
    let second_selected = select_existing_endpoint(
        second,
        curves.ranges[1],
        samples.second_parameters,
        &samples.second,
        *max_vertex_tolerance,
    )?;
    let (second_modified, second_selected) = cut_selected_endpoint(
        tool,
        second,
        face,
        curves.ranges[1],
        samples.second_parameters,
        second_selected,
        counts,
    )?;
    if first_modified && !second_modified {
        return split_overlap_edge(
            tool,
            wire,
            face,
            second_index,
            samples.second_parameters,
            first_selected,
            boxes,
            counts,
            PairAction::RetrySecond,
        );
    }
    if !first_modified && second_modified {
        return split_overlap_edge(
            tool,
            wire,
            face,
            first_index,
            samples.first_parameters,
            second_selected,
            boxes,
            counts,
            PairAction::RetryFirst,
        );
    }
    if first_modified || second_modified {
        return Ok(PairAction::Next);
    }
    split_close_overlap(
        tool,
        wire,
        face,
        first_index,
        second_index,
        first,
        second,
        &samples,
        curves,
        max_vertex_tolerance,
        boxes,
        counts,
        first_box,
        second_box,
    )
}

struct SegmentPoints {
    first_parameters: (f64, f64),
    second_parameters: (f64, f64),
    first: [Point3; 2],
    second: [Point3; 2],
}

struct SelectedEndpoint {
    vertex: Vertex,
    tolerance: f64,
    parameter: f64,
    found: bool,
}

fn segment_points(
    segment: IntersectionSegment,
    first: &Edge,
    second: &Edge,
    face: &Face,
    curves: &PairCurves,
) -> Result<SegmentPoints, Failure> {
    let first_hit = segment.first_point()?;
    let last_hit = segment.last_point()?;
    let first_parameters = (first_hit.param_on_first(), last_hit.param_on_first());
    let second_parameters = (first_hit.param_on_second(), last_hit.param_on_second());
    let first = [
        point_on_edge(first, first_parameters.0, &curves.pcurves[0], face)?,
        point_on_edge(first, first_parameters.1, &curves.pcurves[0], face)?,
    ];
    let second = [
        point_on_edge(second, second_parameters.0, &curves.pcurves[1], face)?,
        point_on_edge(second, second_parameters.1, &curves.pcurves[1], face)?,
    ];
    Ok(SegmentPoints {
        first_parameters,
        second_parameters,
        first,
        second,
    })
}

fn separated_overlap(points: &SegmentPoints, tolerance: f64) -> bool {
    points.first[0].distance(&points.second[0]) > tolerance
        || points.first[1].distance(&points.second[1]) > tolerance
}

fn select_existing_endpoint(
    edge: &Edge,
    range: (f64, f64),
    parameters: (f64, f64),
    overlap_points: &[Point3; 2],
    max_tolerance: f64,
) -> Result<SelectedEndpoint, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let first = analysis.first_vertex(edge);
    let last = analysis.last_vertex(edge);
    let mut selected = SelectedEndpoint {
        vertex: Vertex::default(),
        tolerance: 0.0,
        parameter: 0.0,
        found: false,
    };
    let (first_parameter, last_parameter) = endpoint_parameters(edge, range);
    consider_endpoint(
        &first,
        first_parameter,
        parameters,
        overlap_points,
        range,
        max_tolerance,
        &mut selected,
    )?;
    consider_endpoint(
        &last,
        last_parameter,
        parameters,
        overlap_points,
        range,
        max_tolerance,
        &mut selected,
    )?;
    Ok(selected)
}

fn consider_endpoint(
    vertex: &Vertex,
    vertex_parameter: f64,
    parameters: (f64, f64),
    overlap_points: &[Point3; 2],
    range: (f64, f64),
    max_tolerance: f64,
    selected: &mut SelectedEndpoint,
) -> Result<(), Failure> {
    let position = BRepTool::pnt(vertex)?;
    let maximum_distance = overlap_points[0]
        .distance(&position)
        .max(overlap_points[1].distance(&position));
    let parameter_distance = (vertex_parameter - parameters.0)
        .abs()
        .max((vertex_parameter - parameters.1).abs());
    let short_parameter_span = parameter_distance < (range.1 - range.0).abs() * 0.01;
    if maximum_distance >= max_tolerance && !short_parameter_span {
        return Ok(());
    }
    if selected.found && maximum_distance >= selected.tolerance {
        return Ok(());
    }
    selected.vertex = vertex.clone();
    selected.tolerance = maximum_distance;
    selected.parameter = vertex_parameter;
    selected.found = true;
    Ok(())
}

fn endpoint_parameters(edge: &Edge, range: (f64, f64)) -> (f64, f64) {
    if edge.orientation() == Orientation::Reversed {
        (range.1, range.0)
    } else {
        range
    }
}

fn cut_selected_endpoint(
    tool: &ShapeFixIntersectionTool,
    edge: &Edge,
    face: &Face,
    range: (f64, f64),
    parameters: (f64, f64),
    mut selected: SelectedEndpoint,
    counts: &mut RepairCounts<'_>,
) -> Result<(bool, SelectedEndpoint), Failure> {
    if !selected.found {
        return Ok((false, selected));
    }
    let endpoint_gap_first = (range.0 - parameters.0).abs() + (range.0 - parameters.1).abs();
    let endpoint_gap_last = (range.1 - parameters.0).abs() + (range.1 - parameters.1).abs();
    let endpoint = if endpoint_gap_first > endpoint_gap_last {
        range.0
    } else {
        range.1
    };
    let cut = if (endpoint - parameters.0).abs() > (endpoint - parameters.1).abs() {
        parameters.1
    } else {
        parameters.0
    };
    let mut cut_line = false;
    if tool.cut_edge(edge, endpoint, cut, face, &mut cut_line)? {
        *counts.cut += 1;
    }
    selected.tolerance = selected
        .tolerance
        .max(BRepTool::vertex_tolerance(&selected.vertex)?);
    BRepBuilder::default().update_vertex_tolerance(&selected.vertex, selected.tolerance)?;
    Ok((true, selected))
}

#[allow(clippy::too_many_arguments)] // Mirrors native split-and-retry state.
fn split_overlap_edge(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    index: i32,
    parameters: (f64, f64),
    selected: SelectedEndpoint,
    boxes: &mut Vec<EdgeBox>,
    counts: &mut RepairCounts<'_>,
    retry: PairAction,
) -> Result<PairAction, Failure> {
    if !selected.found {
        return Ok(PairAction::Next);
    }
    if tool.split_edge2(
        wire,
        face,
        index,
        parameters,
        &selected.vertex,
        selected.tolerance,
        boxes,
    )? {
        *counts.split += 1;
        return Ok(retry);
    }
    Ok(PairAction::Next)
}

#[allow(clippy::too_many_arguments)]
fn split_close_overlap(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    first_index: i32,
    second_index: i32,
    first: &Edge,
    second: &Edge,
    points: &SegmentPoints,
    curves: &PairCurves,
    max_vertex_tolerance: &mut f64,
    boxes: &mut Vec<EdgeBox>,
    counts: &mut RepairCounts<'_>,
    first_box: &BoundingBox2,
    second_box: &BoundingBox2,
) -> Result<PairAction, Failure> {
    let first_middle = point_on_edge(
        first,
        0.5 * (points.first_parameters.0 + points.first_parameters.1),
        &curves.pcurves[0],
        face,
    )?;
    let second_middle = point_on_edge(
        second,
        0.5 * (points.second_parameters.0 + points.second_parameters.1),
        &curves.pcurves[1],
        face,
    )?;
    let vertex_point = midpoint(first_middle, second_middle);
    let tolerance = overlap_vertex_tolerance(points, vertex_point, first_middle, second_middle);
    if tolerance >= *max_vertex_tolerance {
        return triple::repair(
            tool,
            wire,
            face,
            first_index,
            second_index,
            first,
            second,
            points,
            *max_vertex_tolerance,
            boxes,
            counts,
            second_box,
        );
    }
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, first_middle, tolerance)?;
    *max_vertex_tolerance = max_vertex_tolerance.max(tolerance);
    let second_split = tool.split_edge2(
        wire,
        face,
        second_index,
        points.second_parameters,
        &vertex,
        tolerance,
        boxes,
    )?;
    if second_split {
        *counts.split += 1;
    }
    if tool.split_edge2(
        wire,
        face,
        first_index,
        points.first_parameters,
        &vertex,
        tolerance,
        boxes,
    )? {
        *counts.split += 1;
        return Ok(PairAction::RetryFirst);
    }
    if second_split {
        return Ok(PairAction::RetrySecond);
    }
    let _ = (first_box, second_box);
    Ok(PairAction::Next)
}

fn overlap_vertex_tolerance(
    points: &SegmentPoints,
    center: Point3,
    first_middle: Point3,
    second_middle: Point3,
) -> f64 {
    let first_max = points.first[0]
        .distance(&center)
        .max(points.first[1].distance(&center));
    let second_max = points.second[0]
        .distance(&center)
        .max(points.second[1].distance(&center));
    first_max
        .max(second_max)
        .max(first_middle.distance(&second_middle))
        * 1.00001
}

fn midpoint(first: Point3, second: Point3) -> Point3 {
    Point3::new(
        (first.x() + second.x()) / 2.0,
        (first.y() + second.y()) / 2.0,
        (first.z() + second.z()) / 2.0,
    )
}
