//! The big overlap of `FixIntersectingWires`: each edge is split at the two overlap ends
//! and the middle edge of the first wire replaces the middle edge of the second wire.
use super::super::aggregate::segment::SegmentPoints;
use super::super::boxes;
use super::cross_points::mean_point;
use super::segments::EdgeEnds;
use super::{EdgePair, PairScan, Step, native_max};
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildEdge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// One overlap end: its mean point (`P01`, `P02`), the vertex put there (`NewV1`,
/// `NewV2`), its tolerance and how many edge ends it matches (`akey1`, `akey2`).
struct OverlapEnd {
    point: Point3,
    vertex: Vertex,
    tolerance: f64,
    key: i32,
}

/// Splits both edges at the overlap ends and shares the first wire's middle edge.
pub(super) fn repair(
    scan: &mut PairScan<'_>,
    pair: &mut EdgePair,
    points: &SegmentPoints,
    ends: [EdgeEnds; 2],
) -> Result<Step, Failure> {
    let mut overlap = [
        overlap_end(points.first[0], points.second[0]),
        overlap_end(points.first[1], points.second[1]),
    ];
    if overlap[0].tolerance > *scan.max_tolerance || overlap[1].tolerance > *scan.max_tolerance {
        return Ok(Step::NextSecond);
    }
    scan.modified = true;
    for end in &mut overlap {
        match_edge_ends(end, &ends[0])?;
    }
    if overlap[0].key > 1 || overlap[1].key > 1 {
        return Ok(Step::NextSecond);
    }
    for end in &mut overlap {
        if end.key == 0 {
            BRepBuilder::default().make_vertex_at(&mut end.vertex, end.point, end.tolerance)?;
        }
    }
    let first_segment = split_at_ends(scan, (0, pair.index[0]), &overlap, points.first_parameters)?;
    let segment = scan.data[0].edge(first_segment)?;
    replace_second_ends(scan, pair, &mut overlap, &ends[1])?;
    let second_segment =
        split_at_ends(scan, (1, pair.index[1]), &overlap, points.second_parameters)?;
    share_segment(scan, segment, second_segment)?;
    Ok(Step::SameFirst)
}

fn overlap_end(first: Point3, second: Point3) -> OverlapEnd {
    let point = mean_point(first, second);
    let tolerance = native_max(first.distance(&point), second.distance(&point));
    OverlapEnd {
        point,
        vertex: Vertex::default(),
        tolerance: native_max(tolerance, precision::confusion()) * 1.00001,
        key: 0,
    }
}

// The analysis of P01 (or P02) against V1 then V2 of edge1: a matched end becomes the
// overlap vertex, its tolerance raised to the overlap tolerance.
fn match_edge_ends(end: &mut OverlapEnd, ends: &EdgeEnds) -> Result<(), Failure> {
    for (vertex, position) in ends.vertices.iter().zip(ends.points) {
        let tolerance = BRepTool::vertex_tolerance(vertex)?;
        if end.point.distance(&position) >= native_max(end.tolerance, tolerance) {
            continue;
        }
        end.vertex = vertex.clone();
        if end.tolerance > tolerance {
            BRepBuilder::default().update_vertex_tolerance(&end.vertex, end.tolerance)?;
        }
        end.key += 1;
    }
    Ok(())
}

// The splits of edge `side` at the overlap parameters; returns the index of the middle
// (segment) edge (`numseg1`, `numseg2`).
fn split_at_ends(
    scan: &mut PairScan<'_>,
    (side, index): (usize, i32),
    overlap: &[OverlapEnd; 2],
    (start, end): (f64, f64),
) -> Result<i32, Failure> {
    match (overlap[0].key, overlap[1].key) {
        (0, 0) => split_twice(scan, (side, index), overlap, (start, end)),
        (0, _) => Ok(index + i32::from(split(scan, side, index, start, &overlap[0])?)),
        (_, 0) => {
            split(scan, side, index, end, &overlap[1])?;
            Ok(index)
        }
        _ => Ok(index),
    }
}

