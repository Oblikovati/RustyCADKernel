// Rust translation of BRepPrim_OneAxis.cxx (AxisEdge, StartEdge, EndEdge, StartTopEdge,
// StartBottomEdge, EndTopEdge, EndBottomEdge, TopEdge, BottomEdge) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edges of `BRepPrim_OneAxis`, each built once.
use super::BRepPrimOneAxis;
use super::one_axis::{
    EAXIS, EBOTEND, EBOTSTART, EBOTTOM, EEND, ESTART, ETOP, ETOPEND, ETOPSTART, are_heights_equal,
    axis_point, is_height_inverted, one_axis_domain_error,
};
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2, Circle3, Line3};

/// The member that builds a vertex.
type VertexMaker<T> = fn(&mut T) -> Result<Vertex, Failure>;

/// The built edge in slot `slot`, or `None` when it is not built yet.
fn built_edge<T: BRepPrimOneAxis + ?Sized>(prim: &T, slot: usize) -> Option<Edge> {
    let base = prim.one_axis_base();
    base.edges_built[slot].then(|| base.my_edges[slot].clone())
}

/// Completes `edge`, stores it in slot `slot` and marks it built.
fn store_edge<T: BRepPrimOneAxis + ?Sized>(prim: &mut T, slot: usize, edge: Edge) -> Edge {
    let base = prim.one_axis_base_mut();
    base.my_builder.complete_edge(&edge);
    base.my_edges[slot] = edge.clone();
    base.edges_built[slot] = true;
    edge
}

/// `AxisEdge()`: the line of the axis between the axis vertices that exist (the edge
/// runs from the lower to the higher one), or a degenerated edge at the common height
/// when the heights are equal; Standard_DomainError without sides or with a closed
/// meridian.
pub(crate) fn axis_edge<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Edge, Failure> {
    if let Some(edge) = built_edge(prim, EAXIS) {
        return Ok(edge);
    }
    if !prim.has_sides() {
        return Err(one_axis_domain_error("AxisEdge", "no sides"));
    }
    if prim.meridian_closed()? {
        return Err(one_axis_domain_error("AxisEdge", "closed"));
    }
    let builder = prim.one_axis_base().my_builder;
    let mut edge = Edge::default();
    if are_heights_equal(prim)? {
        let height = prim.meridian_value(prim.vmax())?.y();
        builder.make_degenerated_edge(&mut edge)?;
        let vertex = prim.axis_top_vertex()?;
        builder.add_edge_vertex_closed(&mut edge, &vertex, height, height)?;
        return Ok(store_edge(prim, EAXIS, edge));
    }
    builder.make_edge_line(&mut edge, Line3::from_axis(prim.axes().axis()))?;
    let is_inverted = is_height_inverted(prim)?;
    let y_max = prim.meridian_value(prim.vmax())?.y();
    let y_min = prim.meridian_value(prim.vmin())?.y();
    if !prim.vmax_infinite() {
        let vertex = prim.axis_top_vertex()?;
        builder.add_edge_vertex(&mut edge, &vertex, y_max, is_inverted)?;
    }
    if !prim.vmin_infinite() {
        let vertex = prim.axis_bottom_vertex()?;
        builder.add_edge_vertex(&mut edge, &vertex, y_min, !is_inverted)?;
    }
    Ok(store_edge(prim, EAXIS, edge))
}

