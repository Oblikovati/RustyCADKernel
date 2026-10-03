//! Three-piece replacement for long coincident segments.
#[path = "triple/second.rs"]
mod second;
use super::super::super::ShapeFixIntersectionTool;
use super::super::super::boxes::{self, EdgeBox};
use super::super::RepairCounts;
use super::SegmentPoints;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildEdge;
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point3;

#[allow(clippy::too_many_arguments)] // Mirrors native's overlap repair inputs.
pub(super) fn repair(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    first_index: i32,
    second_index: i32,
    first: &Edge,
    second: &Edge,
    points: &SegmentPoints,
    max_vertex_tolerance: f64,
    boxes: &mut Vec<EdgeBox>,
    counts: &mut RepairCounts<'_>,
    second_box: &BoundingBox2,
) -> Result<super::super::PairAction, Failure> {
    let Some(mut candidate_points) = candidate_points(points, first, max_vertex_tolerance)? else {
        return Ok(super::super::PairAction::Next);
    };
    if candidate_points.first_key > 1 || candidate_points.last_key > 1 {
        return Ok(super::super::PairAction::Next);
    }
    let first_split = split_first_overlap(
        tool,
        wire,
        face,
        first_index,
        points,
        &candidate_points,
        boxes,
        counts,
    )?;
    split_second_replace_and_remove(
        tool,
        wire,
        face,
        second_index,
        second,
        points,
        &mut candidate_points,
        first_split,
        second_box,
        boxes,
        counts,
    )
}

struct CandidatePoints {
    first: Vertex,
    last: Vertex,
    first_tolerance: f64,
    last_tolerance: f64,
    first_key: i32,
    last_key: i32,
    first_point: Point3,
    last_point: Point3,
}

fn candidate_points(
    points: &SegmentPoints,
    edge: &Edge,
    max_tolerance: f64,
) -> Result<Option<CandidatePoints>, Failure> {
    let p01 = midpoint(points.first[0], points.second[0]);
    let p02 = midpoint(points.first[1], points.second[1]);
    let first_tolerance = triple_tolerance(points.first[0], points.second[0], p01);
    let last_tolerance = triple_tolerance(points.first[1], points.second[1], p02);
    if first_tolerance > max_tolerance || last_tolerance > max_tolerance {
        return Ok(None);
    }
    let (first, first_key) = make_triple_vertex(edge, p01, first_tolerance)?;
    let (last, last_key) = make_triple_vertex(edge, p02, last_tolerance)?;
    Ok(Some(CandidatePoints {
        first,
        last,
        first_tolerance,
        last_tolerance,
        first_key,
        last_key,
        first_point: p01,
        last_point: p02,
    }))
}

fn triple_tolerance(first: Point3, second: Point3, center: Point3) -> f64 {
    first
        .distance(&center)
        .max(second.distance(&center))
        .max(rustycad_tkernel::precision::confusion())
        * 1.00001
}

fn make_triple_vertex(
    edge: &Edge,
    point: Point3,
    tolerance: f64,
) -> Result<(Vertex, i32), Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let endpoints = [analysis.first_vertex(edge), analysis.last_vertex(edge)];
    let mut vertex = Vertex::default();
    let mut matches = 0;
    for endpoint in endpoints {
        matches += match_endpoint(&endpoint, point, tolerance, &mut vertex)?;
    }
    if matches == 0 {
        BRepBuilder::default().make_vertex_at(&mut vertex, point, tolerance)?;
    }
    Ok((vertex, matches))
}

fn match_endpoint(
    endpoint: &Vertex,
    point: Point3,
    tolerance: f64,
    target: &mut Vertex,
) -> Result<i32, Failure> {
    let position = BRepTool::pnt(endpoint)?;
    let new_tolerance = tolerance.max(BRepTool::vertex_tolerance(endpoint)?);
    if point.distance(&position) >= new_tolerance {
        return Ok(0);
    }
    BRepBuilder::default().make_vertex_at(target, position, new_tolerance)?;
    target.set_orientation(endpoint.orientation());
    Ok(1)
}

#[allow(clippy::too_many_arguments)] // Mirrors native's split-and-retry state.
fn split_first_overlap(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    index: i32,
    points: &SegmentPoints,
    candidates: &CandidatePoints,
    boxes: &mut Vec<EdgeBox>,
    counts: &mut RepairCounts<'_>,
) -> Result<(i32, i32), Failure> {
    let mut inserted = 0;
    let mut segment_index = index;
    if candidates.first_key == 0 && candidates.last_key > 0 {
        inserted = split_if_success(
            tool,
            wire,
            face,
            index,
            points.first_parameters.0,
            &candidates.first,
            candidates.first_tolerance,
            boxes,
            counts,
        )?;
        if inserted > 0 {
            segment_index = index + 1;
        }
        return Ok((inserted, segment_index));
    }
    if candidates.first_key > 0 && candidates.last_key == 0 {
        inserted = split_if_success(
            tool,
            wire,
            face,
            index,
            points.first_parameters.1,
            &candidates.last,
            candidates.last_tolerance,
            boxes,
            counts,
        )?;
        if inserted > 0 {
            segment_index = index + 1;
        }
        return Ok((inserted, segment_index));
    }
    if candidates.first_key > 0 || candidates.last_key > 0 {
        return Ok((inserted, segment_index));
    }
    inserted += split_if_success(
        tool,
        wire,
        face,
        index,
        points.first_parameters.0,
        &candidates.first,
        candidates.first_tolerance,
        boxes,
        counts,
    )?;
    let current = wire.edge(index)?;
    let (_, first_range, last_range) =
        ShapeAnalysisEdge::new().pcurve_on_face(&current, face, false)?;
    let outside =
        (first_range - points.first_parameters.1) * (last_range - points.first_parameters.1) > 0.0;
    let next_index = index + i32::from(outside);
    let second_inserted = split_if_success(
        tool,
        wire,
        face,
        next_index,
        points.first_parameters.1,
        &candidates.last,
        candidates.last_tolerance,
        boxes,
        counts,
    )?;
    inserted += second_inserted;
    if second_inserted > 0 {
        segment_index = index + 1;
    }
    Ok((inserted, segment_index))
}

