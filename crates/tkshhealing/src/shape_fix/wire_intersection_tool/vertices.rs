//! Endpoint selection and replacement for intersecting nonadjacent wire edges.
use super::{EdgeBox, ShapeFixIntersectionTool, boxes::replace_box};
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildEdge;
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;

pub(super) struct VertexUnion {
    pub(super) keep: Vertex,
    pub(super) discard: Vertex,
    pub(super) distance: f64,
    pub(super) tolerance: f64,
}

pub(super) struct VertexCandidate {
    pub(super) vertex: Vertex,
    pub(super) tolerance: f64,
    pub(super) should_split: bool,
}

pub(super) fn closest_vertex_union(
    first: &Edge,
    second: &Edge,
) -> Result<Option<VertexUnion>, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let first_vertices = [analysis.first_vertex(first), analysis.last_vertex(first)];
    let second_vertices = [analysis.first_vertex(second), analysis.last_vertex(second)];
    let distances = vertex_distances(&first_vertices, &second_vertices)?;
    let Some((first_index, second_index, distance)) = closest_pair(&distances) else {
        return Ok(None);
    };
    let keep = first_vertices[first_index].clone();
    let discard = second_vertices[second_index].clone();
    let tolerance = BRepTool::vertex_tolerance(&keep)?.max(BRepTool::vertex_tolerance(&discard)?);
    Ok(Some(VertexUnion {
        keep,
        discard,
        distance,
        tolerance,
    }))
}

fn vertex_distances(first: &[Vertex; 2], second: &[Vertex; 2]) -> Result<[[f64; 2]; 2], Failure> {
    let mut distances = [[0.0; 2]; 2];
    for (row, left) in first.iter().enumerate() {
        for (column, right) in second.iter().enumerate() {
            distances[row][column] = BRepTool::pnt(left)?.distance(&BRepTool::pnt(right)?);
        }
    }
    Ok(distances)
}

fn closest_pair(distances: &[[f64; 2]; 2]) -> Option<(usize, usize, f64)> {
    let [[d11, d12], [d21, d22]] = *distances;
    if d11 < d12 && d11 < d21 && d11 < d22 {
        return Some((0, 0, d11));
    }
    if d12 < d21 && d12 < d22 {
        return Some((0, 1, d12));
    }
    if d21 < d22 {
        return Some((1, 0, d21));
    }
    Some((1, 1, d22))
}

pub(super) fn replace_union_edges(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    second: &mut Edge,
    index: i32,
    union: &VertexUnion,
    boxes: &mut Vec<EdgeBox>,
    second_box: &BoundingBox2,
) -> Result<(), Failure> {
    let (first, last) = edge_vertices(second);
    let replacement = replace_discard_vertex(&ShapeBuildEdge, second, &first, &last, union)?;
    tool.context.replace(
        &second.clone().into_shape(),
        &replacement.clone().into_shape(),
    )?;
    wire.set(&replacement, index)?;
    *second = replacement;
    replace_box(boxes, second, second_box.clone());
    replace_neighbor_vertices(tool, wire, index, union, boxes)
}

fn edge_vertices(edge: &Edge) -> (Vertex, Vertex) {
    let analysis = ShapeAnalysisEdge::new();
    (analysis.first_vertex(edge), analysis.last_vertex(edge))
}

fn replace_discard_vertex(
    build: &ShapeBuildEdge,
    edge: &Edge,
    first: &Vertex,
    last: &Vertex,
    union: &VertexUnion,
) -> Result<Edge, Failure> {
    let new_first = if union.discard.is_same(first) {
        &union.keep
    } else {
        first
    };
    let new_last = if union.discard.is_same(last) {
        &union.keep
    } else {
        last
    };
    build.copy_replace_vertices(edge, new_first, new_last)
}

fn replace_neighbor_vertices(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    index: i32,
    union: &VertexUnion,
    boxes: &mut Vec<EdgeBox>,
) -> Result<(), Failure> {
    let previous = if index > 1 {
        index - 1
    } else {
        wire.nb_edges()
    };
    let next = if index < wire.nb_edges() {
        index + 1
    } else {
        1
    };
    for neighbor_index in [previous, next] {
        replace_neighbor_edge(tool, wire, neighbor_index, union, boxes)?;
    }
    Ok(())
}

fn replace_neighbor_edge(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    index: i32,
    union: &VertexUnion,
    boxes: &mut Vec<EdgeBox>,
) -> Result<(), Failure> {
    let source = wire.edge(index)?;
    let (first, last) = edge_vertices(&source);
    if !union.discard.is_same(&first) && !union.discard.is_same(&last) {
        return Ok(());
    }
    let bounds = take_box(boxes, &source);
    let replacement = replace_discard_vertex(&ShapeBuildEdge, &source, &first, &last, union)?;
    tool.context.replace(
        &source.clone().into_shape(),
        &replacement.clone().into_shape(),
    )?;
    wire.set(&replacement, index)?;
    if let Some(bounds) = bounds {
        replace_box(boxes, &replacement, bounds);
    }
    Ok(())
}

fn take_box(boxes: &mut [EdgeBox], edge: &Edge) -> Option<BoundingBox2> {
    boxes
        .iter()
        .find(|entry| entry.edge.is_same(edge))
        .map(|entry| entry.bounds.clone())
}

pub(super) fn nearest_vertex(
    edge: &Edge,
    other: &Edge,
    parameter: f64,
    pcurve: &Handle<dyn Geom2dCurve>,
    face: &Face,
) -> Result<VertexCandidate, Failure> {
    let point = super::point_on_edge(edge, parameter, pcurve, face)?;
    let analysis = ShapeAnalysisEdge::new();
    let other_first = analysis.first_vertex(other);
    let other_last = analysis.last_vertex(other);
    let first_distance = point.distance(&BRepTool::pnt(&other_first)?);
    let last_distance = point.distance(&BRepTool::pnt(&other_last)?);
    let (vertex, distance) = if first_distance < last_distance {
        (other_first, first_distance)
    } else {
        (other_last, last_distance)
    };
    candidate_at(vertex, distance, edge, &analysis)
}

fn candidate_at(
    vertex: Vertex,
    distance: f64,
    edge: &Edge,
    analysis: &ShapeAnalysisEdge,
) -> Result<VertexCandidate, Failure> {
    let first = analysis.first_vertex(edge);
    let last = analysis.last_vertex(edge);
    let should_split = !vertex.is_same(&first) && !vertex.is_same(&last);
    let tolerance = (0.5 * distance * 1.00001).max(BRepTool::vertex_tolerance(&vertex)?);
    Ok(VertexCandidate {
        vertex,
        tolerance,
        should_split,
    })
}