/// `StartEdge()` and `EndEdge()`: the meridian edge at `angle` with its vertices (one
/// vertex at both ends of a closed meridian), or the other meridian edge when there
/// are no sides and it is built. `slots` is `[this, other]` and `makers` the top and
/// bottom vertex members of this side.
fn meridian_edge<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
    slots: [usize; 2],
    angle: f64,
    makers: [VertexMaker<T>; 2],
) -> Result<Edge, Failure> {
    if let Some(edge) = built_edge(prim, slots[0]) {
        return Ok(edge);
    }
    if !prim.has_sides() && prim.one_axis_base().edges_built[slots[1]] {
        let other = prim.one_axis_base().my_edges[slots[1]].clone();
        return Ok(store_edge(prim, slots[0], other));
    }
    let builder = prim.one_axis_base().my_builder;
    let offset = prim.one_axis_base().my_meridian_offset;
    let (vmin, vmax) = (prim.vmin(), prim.vmax());
    let mut edge = prim.make_empty_meridian_edge(angle)?;
    if prim.meridian_closed()? {
        let vertex = makers[0](prim)?;
        builder.add_edge_vertex_closed(&mut edge, &vertex, vmin + offset, vmax + offset)?;
        return Ok(store_edge(prim, slots[0], edge));
    }
    if !prim.vmax_infinite() {
        let vertex = makers[0](prim)?;
        builder.add_edge_vertex(&mut edge, &vertex, vmax + offset, false)?;
    }
    if !prim.vmin_infinite() {
        let vertex = makers[1](prim)?;
        builder.add_edge_vertex(&mut edge, &vertex, vmin + offset, true)?;
    }
    Ok(store_edge(prim, slots[0], edge))
}

/// `StartEdge()`: the meridian edge at angle 0.
pub(crate) fn start_edge<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Edge, Failure> {
    meridian_edge(
        prim,
        [ESTART, EEND],
        0.0,
        [T::top_start_vertex, T::bottom_start_vertex],
    )
}

/// `EndEdge()`: the meridian edge at Angle.
pub(crate) fn end_edge<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Edge, Failure> {
    let angle = prim.angle();
    meridian_edge(
        prim,
        [EEND, ESTART],
        angle,
        [T::top_end_vertex, T::bottom_end_vertex],
    )
}

/// The line of a side edge: from the axis at the meridian height of `v`, along the X
/// direction, rotated by Angle around the axis for an end edge.
fn side_line<T: BRepPrimOneAxis + ?Sized>(
    prim: &T,
    v: f64,
    rotated: bool,
) -> Result<Line3, Failure> {
    let axes: Axis2 = prim.axes();
    let point = axis_point(axes, prim.meridian_value(v)?.y());
    let mut line = Line3::new(point, axes.x_direction());
    if rotated {
        line.rotate(axes.axis(), prim.angle())?;
    }
    Ok(line)
}

/// A side edge between a cap and a side face: the line from the axis vertex
/// (parameter 0) to the meridian vertex (parameter the meridian radius). `axis_first`
/// adds the axis vertex first (forward) as the top edges do; the bottom start edge
/// adds the meridian vertex first (reversed).
struct SideEdge<T: ?Sized> {
    slot: usize,
    member: &'static str,
    top: bool,
    rotated: bool,
    axis_first: bool,
    rim: VertexMaker<T>,
}

/// Builds the side edge `side`; Standard_DomainError without its cap or without sides.
fn side_edge<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
    side: SideEdge<T>,
) -> Result<Edge, Failure> {
    if let Some(edge) = built_edge(prim, side.slot) {
        return Ok(edge);
    }
    let has_cap = if side.top {
        prim.has_top()?
    } else {
        prim.has_bottom()?
    };
    if !has_cap || !prim.has_sides() {
        return Err(one_axis_domain_error(side.member, "no sides or no cap"));
    }
    let builder = prim.one_axis_base().my_builder;
    let v = if side.top { prim.vmax() } else { prim.vmin() };
    let mut edge = Edge::default();
    builder.make_edge_line(&mut edge, side_line(prim, v, side.rotated)?)?;
    let axis_vertex: VertexMaker<T> = if side.top {
        T::axis_top_vertex
    } else {
        T::axis_bottom_vertex
    };
    if side.axis_first {
        let vertex = axis_vertex(prim)?;
        builder.add_edge_vertex(&mut edge, &vertex, 0.0, true)?;
    }
    let rim = (side.rim)(prim)?;
    builder.add_edge_vertex(&mut edge, &rim, prim.meridian_value(v)?.x(), false)?;
    if !side.axis_first {
        let vertex = axis_vertex(prim)?;
        builder.add_edge_vertex(&mut edge, &vertex, 0.0, true)?;
    }
    Ok(store_edge(prim, side.slot, edge))
}

