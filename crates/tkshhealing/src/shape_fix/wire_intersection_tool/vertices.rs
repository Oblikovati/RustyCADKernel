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
    /// The discarded vertex is the last one of the second edge.
    discard_last: bool,
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
        discard_last: second_index == 1,
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
    // Only the vertex slot of the discarded vertex changes, even on a closed edge.
    let (first, last) = edge_vertices(second);
    let replacement = if union.discard_last {
        ShapeBuildEdge.copy_replace_vertices(second, &first, &union.keep)?
    } else {
        ShapeBuildEdge.copy_replace_vertices(second, &union.keep, &last)?
    };
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

// Both neighbours are read before either changes; each of the four native tests copies
// the original neighbour, so a later match overrides an earlier one.
fn replace_neighbor_vertices(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    index: i32,
    union: &VertexUnion,
    boxes: &mut Vec<EdgeBox>,
) -> Result<(), Failure> {
    let count = wire.nb_edges();
    let previous = if index > 1 { index - 1 } else { count };
    let next = if index < count { index + 1 } else { 1 };
    let neighbors = [(wire.edge(previous)?, previous), (wire.edge(next)?, next)];
    for (source, neighbor_index) in neighbors {
        let (first, last) = edge_vertices(&source);
        if first.is_same(&union.discard) {
            let replacement = ShapeBuildEdge.copy_replace_vertices(&source, &union.keep, &last)?;
            replace_neighbor_edge(tool, wire, (&source, neighbor_index), &replacement, boxes)?;
        }
        if last.is_same(&union.discard) {
            let replacement = ShapeBuildEdge.copy_replace_vertices(&source, &first, &union.keep)?;
            replace_neighbor_edge(tool, wire, (&source, neighbor_index), &replacement, boxes)?;
        }
    }
    Ok(())
}

fn replace_neighbor_edge(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    (source, index): (&Edge, i32),
    replacement: &Edge,
    boxes: &mut Vec<EdgeBox>,
) -> Result<(), Failure> {
    if let Some(bounds) = take_box(boxes, source) {
        replace_box(boxes, replacement, bounds);
    }
    tool.context.replace(
        &source.clone().into_shape(),
        &replacement.clone().into_shape(),
    )?;
    wire.set(replacement, index)
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
