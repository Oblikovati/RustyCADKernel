// Rust translation of BRepPrim_OneAxis.cxx (LateralWire, LateralStartWire,
// LateralEndWire, TopWire, BottomWire, StartWire, AxisStartWire, EndWire, AxisEndWire)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The wires of `BRepPrim_OneAxis`, each built once.
use super::BRepPrimOneAxis;
use super::one_axis::{
    WAXISEND, WAXISSTART, WBOTTOM, WEND, WLATERAL, WLATERALEND, WLATERALSTART, WSTART, WTOP,
    are_heights_equal, is_height_inverted, one_axis_domain_error,
};
use rustycad_tkbrep::topods::{Edge, Wire};
use rustycad_tkernel::standard::Failure;

/// One edge of a wire: the member that builds it and whether it is added forward.
type WireEdge<T> = (fn(&mut T) -> Result<Edge, Failure>, bool);

/// The built wire in slot `slot`, or `None` when it is not built yet.
fn built_wire<T: BRepPrimOneAxis + ?Sized>(prim: &T, slot: usize) -> Option<Wire> {
    let base = prim.one_axis_base();
    base.wires_built[slot].then(|| base.my_wires[slot].clone())
}

/// Makes the wire of `edges` in order, completes it and stores it in slot `slot`.
fn make_wire<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
    slot: usize,
    edges: &[WireEdge<T>],
) -> Result<Wire, Failure> {
    let builder = prim.one_axis_base().my_builder;
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for (edge, direct) in edges {
        let edge = edge(prim)?;
        builder.add_wire_edge(&mut wire, &edge, *direct)?;
    }
    builder.complete_wire(&wire)?;
    let base = prim.one_axis_base_mut();
    base.my_wires[slot] = wire.clone();
    base.wires_built[slot] = true;
    Ok(wire)
}

/// `LateralWire()`: the top edge (reversed, when VMax is finite), the end edge, the
/// bottom edge (when VMin is finite) and the start edge (reversed).
pub(crate) fn lateral_wire<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Wire, Failure> {
    if let Some(wire) = built_wire(prim, WLATERAL) {
        return Ok(wire);
    }
    let mut edges: Vec<WireEdge<T>> = Vec::new();
    if !prim.vmax_infinite() {
        edges.push((T::top_edge, false));
    }
    edges.push((T::end_edge, true));
    if !prim.vmin_infinite() {
        edges.push((T::bottom_edge, true));
    }
    edges.push((T::start_edge, false));
    make_wire(prim, WLATERAL, &edges)
}

/// `LateralStartWire()`: the start edge alone, reversed.
pub(crate) fn lateral_start_wire<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
) -> Result<Wire, Failure> {
    if let Some(wire) = built_wire(prim, WLATERALSTART) {
        return Ok(wire);
    }
    make_wire(prim, WLATERALSTART, &[(T::start_edge, false)])
}

/// `LateralEndWire()`: the end edge alone.
pub(crate) fn lateral_end_wire<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Wire, Failure> {
    if let Some(wire) = built_wire(prim, WLATERALEND) {
        return Ok(wire);
    }
    make_wire(prim, WLATERALEND, &[(T::end_edge, true)])
}

/// `TopWire()`: the top edge and, with sides, the start top edge and the end top edge
/// (reversed); Standard_DomainError without a top.
pub(crate) fn top_wire<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Wire, Failure> {
    if let Some(wire) = built_wire(prim, WTOP) {
        return Ok(wire);
    }
    if !prim.has_top()? {
        return Err(one_axis_domain_error("TopWire", "no top"));
    }
    let mut edges: Vec<WireEdge<T>> = vec![(T::top_edge, true)];
    if prim.has_sides() {
        edges.push((T::start_top_edge, true));
        edges.push((T::end_top_edge, false));
    }
    make_wire(prim, WTOP, &edges)
}

/// `BottomWire()`: the bottom edge (reversed) and, with sides, the end bottom edge and
/// the start bottom edge (reversed); Standard_DomainError without a bottom.
pub(crate) fn bottom_wire<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Wire, Failure> {
    if let Some(wire) = built_wire(prim, WBOTTOM) {
        return Ok(wire);
    }
    if !prim.has_bottom()? {
        return Err(one_axis_domain_error("BottomWire", "no bottom"));
    }
    let mut edges: Vec<WireEdge<T>> = vec![(T::bottom_edge, false)];
    if prim.has_sides() {
        edges.push((T::end_bottom_edge, true));
        edges.push((T::start_bottom_edge, false));
    }
    make_wire(prim, WBOTTOM, &edges)
}

