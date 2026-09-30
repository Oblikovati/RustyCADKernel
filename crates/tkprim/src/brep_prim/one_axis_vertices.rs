// Rust translation of BRepPrim_OneAxis.cxx (AxisTopVertex, AxisBottomVertex,
// TopStartVertex, TopEndVertex, BottomStartVertex, BottomEndVertex) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertices of `BRepPrim_OneAxis`, each built once or shared with an equal vertex
//! built before it.
use super::BRepPrimOneAxis;
use super::one_axis::{
    VAXISBOT, VAXISTOP, VBOTEND, VBOTSTART, VTOPEND, VTOPSTART, are_heights_equal, axis_point,
    one_axis_domain_error,
};
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The built vertex in slot `slot`, or `None` when it is not built yet.
fn built_vertex<T: BRepPrimOneAxis + ?Sized>(prim: &T, slot: usize) -> Option<Vertex> {
    let base = prim.one_axis_base();
    base.vertices_built[slot].then(|| base.my_vertices[slot].clone())
}

/// Whether the vertex in slot `slot` is built.
fn is_built<T: BRepPrimOneAxis + ?Sized>(prim: &T, slot: usize) -> bool {
    prim.one_axis_base().vertices_built[slot]
}

/// Stores `vertex` in slot `slot` and marks it built.
fn store_vertex<T: BRepPrimOneAxis + ?Sized>(prim: &mut T, slot: usize, vertex: Vertex) -> Vertex {
    let base = prim.one_axis_base_mut();
    base.my_vertices[slot] = vertex.clone();
    base.vertices_built[slot] = true;
    vertex
}

/// Shares the vertex of slot `from` into slot `slot`.
fn share_vertex<T: BRepPrimOneAxis + ?Sized>(prim: &mut T, slot: usize, from: usize) -> Vertex {
    let vertex = prim.one_axis_base().my_vertices[from].clone();
    store_vertex(prim, slot, vertex)
}

/// Makes a new vertex at `point` and stores it in slot `slot`.
fn make_vertex<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
    slot: usize,
    point: Point3,
) -> Result<Vertex, Failure> {
    let mut vertex = Vertex::default();
    prim.one_axis_base()
        .my_builder
        .make_vertex(&mut vertex, point)?;
    Ok(store_vertex(prim, slot, vertex))
}

/// `AxisTopVertex()` and `AxisBottomVertex()`: the rim vertex of this cap when the
/// meridian end is on the axis and one is built, else the other axis vertex when the
/// heights are equal and it is built, else a new vertex on the axis at the meridian
/// height of `v` (Standard_DomainError for a closed meridian or an infinite end).
/// `slots` is `[this, start rim, end rim, other axis vertex]`.
fn axis_vertex<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
    slots: [usize; 4],
    v: f64,
    member: &str,
    infinite: bool,
) -> Result<Vertex, Failure> {
    if let Some(vertex) = built_vertex(prim, slots[0]) {
        return Ok(vertex);
    }
    if prim.meridian_on_axis(v)? && is_built(prim, slots[1]) {
        return Ok(share_vertex(prim, slots[0], slots[1]));
    }
    if prim.meridian_on_axis(v)? && is_built(prim, slots[2]) {
        return Ok(share_vertex(prim, slots[0], slots[2]));
    }
    if is_built(prim, slots[3]) && are_heights_equal(prim)? {
        return Ok(share_vertex(prim, slots[0], slots[3]));
    }
    if prim.meridian_closed()? || infinite {
        return Err(one_axis_domain_error(
            member,
            "closed meridian or infinite end; expected a finite open meridian",
        ));
    }
    let point = axis_point(prim.axes(), prim.meridian_value(v)?.y());
    make_vertex(prim, slots[0], point)
}

/// `AxisTopVertex()`: the vertex on the axis at the top height.
pub(crate) fn axis_top_vertex<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
) -> Result<Vertex, Failure> {
    let (vmax, infinite) = (prim.vmax(), prim.vmax_infinite());
    axis_vertex(
        prim,
        [VAXISTOP, VTOPSTART, VTOPEND, VAXISBOT],
        vmax,
        "AxisTopVertex",
        infinite,
    )
}