/// `StartTopEdge()`: the line between the start and top faces.
pub(crate) fn start_top_edge<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Edge, Failure> {
    side_edge(
        prim,
        SideEdge {
            slot: ETOPSTART,
            member: "StartTopEdge",
            top: true,
            rotated: false,
            axis_first: true,
            rim: T::top_start_vertex,
        },
    )
}

/// `StartBottomEdge()`: the line between the start and bottom faces.
pub(crate) fn start_bottom_edge<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
) -> Result<Edge, Failure> {
    side_edge(
        prim,
        SideEdge {
            slot: EBOTSTART,
            member: "StartBottomEdge",
            top: false,
            rotated: false,
            axis_first: false,
            rim: T::bottom_start_vertex,
        },
    )
}

/// `EndTopEdge()`: the line between the end and top faces.
pub(crate) fn end_top_edge<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Edge, Failure> {
    side_edge(
        prim,
        SideEdge {
            slot: ETOPEND,
            member: "EndTopEdge",
            top: true,
            rotated: true,
            axis_first: true,
            rim: T::top_end_vertex,
        },
    )
}

/// `EndBottomEdge()`: the line between the end and bottom faces.
pub(crate) fn end_bottom_edge<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Edge, Failure> {
    side_edge(
        prim,
        SideEdge {
            slot: EBOTEND,
            member: "EndBottomEdge",
            top: false,
            rotated: true,
            axis_first: true,
            rim: T::bottom_end_vertex,
        },
    )
}

/// `TopEdge()` and `BottomEdge()`: the circle at the meridian point of `v` (a
/// degenerated edge on the axis) with its vertices (one closed vertex without sides),
/// or the other circle of a closed meridian when it is built. `slots` is
/// `[this, other]` and `makers` the end and start vertex members of this cap.
fn circle_edge<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
    slots: [usize; 2],
    v: f64,
    makers: [VertexMaker<T>; 2],
) -> Result<Edge, Failure> {
    if let Some(edge) = built_edge(prim, slots[0]) {
        return Ok(edge);
    }
    if prim.meridian_closed()? && prim.one_axis_base().edges_built[slots[1]] {
        let other = prim.one_axis_base().my_edges[slots[1]].clone();
        return Ok(store_edge(prim, slots[0], other));
    }
    let builder = prim.one_axis_base().my_builder;
    let mut edge = Edge::default();
    if !prim.meridian_on_axis(v)? {
        let mp = prim.meridian_value(v)?;
        let axes = prim.axes();
        let frame = Axis2::with_x_direction(
            axis_point(axes, mp.y()),
            axes.direction(),
            axes.x_direction(),
        )?;
        builder.make_edge_circle(&mut edge, Circle3::new(frame, mp.x())?)?;
    } else {
        builder.make_degenerated_edge(&mut edge)?;
    }
    let angle = prim.angle();
    if !prim.has_sides() {
        let vertex = makers[0](prim)?;
        builder.add_edge_vertex_closed(&mut edge, &vertex, 0.0, angle)?;
    } else {
        let vertex = makers[0](prim)?;
        builder.add_edge_vertex(&mut edge, &vertex, angle, false)?;
        let vertex = makers[1](prim)?;
        builder.add_edge_vertex(&mut edge, &vertex, 0.0, true)?;
    }
    Ok(store_edge(prim, slots[0], edge))
}

/// `TopEdge()`: the circle at VMax.
pub(crate) fn top_edge<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Edge, Failure> {
    let vmax = prim.vmax();
    circle_edge(
        prim,
        [ETOP, EBOTTOM],
        vmax,
        [T::top_end_vertex, T::top_start_vertex],
    )
}

/// `BottomEdge()`: the circle at VMin.
pub(crate) fn bottom_edge<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Edge, Failure> {
    let vmin = prim.vmin();
    circle_edge(
        prim,
        [EBOTTOM, ETOP],
        vmin,
        [T::bottom_end_vertex, T::bottom_start_vertex],
    )
}
