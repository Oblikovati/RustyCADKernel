//! The segment block of `FixIntersectingWires`: an overlap near an edge end cuts that
//! edge; otherwise a big overlap becomes one shared edge and a short one a shared vertex.
use super::super::aggregate::PairCurves;
use super::super::aggregate::segment::{SegmentPoints, segment_points};
use super::super::point_on_edge;
use super::cross_points::mean_point;
use super::{EdgePair, PairScan, Step, big_segment, native_max};
use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;

/// The oriented end vertices of an edge (`sae.FirstVertex`, `sae.LastVertex`) and their
/// points, read once (`PV1`, `PV2`).
pub(super) struct EdgeEnds {
    pub(super) vertices: [Vertex; 2],
    pub(super) points: [Point3; 2],
}

/// One overlap edge: its p-curve range `[a, b]`, the overlap parameters and points.
struct OverlapSide<'a> {
    edge: &'a Edge,
    range: (f64, f64),
    parameters: (f64, f64),
    points: &'a [Point3; 2],
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EndCut {
    Unchanged,
    Cut,
    Failed,
}

/// Repairs a bounded overlap segment of the pair.
pub(super) fn repair(
    scan: &mut PairScan<'_>,
    pair: &mut EdgePair,
    curves: &PairCurves,
) -> Result<Step, Failure> {
    if curves.intersections.nb_segments()? != 1 {
        return Ok(Step::NextSecond);
    }
    let segment = curves.intersections.segment(1)?;
    if !segment.has_first_point() || !segment.has_last_point() {
        return Ok(Step::NextSecond);
    }
    let points = segment_points(segment, &pair.first, &pair.second, scan.face, curves)?;
    let first_ends = edge_ends(&pair.first)?;
    let first = OverlapSide {
        edge: &pair.first,
        range: curves.ranges[0],
        parameters: points.first_parameters,
        points: &points.first,
    };
    let first_cut = cut_near_end(scan, &first, &first_ends)?;
    if first_cut == EndCut::Failed {
        return Ok(Step::NextSecond);
    }
    let second_ends = edge_ends(&pair.second)?;
    let second = OverlapSide {
        edge: &pair.second,
        range: curves.ranges[1],
        parameters: points.second_parameters,
        points: &points.second,
    };
    let second_cut = cut_near_end(scan, &second, &second_ends)?;
    if second_cut == EndCut::Failed {
        return Ok(Step::NextSecond);
    }
    if first_cut == EndCut::Cut || second_cut == EndCut::Cut {
        // The same pair again with the modified ranges.
        scan.modified = true;
        return Ok(Step::SameSecond);
    }
    if is_big(&points, curves) {
        return big_segment::repair(scan, pair, &points, [first_ends, second_ends]);
    }
    split_in_two(scan, pair, &points, curves)
}

/// The end vertices of `edge` and their points; `edge_ends(&edge)?.points[0]`.
pub(super) fn edge_ends(edge: &Edge) -> Result<EdgeEnds, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let vertices = [analysis.first_vertex(edge), analysis.last_vertex(edge)];
    let points = [BRepTool::pnt(&vertices[0])?, BRepTool::pnt(&vertices[1])?];
    Ok(EdgeEnds { vertices, points })
}

// The analysis of one edge: an end vertex near the whole overlap (in space or in
// parameter) takes the overlap; the edge is cut there and the vertex tolerance raised.
fn cut_near_end(
    scan: &mut PairScan<'_>,
    side: &OverlapSide<'_>,
    ends: &EdgeEnds,
) -> Result<EndCut, Failure> {
    let Some((vertex, tolerance)) = nearest_end(side, ends, *scan.max_tolerance) else {
        return Ok(EndCut::Unchanged);
    };
    let (a, b) = side.range;
    let (p1, p2) = side.parameters;
    let dista = (a - p1).abs() + (a - p2).abs();
    let distb = (b - p1).abs() + (b - p2).abs();
    let end = if dista > distb { a } else { b };
    let cut = if (end - p1).abs() > (end - p2).abs() {
        p2
    } else {
        p1
    };
    let mut cut_line = false;
    if !scan
        .tool
        .cut_edge(side.edge, end, cut, scan.face, &mut cut_line)?
    {
        return Ok(EndCut::Failed);
    }
    if tolerance > BRepTool::vertex_tolerance(&vertex)? {
        BRepBuilder::default().update_vertex_tolerance(&vertex, tolerance * 1.00001)?;
    }
    Ok(EndCut::Cut)
}