/// Whether a side wire holds the axis edge: an open meridian with distinct heights and
/// a finite end.
fn side_has_axis<T: BRepPrimOneAxis + ?Sized>(prim: &T) -> Result<bool, Failure> {
    let heights_equal = are_heights_equal(prim)?;
    Ok(!prim.meridian_closed()?
        && !heights_equal
        && (!prim.vmax_infinite() || !prim.vmin_infinite()))
}

/// `StartWire()`: the start bottom edge, the axis edge (reversed unless the heights
/// are inverted), the start top edge (reversed) and the start edge, each when it
/// exists; Standard_DomainError without sides.
pub(crate) fn start_wire<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Wire, Failure> {
    if let Some(wire) = built_wire(prim, WSTART) {
        return Ok(wire);
    }
    if !prim.has_sides() {
        return Err(one_axis_domain_error("StartWire", "no sides"));
    }
    let is_inverted = is_height_inverted(prim)?;
    let mut edges: Vec<WireEdge<T>> = Vec::new();
    if prim.has_bottom()? {
        edges.push((T::start_bottom_edge, true));
    }
    if side_has_axis(prim)? {
        edges.push((T::axis_edge, is_inverted));
    }
    if prim.has_top()? {
        edges.push((T::start_top_edge, false));
    }
    edges.push((T::start_edge, true));
    make_wire(prim, WSTART, &edges)
}

/// `EndWire()`: the end top edge, the axis edge (forward unless the heights are
/// inverted), the end bottom edge (reversed) and the end edge (reversed), each when it
/// exists; Standard_DomainError without sides.
pub(crate) fn end_wire<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Wire, Failure> {
    if let Some(wire) = built_wire(prim, WEND) {
        return Ok(wire);
    }
    if !prim.has_sides() {
        return Err(one_axis_domain_error("EndWire", "no sides"));
    }
    let is_inverted = is_height_inverted(prim)?;
    let mut edges: Vec<WireEdge<T>> = Vec::new();
    if prim.has_top()? {
        edges.push((T::end_top_edge, true));
    }
    if side_has_axis(prim)? {
        edges.push((T::axis_edge, !is_inverted));
    }
    if prim.has_bottom()? {
        edges.push((T::end_bottom_edge, false));
    }
    edges.push((T::end_edge, false));
    make_wire(prim, WEND, &edges)
}

/// The checks of the axis wires: sides, an infinite range at both ends and an open
/// meridian, else Standard_DomainError.
fn check_axis_wire<T: BRepPrimOneAxis + ?Sized>(prim: &T, member: &str) -> Result<(), Failure> {
    if !prim.has_sides() {
        return Err(one_axis_domain_error(member, "no sides"));
    }
    if !prim.vmax_infinite() || !prim.vmin_infinite() {
        return Err(one_axis_domain_error(member, "not infinite"));
    }
    if prim.meridian_closed()? {
        return Err(one_axis_domain_error(member, "meridian closed"));
    }
    Ok(())
}

/// `AxisStartWire()`: the axis edge alone, reversed; Standard_DomainError without
/// sides, with a finite end or a closed meridian.
pub(crate) fn axis_start_wire<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Wire, Failure> {
    if let Some(wire) = built_wire(prim, WAXISSTART) {
        return Ok(wire);
    }
    check_axis_wire(prim, "AxisStartWire")?;
    make_wire(prim, WAXISSTART, &[(T::axis_edge, false)])
}

/// `AxisEndWire()`: the axis edge alone; Standard_DomainError without sides, with a
/// finite end or a closed meridian.
pub(crate) fn axis_end_wire<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Wire, Failure> {
    if let Some(wire) = built_wire(prim, WAXISEND) {
        return Ok(wire);
    }
    check_axis_wire(prim, "AxisEndWire")?;
    make_wire(prim, WAXISEND, &[(T::axis_edge, true)])
}
