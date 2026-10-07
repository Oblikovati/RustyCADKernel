//! The point block of `FixIntersectingWires`: one or two intersection points, the better
//! one chosen by `SelectIntPnt`, then the four native position cases in their order.
use super::super::aggregate::{PairCurves, points};
use super::super::point_on_edge;
use super::{EdgePair, PairScan, Step, native_max};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_res2d::{IntersectionPoint, Position};
use rustycad_tkmath::gp::Point3;

/// Repairs a point intersection of the pair; `None` when the native code falls through
/// to the segment block.
pub(super) fn repair(
    scan: &mut PairScan<'_>,
    pair: &mut EdgePair,
    curves: &PairCurves,
) -> Result<Option<Step>, Failure> {
    let count = curves.intersections.nb_points()?;
    if count == 0 || count >= 3 {
        return Ok(None);
    }
    let point = points::selected_point(&curves.intersections, count)?;
    let middle = [
        point.transition_of_first().position_on_curve() == Position::Middle,
        point.transition_of_second().position_on_curve() == Position::Middle,
    ];
    if middle == [true, true]
        && let Some(step) = split_both(scan, pair, curves, &point)?
    {
        return Ok(Some(step));
    }
    if middle == [true, false] && split_at_vertex(scan, pair, curves, point.param_on_first(), 0)? {
        return Ok(Some(Step::SameFirst));
    }
    if middle == [false, true] && split_at_vertex(scan, pair, curves, point.param_on_second(), 1)? {
        return Ok(Some(Step::SameSecond));
    }
    if middle == [false, false] && union_vertices(scan, pair)? {
        scan.replaced += 1;
    }
    Ok(None)
}

// Both positions in the middle: a vertex at the mean point splits edge2, then edge1.
fn split_both(
    scan: &mut PairScan<'_>,
    pair: &EdgePair,
    curves: &PairCurves,
    point: &IntersectionPoint,
) -> Result<Option<Step>, Failure> {
    let parameters = [point.param_on_first(), point.param_on_second()];
    let first = point_on_edge(&pair.first, parameters[0], &curves.pcurves[0], scan.face)?;
    let second = point_on_edge(&pair.second, parameters[1], &curves.pcurves[1], scan.face)?;
    let tolerance = native_max(
        (first.distance(&second) / 2.0) * 1.00001,
        precision::confusion(),
    );
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, mean_point(first, second), tolerance)?;
    *scan.max_tolerance = native_max(*scan.max_tolerance, tolerance);
    let second_split = scan.tool.split_edge1(
        scan.data[1],
        scan.face,
        pair.index[1],
        parameters[1],
        &vertex,
        tolerance,
        scan.boxes[1],
    )?;
    scan.modifications += i32::from(second_split);
    if scan.tool.split_edge1(
        scan.data[0],
        scan.face,
        pair.index[0],
        parameters[0],
        &vertex,
        tolerance,
        scan.boxes[0],
    )? {
        scan.modifications += 1;
        return Ok(Some(Step::SameFirst));
    }
    Ok(second_split.then_some(Step::SameSecond))
}

// FindVertAndSplitEdge with `aTmpKey = true`: edge `side` is split at the nearer vertex
// of the other edge. Its own `num--` and the loop step cancel out.
fn split_at_vertex(
    scan: &mut PairScan<'_>,
    pair: &EdgePair,
    curves: &PairCurves,
    parameter: f64,
    side: usize,
) -> Result<bool, Failure> {
    let (edge, other) = if side == 0 {
        (&pair.first, &pair.second)
    } else {
        (&pair.second, &pair.first)
    };
    let mut index = pair.index[side];
    let split = scan.tool.find_vertex_and_split_edge(
        parameter,
        edge,
        other,
        &curves.pcurves[side],
        scan.max_tolerance,
        &mut index,
        scan.data[side],
        scan.face,
        scan.boxes[side],
        true,
    )?;
    scan.modifications += i32::from(split);
    Ok(split)
}

// UnionVertexes on the second wire; edge2 takes the replaced edge.
fn union_vertices(scan: &mut PairScan<'_>, pair: &mut EdgePair) -> Result<bool, Failure> {
    scan.tool.union_vertices(
        scan.data[1],
        &pair.first,
        &mut pair.second,
        pair.index[1],
        scan.boxes[1],
        &pair.second_box,
    )
}

/// `gp_Pnt((a.X() + b.X()) / 2, ...)`.
pub(super) fn mean_point(first: Point3, second: Point3) -> Point3 {
    Point3::new(
        (first.x() + second.x()) / 2.0,
        (first.y() + second.y()) / 2.0,
        (first.z() + second.z()) / 2.0,
    )
}
