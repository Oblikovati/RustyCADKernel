//! Partial-overlap endpoint reuse and insertion from `FixIntersectingWires`.
use super::super::boxes::EdgeBox;
use super::{OverlapVertices, PairEffect, ShapeFixIntersectionTool, WireRecord};
use super::{
    edge_containing_parameter, find_middle_edge, make_overlap_vertex, middle_edge_index,
    overlap_vertex, replace_second_edge,
};
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildEdge;
use crate::shape_fix::wire::wire_intersection_tool::aggregate::segment::SegmentPoints;
use crate::shape_fix::wire::wire_intersection_tool::boxes;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

#[allow(clippy::too_many_arguments)]
pub(super) fn replace_partial_overlap(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    first: &mut WireRecord,
    second: &mut WireRecord,
    first_index: i32,
    second_index: i32,
    edge1: &Edge,
    edge2: Edge,
    points: &SegmentPoints,
    max_tolerance: &mut f64,
) -> Result<PairEffect, Failure> {
    let Some(vertices) = partial_overlap_vertices(edge1, points, *max_tolerance)? else {
        return Ok(PairEffect::None);
    };
    if !split_partial_source(tool, face, first, first_index, points, &vertices)? {
        return Ok(PairEffect::None);
    }
    let shared = find_middle_edge(&first.data, face, points.first_parameters)?;
    if !replace_partial_target(
        tool,
        face,
        second,
        second_index,
        &edge2,
        points,
        &vertices,
        &shared,
        &first.boxes,
    )? {
        return Ok(PairEffect::None);
    }
    *max_tolerance = (*max_tolerance)
        .max(vertices.tolerances.0)
        .max(vertices.tolerances.1);
    Ok(PairEffect::Replaced)
}

fn partial_overlap_vertices(
    edge: &Edge,
    points: &SegmentPoints,
    maximum: f64,
) -> Result<Option<OverlapVertices>, Failure> {
    let (start_point, start_tolerance) = overlap_vertex(points.first[0], points.second[0]);
    let (end_point, end_tolerance) = overlap_vertex(points.first[1], points.second[1]);
    if start_tolerance > maximum || end_tolerance > maximum {
        return Ok(None);
    }
    let Some(start) = reuse_or_make_vertex(edge, start_point, start_tolerance)? else {
        return Ok(None);
    };
    let Some(end) = reuse_or_make_vertex(edge, end_point, end_tolerance)? else {
        return Ok(None);
    };
    Ok(Some(OverlapVertices {
        start,
        end,
        tolerances: (start_tolerance, end_tolerance),
    }))
}

fn reuse_or_make_vertex(
    edge: &Edge,
    point: Point3,
    tolerance: f64,
) -> Result<Option<Vertex>, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let endpoints = [analysis.first_vertex(edge), analysis.last_vertex(edge)];
    let matches = endpoint_matches(&endpoints, point, tolerance)?;
    if matches.len() > 1 {
        return Ok(None);
    }
    if let Some(vertex) = matches.first() {
        return Ok(Some(vertex.clone()));
    }
    Ok(Some(make_overlap_vertex(point, tolerance)?))
}

fn endpoint_matches(
    endpoints: &[Vertex; 2],
    point: Point3,
    tolerance: f64,
) -> Result<Vec<Vertex>, Failure> {
    let mut matches = Vec::new();
    for vertex in endpoints {
        let distance = BRepTool::pnt(vertex)?.distance(&point);
        let accepted = tolerance.max(BRepTool::vertex_tolerance(vertex)?);
        if distance < accepted {
            matches.push(vertex.clone());
        }
    }
    Ok(matches)
}

#[allow(clippy::too_many_arguments)]
fn split_partial_source(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    wire: &mut WireRecord,
    index: i32,
    points: &SegmentPoints,
    vertices: &OverlapVertices,
) -> Result<bool, Failure> {
    if !split_boundary(
        tool,
        face,
        wire,
        index,
        points.first_parameters.0,
        &vertices.start,
        &points.first[0],
    )? {
        return Ok(false);
    }
    let Some(end_index) = edge_containing_parameter(&wire.data, face, points.first_parameters.1)?
    else {
        return Ok(false);
    };
    split_boundary(
        tool,
        face,
        wire,
        end_index,
        points.first_parameters.1,
        &vertices.end,
        &points.first[1],
    )
}

