//! Point-intersection repair helpers for `ShapeFix_IntersectionTool::FixSelfIntersectWire`.
use super::super::{ShapeFixIntersectionTool, boxes, point_on_edge};
use super::INTERSECTION_TOLERANCE;
use super::{PairAction, PairCurves, RepairCounts};
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_res2d::{Domain, IntersectionPoint, Position};
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point3;

pub(super) fn pair_curves(
    first: &Edge,
    second: &Edge,
    face: &Face,
) -> Result<Option<PairCurves>, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let (first_curve, first_start, first_end) = analysis.pcurve_on_face(first, face, false)?;
    if first_curve.is_null() {
        return Ok(None);
    }
    let (second_curve, second_start, second_end) = analysis.pcurve_on_face(second, face, false)?;
    if second_curve.is_null() {
        return Ok(None);
    }
    let first_adaptor = Geom2dAdaptorCurve::with_curve(&first_curve)?;
    let second_adaptor = Geom2dAdaptorCurve::with_curve(&second_curve)?;
    let first_domain = bounded_domain(&first_curve, (first_start, first_end))?;
    let second_domain = bounded_domain(&second_curve, (second_start, second_end))?;
    let intersections = GInter::with_domains(
        &first_adaptor,
        &first_domain,
        &second_adaptor,
        &second_domain,
        INTERSECTION_TOLERANCE,
        INTERSECTION_TOLERANCE,
    )?;
    Ok(Some(PairCurves {
        pcurves: [first_curve, second_curve],
        ranges: [(first_start, first_end), (second_start, second_end)],
        intersections,
    }))
}

fn bounded_domain(
    curve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    range: (f64, f64),
) -> Result<Domain, Failure> {
    let first = curve.value(range.0)?;
    let last = curve.value(range.1)?;
    Ok(Domain::bounded(
        first,
        range.0,
        INTERSECTION_TOLERANCE,
        last,
        range.1,
        INTERSECTION_TOLERANCE,
    ))
}

pub(super) fn selected_point(
    intersections: &GInter,
    count: i32,
) -> Result<IntersectionPoint, Failure> {
    let mut best = intersections.point(1)?;
    let mut best_score = middle_score(&best);
    for index in 2..=count {
        let candidate = intersections.point(index)?;
        let score = middle_score(&candidate);
        if score > best_score {
            best = candidate;
            best_score = score;
        }
    }
    Ok(best)
}

fn middle_score(point: &IntersectionPoint) -> u8 {
    u8::from(point.transition_of_first().position_on_curve() == Position::Middle)
        + 2 * u8::from(point.transition_of_second().position_on_curve() == Position::Middle)
}