/// `AxisBottomVertex()`: the vertex on the axis at the bottom height.
pub(crate) fn axis_bottom_vertex<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
) -> Result<Vertex, Failure> {
    let (vmin, infinite) = (prim.vmin(), prim.vmin_infinite());
    axis_vertex(
        prim,
        [VAXISBOT, VBOTSTART, VBOTEND, VAXISTOP],
        vmin,
        "AxisBottomVertex",
        infinite,
    )
}

/// A rim vertex: the meridian point of `v` in the XZ plane of the axes, rotated by
/// Angle around the axis for an end vertex.
fn rim_point<T: BRepPrimOneAxis + ?Sized>(
    prim: &T,
    v: f64,
    rotated: bool,
) -> Result<Point3, Failure> {
    let mp = prim.meridian_value(v)?;
    let axes = prim.axes();
    let mut point = axis_point(axes, mp.y());
    let mut x = Vector3::from(axes.x_direction());
    x.scale(mp.x());
    point.translate(x);
    if rotated {
        point.rotate(axes.axis(), prim.angle())?;
    }
    Ok(point)
}

/// The sharing rules of a rim vertex: `slots` is `[this, axis vertex of its cap, the
/// other rim vertex of its cap, the rim vertex of the other cap on the same side, the
/// rim vertex of the other cap on the other side]`.
struct RimVertex {
    slots: [usize; 5],
    top: bool,
    rotated: bool,
}

/// `TopStartVertex()`, `TopEndVertex()`, `BottomStartVertex()`, `BottomEndVertex()`:
/// the axis vertex of the cap when the meridian end is on the axis, the other rim
/// vertex of the cap when the meridian end is on the axis or there are no sides, the
/// rim vertex of the other cap when the meridian is closed (on the other side too
/// without sides), each when built; else a new vertex at the meridian point.
fn rim_vertex<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
    rim: RimVertex,
) -> Result<Vertex, Failure> {
    let [slot, axis, other_rim, same_side, other_side] = rim.slots;
    if let Some(vertex) = built_vertex(prim, slot) {
        return Ok(vertex);
    }
    let v = if rim.top { prim.vmax() } else { prim.vmin() };
    if prim.meridian_on_axis(v)? && is_built(prim, axis) {
        return Ok(share_vertex(prim, slot, axis));
    }
    if (prim.meridian_on_axis(v)? || !prim.has_sides()) && is_built(prim, other_rim) {
        return Ok(share_vertex(prim, slot, other_rim));
    }
    if prim.meridian_closed()? && is_built(prim, same_side) {
        return Ok(share_vertex(prim, slot, same_side));
    }
    if (prim.meridian_closed()? && !prim.has_sides()) && is_built(prim, other_side) {
        return Ok(share_vertex(prim, slot, other_side));
    }
    let point = rim_point(prim, v, rim.rotated)?;
    make_vertex(prim, slot, point)
}

/// `TopStartVertex()`: the vertex (0, VMax).
pub(crate) fn top_start_vertex<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
) -> Result<Vertex, Failure> {
    rim_vertex(
        prim,
        RimVertex {
            slots: [VTOPSTART, VAXISTOP, VTOPEND, VBOTSTART, VBOTEND],
            top: true,
            rotated: false,
        },
    )
}

/// `TopEndVertex()`: the vertex (Angle, VMax).
pub(crate) fn top_end_vertex<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Vertex, Failure> {
    rim_vertex(
        prim,
        RimVertex {
            slots: [VTOPEND, VAXISTOP, VTOPSTART, VBOTEND, VBOTSTART],
            top: true,
            rotated: true,
        },
    )
}

/// `BottomStartVertex()`: the vertex (0, VMin).
pub(crate) fn bottom_start_vertex<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
) -> Result<Vertex, Failure> {
    rim_vertex(
        prim,
        RimVertex {
            slots: [VBOTSTART, VAXISBOT, VBOTEND, VTOPSTART, VTOPEND],
            top: false,
            rotated: false,
        },
    )
}

/// `BottomEndVertex()`: the vertex (Angle, VMin).
pub(crate) fn bottom_end_vertex<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
) -> Result<Vertex, Failure> {
    rim_vertex(
        prim,
        RimVertex {
            slots: [VBOTEND, VAXISBOT, VBOTSTART, VTOPEND, VTOPSTART],
            top: false,
            rotated: true,
        },
    )
}