#[allow(clippy::too_many_arguments)] // Keeps native edge replacement inputs explicit.
fn split_if_success(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    index: i32,
    parameter: f64,
    vertex: &Vertex,
    tolerance: f64,
    boxes: &mut Vec<EdgeBox>,
    counts: &mut RepairCounts<'_>,
) -> Result<i32, Failure> {
    if tool.split_edge1(wire, face, index, parameter, vertex, tolerance, boxes)? {
        *counts.split += 1;
        return Ok(1);
    }
    Ok(0)
}

#[allow(clippy::too_many_arguments)] // Mirrors native's second-edge replacement sequence.
fn split_second_replace_and_remove(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    second_index: i32,
    second: &Edge,
    points: &SegmentPoints,
    candidates: &mut CandidatePoints,
    first_split: (i32, i32),
    second_box: &BoundingBox2,
    boxes: &mut Vec<EdgeBox>,
    counts: &mut RepairCounts<'_>,
) -> Result<super::super::PairAction, Failure> {
    let shifted_index = second_index + first_split.0;
    let (_, second_keys) = replace_overlap_endpoints(
        tool,
        wire,
        shifted_index,
        second,
        candidates,
        second_box,
        boxes,
        counts,
    )?;
    let second_split = second::split_second_overlap(
        tool,
        wire,
        face,
        shifted_index,
        points,
        candidates,
        second_keys,
        boxes,
        counts,
    )?;
    wire.remove(second_split.1)?;
    wire.remove(first_split.1)?;
    *counts.removed += 2;
    Ok(super::super::PairAction::Next)
}

#[allow(clippy::too_many_arguments)] // Mirrors native endpoint selection and replacement.
fn replace_overlap_endpoints(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    index: i32,
    source: &Edge,
    candidates: &mut CandidatePoints,
    bounds: &BoundingBox2,
    boxes: &mut Vec<EdgeBox>,
    counts: &mut RepairCounts<'_>,
) -> Result<(Edge, (i32, i32)), Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let mut current = source.clone();
    let mut first = analysis.first_vertex(&current);
    let mut last = analysis.last_vertex(&current);
    let first_on_first = replace_edge_endpoint(
        tool,
        wire,
        index,
        &mut current,
        &mut first,
        &last,
        true,
        &candidates.first,
        &candidates.first_point,
        &mut candidates.first_tolerance,
        counts,
    )?;
    let last_on_first = replace_edge_endpoint(
        tool,
        wire,
        index,
        &mut current,
        &mut first,
        &last,
        true,
        &candidates.last,
        &candidates.last_point,
        &mut candidates.last_tolerance,
        counts,
    )?;
    let first_on_last = replace_edge_endpoint(
        tool,
        wire,
        index,
        &mut current,
        &mut last,
        &first,
        false,
        &candidates.first,
        &candidates.first_point,
        &mut candidates.first_tolerance,
        counts,
    )?;
    let last_on_last = replace_edge_endpoint(
        tool,
        wire,
        index,
        &mut current,
        &mut last,
        &first,
        false,
        &candidates.last,
        &candidates.last_point,
        &mut candidates.last_tolerance,
        counts,
    )?;
    wire.set(&current, index)?;
    boxes::replace_box(boxes, &current, bounds.clone());
    let first_key = if first_on_first {
        1
    } else if first_on_last {
        2
    } else {
        0
    };
    let last_key = if last_on_first {
        1
    } else if last_on_last {
        2
    } else {
        0
    };
    Ok((current, (first_key, last_key)))
}

#[allow(clippy::too_many_arguments)] // Keeps the native edge mutation inputs explicit.
fn replace_edge_endpoint(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    index: i32,
    edge: &mut Edge,
    endpoint: &mut Vertex,
    other: &Vertex,
    is_first: bool,
    candidate: &Vertex,
    candidate_point: &Point3,
    candidate_tolerance: &mut f64,
    counts: &mut RepairCounts<'_>,
) -> Result<bool, Failure> {
    let distance = BRepTool::pnt(endpoint)?.distance(candidate_point);
    if distance >= *candidate_tolerance {
        return Ok(false);
    }
    *candidate_tolerance += distance;
    BRepBuilder::default().update_vertex_tolerance(candidate, *candidate_tolerance)?;
    let replacement = oriented(candidate, endpoint.orientation());
    tool.context().replace(
        &endpoint.clone().into_shape(),
        &replacement.clone().into_shape(),
    )?;
    let (first, last) = if is_first {
        (&replacement, other)
    } else {
        (other, &replacement)
    };
    let previous = edge.clone();
    *edge = ShapeBuildEdge.copy_replace_vertices(edge, first, last)?;
    tool.context()
        .replace(&previous.clone().into_shape(), &edge.clone().into_shape())?;
    wire.set(edge, index)?;
    *endpoint = replacement;
    *counts.replaced += 1;
    Ok(true)
}

fn oriented(vertex: &Vertex, orientation: Orientation) -> Vertex {
    let mut result = vertex.clone();
    result.set_orientation(orientation);
    result
}

fn midpoint(first: Point3, second: Point3) -> Point3 {
    Point3::new(
        (first.x() + second.x()) / 2.0,
        (first.y() + second.y()) / 2.0,
        (first.z() + second.z()) / 2.0,
    )
}