pub(super) fn apply_pair_action(action: PairAction, first: &mut i32, second: &mut i32) {
    match action {
        PairAction::Next => *second += 1,
        PairAction::RetryFirst => *first -= 1,
        PairAction::RetrySecond => *second -= 1,
        PairAction::Abort => unreachable!("aborted actions return before index updates"),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn repair_point(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    first_index: i32,
    second_index: i32,
    first: &Edge,
    second: &Edge,
    second_box: &BoundingBox2,
    curves: &PairCurves,
    point: IntersectionPoint,
    max_vertex_tolerance: &mut f64,
    counts: &mut RepairCounts<'_>,
    boxes: &mut Vec<boxes::EdgeBox>,
) -> Result<PairAction, Failure> {
    let transitions = [point.transition_of_first(), point.transition_of_second()];
    match (
        transitions[0].position_on_curve(),
        transitions[1].position_on_curve(),
    ) {
        (Position::Middle, Position::Middle) => repair_middle_crossing(
            tool,
            wire,
            face,
            first_index,
            second_index,
            first,
            second,
            second_box,
            curves,
            point,
            max_vertex_tolerance,
            counts,
            boxes,
        ),
        (Position::Middle, _) => repair_using_other_edge_vertex(
            tool,
            wire,
            face,
            first_index,
            first,
            second,
            &curves.pcurves[0],
            point.param_on_first(),
            PairAction::RetryFirst,
            max_vertex_tolerance,
            counts,
            boxes,
        ),
        (_, Position::Middle) => repair_using_other_edge_vertex(
            tool,
            wire,
            face,
            second_index,
            second,
            first,
            &curves.pcurves[1],
            point.param_on_second(),
            PairAction::RetrySecond,
            max_vertex_tolerance,
            counts,
            boxes,
        ),
        _ => repair_union(
            tool,
            wire,
            first,
            second,
            second_index,
            second_box,
            counts,
            boxes,
        ),
    }
}

#[allow(clippy::too_many_arguments)] // Mirrors native vertex union inputs.
fn repair_union(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    first: &Edge,
    second: &Edge,
    second_index: i32,
    second_box: &BoundingBox2,
    counts: &mut RepairCounts<'_>,
    boxes: &mut Vec<boxes::EdgeBox>,
) -> Result<PairAction, Failure> {
    let mut replacement = second.clone();
    let unioned = tool.union_vertices(
        wire,
        first,
        &mut replacement,
        second_index,
        boxes,
        second_box,
    )?;
    if unioned {
        *counts.replaced += 1;
    }
    Ok(PairAction::Next)
}

#[allow(clippy::too_many_arguments)]
fn repair_middle_crossing(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    first_index: i32,
    second_index: i32,
    first: &Edge,
    second: &Edge,
    second_box: &BoundingBox2,
    curves: &PairCurves,
    point: IntersectionPoint,
    max_vertex_tolerance: &mut f64,
    counts: &mut RepairCounts<'_>,
    boxes: &mut Vec<boxes::EdgeBox>,
) -> Result<PairAction, Failure> {
    let first_point = point_on_edge(first, point.param_on_first(), &curves.pcurves[0], face)?;
    let second_point = point_on_edge(second, point.param_on_second(), &curves.pcurves[1], face)?;
    let mut shared_vertex = Vertex::default();
    let mut shared_tolerance = 0.0;
    let first_adjusted = adjust_intersection_endpoint(
        tool,
        first,
        point.param_on_first(),
        &first_point,
        *max_vertex_tolerance,
        &mut shared_vertex,
        &mut shared_tolerance,
        face,
        counts,
    )?;
    let second_adjusted = adjust_intersection_endpoint(
        tool,
        second,
        point.param_on_second(),
        &second_point,
        *max_vertex_tolerance,
        &mut shared_vertex,
        &mut shared_tolerance,
        face,
        counts,
    )?;
    if first_adjusted
        && !second_adjusted
        && tool.split_edge1(
            wire,
            face,
            second_index,
            point.param_on_second(),
            &shared_vertex,
            shared_tolerance,
            boxes,
        )?
    {
        *counts.split += 1;
        return Ok(PairAction::RetrySecond);
    }
    if !first_adjusted
        && second_adjusted
        && tool.split_edge1(
            wire,
            face,
            first_index,
            point.param_on_first(),
            &shared_vertex,
            shared_tolerance,
            boxes,
        )?
    {
        *counts.split += 1;
        return Ok(PairAction::RetryFirst);
    }
    if first_adjusted || second_adjusted {
        return Ok(PairAction::Next);
    }
    let midpoint = midpoint(first_point, second_point);
    let tolerance = (0.5 * first_point.distance(&second_point) * 1.00001)
        .max(rustycad_tkernel::precision::confusion());
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, midpoint, tolerance)?;
    *max_vertex_tolerance = max_vertex_tolerance.max(tolerance);
    let second_split = tool.split_edge1(
        wire,
        face,
        second_index,
        point.param_on_second(),
        &vertex,
        tolerance,
        boxes,
    )?;
    if second_split {
        *counts.split += 1;
    }
    if tool.split_edge1(
        wire,
        face,
        first_index,
        point.param_on_first(),
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
    let _ = second_box;
    Ok(PairAction::Next)
}

#[allow(clippy::too_many_arguments)]
fn adjust_intersection_endpoint(
    tool: &ShapeFixIntersectionTool,
    edge: &Edge,
    parameter: f64,
    intersection: &Point3,
    max_vertex_tolerance: f64,
    selected_vertex: &mut Vertex,
    selected_tolerance: &mut f64,
    face: &Face,
    counts: &mut RepairCounts<'_>,
) -> Result<bool, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let first = analysis.first_vertex(edge);
    let last = analysis.last_vertex(edge);
    let first_distance = intersection.distance(&BRepTool::pnt(&first)?);
    let last_distance = intersection.distance(&BRepTool::pnt(&last)?);
    let nearest_distance = first_distance.min(last_distance);
    if first_distance == last_distance || nearest_distance >= max_vertex_tolerance {
        return Ok(false);
    }
    let (vertex, distance) = if first_distance < last_distance {
        (&first, first_distance)
    } else {
        (&last, last_distance)
    };
    let tolerance = (distance * 1.00001).max(BRepTool::vertex_tolerance(vertex)?);
    BRepBuilder::default().update_vertex_tolerance(vertex, tolerance)?;
    *selected_vertex = vertex.clone();
    *selected_tolerance = tolerance;
    let (_, range_first, range_last) =
        ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    let first_gap = (range_first - parameter).abs();
    let last_gap = (range_last - parameter).abs();
    let endpoint = if first_gap > last_gap {
        range_first
    } else {
        range_last
    };
    let mut cut_line = false;
    let was_cut = tool.cut_edge(edge, endpoint, parameter, face, &mut cut_line)?;
    if was_cut {
        *counts.cut += 1;
    }
    Ok(was_cut || nearest_distance < rustycad_tkernel::precision::confusion())
}

#[allow(clippy::too_many_arguments)] // Mirrors native FindVertAndSplitEdge inputs.
fn repair_using_other_edge_vertex(
    tool: &ShapeFixIntersectionTool,
    wire: &ShapeExtendWireData,
    face: &Face,
    edge_index: i32,
    edge: &Edge,
    other: &Edge,
    pcurve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    parameter: f64,
    retry: PairAction,
    max_vertex_tolerance: &mut f64,
    counts: &mut RepairCounts<'_>,
    boxes: &mut Vec<boxes::EdgeBox>,
) -> Result<PairAction, Failure> {
    let mut index = edge_index;
    if tool.find_vertex_and_split_edge(
        parameter,
        edge,
        other,
        pcurve,
        max_vertex_tolerance,
        &mut index,
        wire,
        face,
        boxes,
        false,
    )? {
        *counts.split += 1;
        return Ok(retry);
    }
    Ok(PairAction::Next)
}

fn midpoint(first: Point3, second: Point3) -> Point3 {
    Point3::new(
        (first.x() + second.x()) / 2.0,
        (first.y() + second.y()) / 2.0,
        (first.z() + second.z()) / 2.0,
    )
}
