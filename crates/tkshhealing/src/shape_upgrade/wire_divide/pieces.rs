//! Build each segment while preserving small-piece and context replacement state.
use super::*;
use super::{perform::WireSupport, segments::DividedCurves, vertices::EdgeVertices};
use crate::shape_build::ShapeBuildEdge;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::{geom::GeomCurve, top_abs::Orientation};
use rustycad_tkmath::gp::Point3;
mod construction;
mod geometry;
mod non_manifold;
mod ranges;
mod small;
struct PieceGeometry {
    spatial: Handle<dyn GeomCurve>,
    planar: Handle<dyn Geom2dCurve>,
    reversed: Handle<dyn Geom2dCurve>,
}
struct PieceCursor {
    vertices: EdgeVertices,
    saved_index: usize,
    saved_first: f64,
    small_count: usize,
}
pub(super) fn divide<T: WireDivider + ?Sized>(
    tool: &T,
    support: &mut WireSupport,
    edge: &Edge,
    curves: &DividedCurves,
    vertices: EdgeVertices,
    result: &mut Wire,
) -> Result<(), Failure> {
    let mut replacement = Wire::default();
    BRepBuilder::default().make_wire(&mut replacement);
    let mut cursor = prepare_cursor(vertices)?;
    let small = prepare_small(tool, support, edge, curves)?;
    let mut count = 0;
    for index in 1..=curves.count() {
        let Some(piece) = build_piece(tool, support, edge, curves, &small, &mut cursor, index)?
        else {
            continue;
        };
        BRepBuilder::default().add(&mut replacement, &piece)?;
        BRepBuilder::default().add(result, &piece)?;
        count += 1;
    }
    if count == 0 {
        return support.context.remove(edge);
    }
    replacement.set_closed(BRepTool::is_closed(&replacement)?)?;
    support.context.replace(edge, &replacement)
}
fn prepare_cursor(vertices: EdgeVertices) -> Result<PieceCursor, Failure> {
    let mut cursor = PieceCursor {
        vertices,
        saved_index: 0,
        saved_first: 0.,
        small_count: 0,
    };
    cursor.vertices.first_point = BRepTool::pnt(&cursor.vertices.first)?;
    Ok(cursor)
}
fn prepare_small<T: WireDivider + ?Sized>(
    tool: &T,
    support: &WireSupport,
    edge: &Edge,
    curves: &DividedCurves,
) -> Result<Handle<ShapeUpgradeFixSmallCurves>, Failure> {
    let small = tool.wire_divide_data().with(|fields| fields.small.clone());
    state::require(&small, "small curve tool")?;
    small.set_min_tolerance(tool.wire_divide_base().min_tolerance());
    small.init(edge, &support.face);
    small.set_split_curve_3d_tool(&curves.split3);
    small.set_split_curve_2d_tool(&curves.split2);
    small.set_precision(tool.wire_divide_base().min_tolerance());
    Ok(small)
}
fn build_piece<T: WireDivider + ?Sized>(
    tool: &T,
    support: &mut WireSupport,
    edge: &Edge,
    curves: &DividedCurves,
    small: &Handle<ShapeUpgradeFixSmallCurves>,
    cursor: &mut PieceCursor,
    index: usize,
) -> Result<Option<Edge>, Failure> {
    let mut geometry = geometry::at(curves, index)?;
    let Some(mut vertex) =
        small::next_vertex(tool, support, curves, small, cursor, &mut geometry, index)?
    else {
        return Ok(None);
    };
    let mut next = construction::new_edge(edge, cursor, &mut vertex, &geometry)?;
    if cursor.saved_index == 0 {
        cursor.saved_index = index;
    }
    let bounds = ranges::set_ranges(support, &next, curves, &geometry, cursor.saved_index, index)?;
    cursor.saved_index = 0;
    non_manifold::attach(support, edge, &mut next, cursor, &vertex, bounds)?;
    cursor.vertices.first = vertex;
    Ok(Some(next))
}