// Both overlap ends are new vertices: split at the start, then at the end on the piece
// that holds it (the next edge when the end is outside the first piece's range).
fn split_twice(
    scan: &mut PairScan<'_>,
    (side, index): (usize, i32),
    overlap: &[OverlapEnd; 2],
    (start, end): (f64, f64),
) -> Result<i32, Failure> {
    let mut segment = index;
    let mut end_index = index;
    if split(scan, side, index, start, &overlap[0])? {
        // Only the second wire's split names the segment here; the first wire's
        // segment waits for the second split, as natively.
        segment = if side == 1 { index + 1 } else { segment };
        if end_outside(scan, side, index, end)? {
            end_index += 1;
        }
    }
    if split(scan, side, end_index, end, &overlap[1])? {
        segment = index + 1;
    }
    Ok(segment)
}

// `(a - p) * (b - p) > 0` on the p-curve range of the first piece.
fn end_outside(
    scan: &PairScan<'_>,
    side: usize,
    index: i32,
    parameter: f64,
) -> Result<bool, Failure> {
    let piece = scan.data[side].edge(index)?;
    let (_, a, b) = ShapeAnalysisEdge::new().pcurve_on_face(&piece, scan.face, false)?;
    Ok((a - parameter) * (b - parameter) > 0.0)
}

fn split(
    scan: &mut PairScan<'_>,
    side: usize,
    index: i32,
    parameter: f64,
    end: &OverlapEnd,
) -> Result<bool, Failure> {
    let done = scan.tool.split_edge1(
        scan.data[side],
        scan.face,
        index,
        parameter,
        &end.vertex,
        end.tolerance,
        scan.boxes[side],
    )?;
    scan.modifications += i32::from(done);
    Ok(done)
}

// The four tests of edge2's ends against P01 then P02, with the stale points PV12 and
// PV22: a match grows the overlap tolerance and puts the overlap vertex at that end.
fn replace_second_ends(
    scan: &mut PairScan<'_>,
    pair: &mut EdgePair,
    overlap: &mut [OverlapEnd; 2],
    ends: &EdgeEnds,
) -> Result<(), Failure> {
    let mut vertices = ends.vertices.clone();
    for end in overlap.iter_mut() {
        end.key = 0;
        for side in 0..2 {
            let distance = end.point.distance(&ends.points[side]);
            if distance >= end.tolerance {
                continue;
            }
            end.tolerance += distance;
            BRepBuilder::default().update_vertex_tolerance(&end.vertex, end.tolerance)?;
            vertices[side] = end.vertex.clone();
            replace_second_edge(scan, pair, &vertices)?;
            end.key = side as i32 + 1;
        }
    }
    Ok(())
}

// `CopyReplaceVertices(edge2, V12, V22)`, recorded in the context, the wire data and the
// boxes (with the box of the original edge2).
fn replace_second_edge(
    scan: &mut PairScan<'_>,
    pair: &mut EdgePair,
    vertices: &[Vertex; 2],
) -> Result<(), Failure> {
    let edge = ShapeBuildEdge.copy_replace_vertices(&pair.second, &vertices[0], &vertices[1])?;
    scan.tool.context().replace(
        &pair.second.clone().into_shape(),
        &edge.clone().into_shape(),
    )?;
    scan.data[1].set(&edge, pair.index[1])?;
    boxes::replace_box(scan.boxes[1], &edge, pair.second_box.clone());
    pair.second = edge;
    Ok(())
}

// The middle edge of the second wire takes the box of the first wire's middle edge and
// is replaced by it, reversed when their first vertices differ.
fn share_segment(scan: &mut PairScan<'_>, mut segment: Edge, index: i32) -> Result<(), Failure> {
    let target = scan.data[1].edge(index)?;
    let Some(bounds) = boxes::find_edge_box(scan.boxes[0], &segment) else {
        return Err(Failure::with_message(
            "Standard_NoSuchObject",
            &format!(
                "FixIntersectingWires: the segment edge at index {index} has no 2D box; expected an edge bound in the first wire's boxes"
            ),
        ));
    };
    boxes::replace_box(scan.boxes[1], &target, bounds);
    let analysis = ShapeAnalysisEdge::new();
    if !analysis
        .first_vertex(&segment)
        .is_same(&analysis.first_vertex(&target))
    {
        segment.reverse();
    }
    scan.tool
        .context()
        .replace(&target.clone().into_shape(), &segment.clone().into_shape())?;
    scan.data[1].set(&segment, index)
}
