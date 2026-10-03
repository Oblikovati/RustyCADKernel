//! Full-coverage long-overlap reconstruction from `FixIntersectingWires`.
use super::super::boxes::EdgeBox;
use super::{PairEffect, ShapeFixIntersectionTool, WireRecord};
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_extend::ShapeExtendWireData;
use crate::shape_fix::wire::wire_intersection_tool::aggregate::segment::SegmentPoints;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

struct OverlapVertices {
    start: Vertex,
    end: Vertex,
    tolerances: (f64, f64),
}

#[allow(clippy::too_many_arguments)]
pub(super) fn replace_full_second_overlap(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    first: &mut WireRecord,
    second: &mut WireRecord,
    first_index: i32,
    second_index: i32,
    edge2: &Edge,
    points: &SegmentPoints,
    max_tolerance: &mut f64,
) -> Result<PairEffect, Failure> {
    let second_range = BRepTool::range_on_face(edge2, face)?;
    if !range_matches(second_range, points.second_parameters) {
        return Ok(PairEffect::None);
    }
    let Some(vertices) = overlap_vertices(points, *max_tolerance)? else {
        return Ok(PairEffect::None);
    };
    *max_tolerance = max_tolerance
        .max(vertices.tolerances.0)
        .max(vertices.tolerances.1);
    if !split_source_overlap(
        tool,
        face,
        first,
        first_index,
        points,
        &vertices.start,
        &vertices.end,
        vertices.tolerances.0,
        vertices.tolerances.1,
    )? {
        return Ok(PairEffect::None);
    }
    let shared = find_middle_edge(&first.data, face, points.first_parameters)?;
    replace_second_edge(tool, second, second_index, edge2, &shared, &first.boxes)?;
    Ok(PairEffect::Replaced)
}

fn overlap_vertices(
    points: &SegmentPoints,
    maximum: f64,
) -> Result<Option<OverlapVertices>, Failure> {
    let (start_point, start_tolerance) = overlap_vertex(points.first[0], points.second[0]);
    let (end_point, end_tolerance) = overlap_vertex(points.first[1], points.second[1]);
    if start_tolerance > maximum || end_tolerance > maximum {
        return Ok(None);
    }
    let start = make_overlap_vertex(start_point, start_tolerance)?;
    let end = make_overlap_vertex(end_point, end_tolerance)?;
    Ok(Some(OverlapVertices {
        start,
        end,
        tolerances: (start_tolerance, end_tolerance),
    }))
}

fn range_matches(range: (f64, f64), parameters: (f64, f64)) -> bool {
    let tolerance = 1.0e-7;
    (range.0 - parameters.0).abs() <= tolerance && (range.1 - parameters.1).abs() <= tolerance
        || (range.0 - parameters.1).abs() <= tolerance
            && (range.1 - parameters.0).abs() <= tolerance
}

fn overlap_vertex(first: Point3, second: Point3) -> (Point3, f64) {
    let point = midpoint(first, second);
    let tolerance = first
        .distance(&point)
        .max(second.distance(&point))
        .max(1.0e-7)
        * 1.00001;
    (point, tolerance)
}

fn make_overlap_vertex(point: Point3, tolerance: f64) -> Result<Vertex, Failure> {
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, point, tolerance)?;
    Ok(vertex)
}

fn midpoint(first: Point3, second: Point3) -> Point3 {
    Point3::new(
        0.5 * (first.x() + second.x()),
        0.5 * (first.y() + second.y()),
        0.5 * (first.z() + second.z()),
    )
}

#[allow(clippy::too_many_arguments)]
fn split_source_overlap(
    tool: &ShapeFixIntersectionTool,
    face: &Face,
    wire: &mut WireRecord,
    index: i32,
    points: &SegmentPoints,
    start: &Vertex,
    end: &Vertex,
    start_tolerance: f64,
    end_tolerance: f64,
) -> Result<bool, Failure> {
    if !tool.split_edge1(
        &wire.data,
        face,
        index,
        points.first_parameters.0,
        start,
        start_tolerance,
        &mut wire.boxes,
    )? {
        return Ok(false);
    }
    let Some(end_index) = edge_containing_parameter(&wire.data, face, points.first_parameters.1)?
    else {
        return Ok(false);
    };
    if !tool.split_edge1(
        &wire.data,
        face,
        end_index,
        points.first_parameters.1,
        end,
        end_tolerance,
        &mut wire.boxes,
    )? {
        return Ok(false);
    }
    Ok(true)
}

fn edge_containing_parameter(
    wire: &ShapeExtendWireData,
    face: &Face,
    parameter: f64,
) -> Result<Option<i32>, Failure> {
    for index in 1..=wire.nb_edges() {
        let range = BRepTool::range_on_face(&wire.edge(index)?, face)?;
        if parameter >= range.0.min(range.1) - 1.0e-7 && parameter <= range.0.max(range.1) + 1.0e-7
        {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn find_middle_edge(
    wire: &ShapeExtendWireData,
    face: &Face,
    parameters: (f64, f64),
) -> Result<Edge, Failure> {
    let middle = 0.5 * (parameters.0 + parameters.1);
    let index = edge_containing_parameter(wire, face, middle)?
        .ok_or_else(|| Failure::new("long overlap middle segment was not created"))?;
    wire.edge(index)
}

fn replace_second_edge(
    tool: &ShapeFixIntersectionTool,
    wire: &mut WireRecord,
    index: i32,
    source: &Edge,
    shared: &Edge,
    first_boxes: &[EdgeBox],
) -> Result<(), Failure> {
    let mut replacement = shared.clone();
    orient_like(&mut replacement, source);
    tool.context().replace(
        &source.clone().into_shape(),
        &replacement.clone().into_shape(),
    )?;
    wire.data.set(&replacement, index)?;
    wire.boxes.retain(|entry| !entry.edge.is_same(source));
    if let Some(entry) = first_boxes.iter().find(|entry| entry.edge.is_same(shared)) {
        wire.boxes.push(EdgeBox {
            edge: replacement,
            bounds: entry.bounds.clone(),
        });
    }
    Ok(())
}

fn orient_like(replacement: &mut Edge, source: &Edge) {
    let analysis = ShapeAnalysisEdge::new();
    if !analysis
        .first_vertex(replacement)
        .is_same(&analysis.first_vertex(source))
    {
        replacement.reverse();
    }
}
