// Rust translation of the static helpers Substitute, KeepEdge, BuildVertex, NullEdge,
// ReverseEdgeInFirstOrLastWire and ReverseModifiedEdges
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Topological helpers of BRepFill_Sweep: substitutions, edges sharing a curve,
//! vertices and degenerated edges, and edge reversals in wires.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::Substitution;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `Substitute(aSubstitute, Old, New)`: the vertices and the edge of `old` replaced
/// (FORWARD) by those of `new`, unless already substituted.
pub(super) fn substitute(
    a_substitute: &mut Substitution,
    old: &Edge,
    new: &Edge,
) -> Result<(), Failure> {
    let (old_v1, old_v2) = topexp::vertices(old, false);
    let (new_v1, new_v2) = topexp::vertices(new, false);
    for (old_shape, new_shape) in [
        (old_v1.into_shape(), new_v1.into_shape()),
        (old_v2.into_shape(), new_v2.into_shape()),
        (old.as_ref().clone(), new.as_ref().clone()),
    ] {
        if !a_substitute.is_copied(&old_shape) {
            a_substitute.substitute(&old_shape, &[new_shape.oriented(Orientation::Forward)])?;
        }
    }
    Ok(())
}

/// `KeepEdge(Face, Edge, List)`: the edges of the face on the same 3d curve (the same
/// handle and location) as `edge`.
pub(super) fn keep_edge(face: &Shape, edge: &Shape) -> Result<Vec<Shape>, Failure> {
    let reference = BRepTool::curve_with_location(&topods::edge(edge)?)?;
    let mut list: Vec<Shape> = Vec::new();
    for e in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let found = BRepTool::curve_with_location(&topods::edge(&e)?)?;
        if found.curve == reference.curve && found.location == reference.location {
            list.push(e);
        }
    }
    Ok(list)
}

/// `BuildVertex(Iso, isfirst, First, Last, Vertex)`: a new vertex at the first or last
/// parameter of the iso curve.
pub(super) fn build_vertex(
    iso: &Handle<dyn GeomCurve>,
    isfirst: bool,
    first: f64,
    last: f64,
) -> Result<Vertex, Failure> {
    let curve = iso.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepFill_Sweep BuildVertex: null iso curve; expected a curve",
            "BRepFill_Sweep",
        )
    })?;
    let val: f64 = if isfirst { first } else { last };
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(
        &mut vertex,
        curve.value(val)?,
        precision::confusion(),
    )?;
    Ok(vertex)
}

/// `NullEdge(Vertex)`: a degenerated edge without curve on the vertex (made FORWARD).
pub(super) fn null_edge(vertex: &mut Shape) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut e = Edge::default();
    builder.make_edge(&mut e)?;
    vertex.set_orientation(Orientation::Forward);
    builder.add(&mut e, vertex)?;
    builder.add(&mut e, &vertex.reversed())?;
    builder.degenerated(&e, true)?;
    Ok(e)
}

/// `ReverseEdgeInFirstOrLastWire(theWire, theEdge)`: the edge of the wire that is the
/// same as `the_edge` replaced by its reverse.
pub(super) fn reverse_edge_in_first_or_last_wire(
    the_wire: &mut Shape,
    the_edge: &Shape,
) -> Result<(), Failure> {
    let mut edge_to_reverse = Shape::default();
    let mut itw = ShapeIterator::new(the_wire, true, true);
    while itw.more() {
        if itw.value()?.is_same(the_edge) {
            edge_to_reverse = itw.value()?.clone();
            break;
        }
        itw.next();
    }
    if edge_to_reverse.is_null() {
        return Ok(());
    }
    let builder = BRepBuilder::default();
    the_wire.set_free(true)?;
    builder.remove(the_wire, &edge_to_reverse)?;
    edge_to_reverse.reverse();
    builder.add(the_wire, &edge_to_reverse)
}

/// `ReverseModifiedEdges(theWire, theEmap)`: the wire rebuilt with the edges of the
/// map reversed.
pub(super) fn reverse_modified_edges(
    the_wire: &mut Wire,
    the_emap: &MapOfShape,
) -> Result<(), Failure> {
    if the_emap.is_empty() {
        return Ok(());
    }
    let mut ledges: Vec<Shape> = Vec::new();
    let mut itw = ShapeIterator::new(the_wire, true, true);
    while itw.more() {
        ledges.push(itw.value()?.clone());
        itw.next();
    }
    let builder = BRepBuilder::default();
    the_wire.set_free(true)?;
    for e in &ledges {
        builder.remove(the_wire, e)?;
    }
    for e in ledges {
        let mut an_edge = e;
        if the_emap.contains(&an_edge) {
            an_edge.reverse();
        }
        builder.add(the_wire, &an_edge)?;
    }
    Ok(())
}