// The first end, then the last end when it is nearer; `pdist` measures from the
// parameter of that end (`a` for the first end of a forward edge).
fn nearest_end(
    side: &OverlapSide<'_>,
    ends: &EdgeEnds,
    max_tolerance: f64,
) -> Option<(Vertex, f64)> {
    let (a, b) = side.range;
    let (p1, p2) = side.parameters;
    let reversed = side.edge.orientation() == Orientation::Reversed;
    let mut chosen: Option<(Vertex, f64)> = None;
    for end in 0..2 {
        let distance = native_max(
            side.points[0].distance(&ends.points[end]),
            side.points[1].distance(&ends.points[end]),
        );
        let parameter = if (end == 0) != reversed { a } else { b };
        let pdist = native_max((parameter - p1).abs(), (parameter - p2).abs());
        let near = distance < max_tolerance || pdist < (b - a).abs() * 0.01;
        let better = chosen
            .as_ref()
            .is_none_or(|(_, tolerance)| distance < *tolerance);
        if near && better {
            chosen = Some((ends.vertices[end].clone(), distance));
        }
    }
    chosen
}

fn is_big(points: &SegmentPoints, curves: &PairCurves) -> bool {
    let (p11, p12) = points.first_parameters;
    let (p21, p22) = points.second_parameters;
    let (a1, b1) = curves.ranges[0];
    let (a2, b2) = curves.ranges[1];
    (p12 - p11).abs() > (b1 - a1).abs() / 2.0 || (p22 - p21).abs() > (b2 - a2).abs() / 2.0
}

// A short overlap: one vertex at the middle of the first edge's overlap splits edge2,
// then edge1, each trimmed to the overlap ends.
fn split_in_two(
    scan: &mut PairScan<'_>,
    pair: &EdgePair,
    points: &SegmentPoints,
    curves: &PairCurves,
) -> Result<Step, Failure> {
    let (p11, p12) = points.first_parameters;
    let (p21, p22) = points.second_parameters;
    let [pnt11, pnt12] = points.first;
    let [pnt21, pnt22] = points.second;
    let middle = mean_point(pnt11, pnt12);
    let pnt10 = point_on_edge(
        &pair.first,
        (p11 + p12) / 2.0,
        &curves.pcurves[0],
        scan.face,
    )?;
    let pnt20 = point_on_edge(
        &pair.second,
        (p21 + p22) / 2.0,
        &curves.pcurves[1],
        scan.face,
    )?;
    let first = native_max(pnt11.distance(&middle), pnt12.distance(&pnt10));
    let second = native_max(pnt21.distance(&middle), pnt22.distance(&pnt10));
    let tolerance = native_max(native_max(first, second), pnt10.distance(&pnt20)) * 1.00001;
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, pnt10, tolerance)?;
    *scan.max_tolerance = native_max(*scan.max_tolerance, tolerance);
    scan.modified = true;
    let second_split = scan.tool.split_edge2(
        scan.data[1],
        scan.face,
        pair.index[1],
        (p21, p22),
        &vertex,
        tolerance,
        scan.boxes[1],
    )?;
    scan.modifications += i32::from(second_split);
    if scan.tool.split_edge2(
        scan.data[0],
        scan.face,
        pair.index[0],
        (p11, p12),
        &vertex,
        tolerance,
        scan.boxes[0],
    )? {
        scan.modifications += 1;
        return Ok(Step::SameFirst);
    }
    Ok(if second_split {
        Step::SameSecond
    } else {
        Step::NextSecond
    })
}