#[allow(clippy::too_many_arguments)]
fn replace_partial_target(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    wire: &mut WireRecord,
    index: i32,
    source: &Edge,
    points: &SegmentPoints,
    vertices: &OverlapVertices,
    shared: &Edge,
    source_boxes: &[EdgeBox],
) -> Result<bool, Failure> {
    replace_matching_target_vertices(tool, wire, index, source, points, vertices)?;
    let Some(start_index) =
        edge_containing_parameter(&wire.data, face, points.second_parameters.0)?
    else {
        return Ok(false);
    };
    if !split_boundary(
        tool,
        face,
        wire,
        start_index,
        points.second_parameters.0,
        &vertices.start,
        &points.second[0],
    )? {
        return Ok(false);
    }
    let Some(split_index) =
        edge_containing_parameter(&wire.data, face, points.second_parameters.1)?
    else {
        return Ok(false);
    };
    if !split_boundary(
        tool,
        face,
        wire,
        split_index,
        points.second_parameters.1,
        &vertices.end,
        &points.second[1],
    )? {
        return Ok(false);
    }
    let middle_index = middle_edge_index(&wire.data, face, points.second_parameters)?;
    let middle = wire.data.edge(middle_index)?;
    replace_second_edge(tool, wire, middle_index, &middle, shared, source_boxes)?;
    Ok(true)
}

fn replace_matching_target_vertices(
    tool: &ShapeFixIntersectionTool,
    wire: &mut WireRecord,
    index: i32,
    edge: &Edge,
    points: &SegmentPoints,
    vertices: &OverlapVertices,
) -> Result<(), Failure> {
    let mut current = edge.clone();
    replace_target_endpoint(
        tool,
        wire,
        index,
        &mut current,
        points.second[0],
        &vertices.start,
    )?;
    replace_target_endpoint(
        tool,
        wire,
        index,
        &mut current,
        points.second[1],
        &vertices.end,
    )?;
    Ok(())
}

fn replace_target_endpoint(
    tool: &ShapeFixIntersectionTool,
    wire: &mut WireRecord,
    index: i32,
    edge: &mut Edge,
    point: Point3,
    shared: &Vertex,
) -> Result<bool, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let endpoints = [analysis.first_vertex(edge), analysis.last_vertex(edge)];
    let tolerance = BRepTool::vertex_tolerance(shared)?;
    let Some((is_first, endpoint, distance)) =
        nearest_matched_endpoint(&endpoints, point, tolerance)?
    else {
        return Ok(false);
    };
    BRepBuilder::default().update_vertex_tolerance(shared, tolerance + distance)?;
    let replacement_vertex = oriented_like(shared, &endpoint);
    let (first, last) = if is_first {
        (&replacement_vertex, &endpoints[1])
    } else {
        (&endpoints[0], &replacement_vertex)
    };
    let replacement = ShapeBuildEdge.copy_replace_vertices(edge, first, last)?;
    tool.context().replace(
        &edge.clone().into_shape(),
        &replacement.clone().into_shape(),
    )?;
    wire.data.set(&replacement, index)?;
    replace_box(&mut wire.boxes, edge, &replacement)?;
    *edge = replacement;
    Ok(true)
}

fn nearest_matched_endpoint(
    endpoints: &[Vertex; 2],
    point: Point3,
    tolerance: f64,
) -> Result<Option<(bool, Vertex, f64)>, Failure> {
    let distances = [
        BRepTool::pnt(&endpoints[0])?.distance(&point),
        BRepTool::pnt(&endpoints[1])?.distance(&point),
    ];
    let Some(index) = (0..2)
        .filter(|index| distances[*index] < tolerance)
        .min_by(|a, b| distances[*a].total_cmp(&distances[*b]))
    else {
        return Ok(None);
    };
    Ok(Some((
        index == 0,
        endpoints[index].clone(),
        distances[index],
    )))
}

fn oriented_like(shared: &Vertex, endpoint: &Vertex) -> Vertex {
    let mut replacement = shared.clone();
    replacement.set_orientation(endpoint.orientation());
    replacement
}

fn split_boundary(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    wire: &mut WireRecord,
    index: i32,
    parameter: f64,
    vertex: &Vertex,
    point: &Point3,
) -> Result<bool, Failure> {
    let edge = wire.data.edge(index)?;
    let range = BRepTool::range_on_face(&edge, face)?;
    if (parameter - range.0).abs() <= 1.0e-7 || (parameter - range.1).abs() <= 1.0e-7 {
        return endpoint_is_shared(&edge, vertex, point);
    }
    tool.split_edge1(
        &wire.data,
        face,
        index,
        parameter,
        vertex,
        1.0e-5,
        &mut wire.boxes,
    )
}

fn endpoint_is_shared(edge: &Edge, shared: &Vertex, point: &Point3) -> Result<bool, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    for endpoint in [analysis.first_vertex(edge), analysis.last_vertex(edge)] {
        if BRepTool::pnt(&endpoint)?.distance(point) <= 1.0e-7 {
            return Ok(endpoint.is_same(shared));
        }
    }
    Ok(false)
}

fn replace_box(boxes: &mut Vec<EdgeBox>, source: &Edge, replacement: &Edge) -> Result<(), Failure> {
    let bounds = boxes::find_edge_box(boxes, source);
    boxes.retain(|entry| !entry.edge.is_same(source));
    if let Some(bounds) = bounds {
        boxes.push(EdgeBox {
            edge: replacement.clone(),
            bounds,
        });
    }
    Ok(())
}
