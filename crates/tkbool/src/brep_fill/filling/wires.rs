// Rust translation of BRepFill_Filling::BuildWires and FindExtremitiesOfHoles and of the
// static MakeFinVec and WireFromList (ModelingAlgorithms/TKBool/BRepFill/
// BRepFill_Filling.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The wires of the boundary edges of a filling, the vertices at the ends of its holes
//! and the final wire.
use super::BRepFillFilling;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Vector3;
use rustycad_tktopalgo::brep_lib::BRepLibMakeWire;
use std::f64::consts::PI;

/// `MakeFinVec(aWire, aVertex)`: the vector to the end vertex of the wire from the
/// other vertex of its edge there.
fn make_fin_vec(a_wire: &Wire, a_vertex: &Vertex) -> Result<Vector3, Failure> {
    let mut origin = Vertex::default();
    let mut explo = WireExplorer::with_wire(a_wire)?;
    while explo.more() {
        let (vfirst, vlast) = topexp::vertices(explo.current(), false);
        if vfirst.is_same(a_vertex) {
            origin = vlast;
            break;
        }
        if vlast.is_same(a_vertex) {
            origin = vfirst;
            break;
        }
        explo.next()?;
    }
    Ok(Vector3::between(
        BRepTool::pnt(&origin)?,
        BRepTool::pnt(a_vertex)?,
    ))
}

/// `WireFromList(Edges)`: the closed wire of the edges, each oriented to continue the
/// chain from either end (a warning and an incomplete wire when the chain breaks).
pub(super) fn wire_from_list(mut edges: Vec<Shape>) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut a_wire = Wire::default();
    builder.make_wire(&mut a_wire);
    if edges.is_empty() {
        return Err(empty_list("WireFromList: no edge"));
    }
    let first: Edge = topods::edge(&edges.remove(0))?;
    builder.add(&mut a_wire, &first)?;
    let (mut v1, mut v2) = topexp::vertices(&first, true);
    while !edges.is_empty() {
        let Some((index, an_edge)) = next_chained(&edges, &mut v1, &mut v2)? else {
            // The wire is not complete, some edges are lost (a native warning).
            break;
        };
        builder.add(&mut a_wire, &an_edge)?;
        edges.remove(index);
    }
    a_wire.set_closed(true)?;
    Ok(a_wire)
}

/// The first edge touching an end of the chain, oriented to continue it, with the end
/// moved to its other vertex.
fn next_chained(
    edges: &[Shape],
    v1: &mut Vertex,
    v2: &mut Vertex,
) -> Result<Option<(usize, Edge)>, Failure> {
    for (index, e) in edges.iter().enumerate() {
        let mut an_edge: Edge = topods::edge(e)?;
        let (v3, v4) = topexp::vertices(&an_edge, true);
        if !(v1.is_same(&v3) || v1.is_same(&v4) || v2.is_same(&v3) || v2.is_same(&v4)) {
            continue;
        }
        if v1.is_same(&v3) {
            an_edge.reverse();
            *v1 = v4;
        } else if v1.is_same(&v4) {
            *v1 = v3;
        } else if v2.is_same(&v3) {
            *v2 = v4;
        } else {
            an_edge.reverse();
            *v2 = v3;
        }
        return Ok(Some((index, an_edge)));
    }
    Ok(None)
}

/// Whether two vertices are each within the other's tolerance.
fn geometrically_close(v_wire: &Vertex, v_edge: &Vertex) -> Result<bool, Failure> {
    let a_dist: f64 = BRepTool::pnt(v_wire)?.distance(&BRepTool::pnt(v_edge)?);
    Ok(
        a_dist < BRepTool::vertex_tolerance(v_wire)?
            && a_dist < BRepTool::vertex_tolerance(v_edge)?,
    )
}

impl BRepFillFilling {
    /// `BuildWires(EdgeList, WireList)` (private): the edges chained into wires by
    /// shared vertices, else by vertices within their tolerances (the edge MakeWire
    /// rebuilds is recorded as the new edge).
    pub(super) fn build_wires(&mut self, mut edge_list: Vec<Shape>) -> Result<Vec<Shape>, Failure> {
        let mut wire_list: Vec<Shape> = Vec::new();
        while !edge_list.is_empty() {
            let mut mw = BRepLibMakeWire::new();
            mw.add_edge(&topods::edge(&edge_list.remove(0))?)?;
            loop {
                let (v0, v1) = topexp::wire_vertices(&mw.wire()?);
                let v_wire: [Vertex; 2] = [v0, v1];
                if let Some(index) = shared_vertex_edge(&edge_list, &v_wire)? {
                    mw.add_edge(&topods::edge(&edge_list.remove(index))?)?;
                    continue;
                }
                if let Some(index) = close_vertex_edge(&edge_list, &v_wire)? {
                    let cur_edge: Edge = topods::edge(&edge_list.remove(index))?;
                    mw.add_edge(&cur_edge)?;
                    let new_edge: Edge = mw.edge().clone();
                    self.my_old_new_map.bind(
                        cur_edge.oriented(Orientation::Forward),
                        new_edge.oriented(Orientation::Forward),
                    );
                    continue;
                }
                wire_list.push(mw.wire()?.into_shape());
                break;
            }
        }
        Ok(wire_list)
    }

    /// `FindExtremitiesOfHoles(WireList, VerSeq)` (private): for an open first wire, the
    /// pairs of vertices bounding the holes between the wires, chained from its last
    /// vertex by the smallest turn.
    pub(super) fn find_extremities_of_holes(
        &self,
        wire_list: &[Shape],
    ) -> Result<Vec<Vertex>, Failure> {
        let mut ver_seq: Vec<Vertex> = Vec::new();
        let mut wire_seq: Vec<Shape> = wire_list.to_vec();
        if wire_seq.is_empty() {
            return Err(empty_list("FindExtremitiesOfHoles: no wire"));
        }
        let the_wire: Wire = topods::wire(&wire_seq.remove(0))?;
        if BRepTool::is_closed(&the_wire)? {
            return Ok(ver_seq);
        }
        let (vfirst, vlast) = topexp::wire_vertices(&the_wire);
        if vfirst.is_same(&vlast) {
            // The wire is closed indeed despite its being not detected earlier.
            return Ok(ver_seq);
        }
        let mut fin_vec: Vector3 = make_fin_vec(&the_wire, &vlast)?;
        let mut the_vertex: Vertex = vlast.clone();
        ver_seq.push(vlast);
        while !wire_seq.is_empty() {
            let (min_vtx, min_ind, is_last) = nearest_turn(&wire_seq, &fin_vec, &the_vertex)?;
            ver_seq.push(min_vtx);
            let min_wire: Wire = topods::wire(
                wire_seq
                    .get(min_ind)
                    .ok_or_else(|| empty_list("FindExtremitiesOfHoles: wire index"))?,
            )?;
            let (vfirst, vlast) = topexp::wire_vertices(&min_wire);
            the_vertex = if is_last { vlast } else { vfirst };
            fin_vec = make_fin_vec(&min_wire, &the_vertex)?;
            ver_seq.push(the_vertex.clone());
            wire_seq.remove(min_ind);
        }
        ver_seq.push(topexp::wire_vertices(&the_wire).0);
        Ok(ver_seq)
    }
}

/// The index of the first edge sharing a vertex with an end of the wire.
fn shared_vertex_edge(edge_list: &[Shape], v_wire: &[Vertex; 2]) -> Result<Option<usize>, Failure> {
    for (index, e) in edge_list.iter().enumerate() {
        let (e0, e1) = topexp::vertices(&topods::edge(e)?, false);
        if v_wire.iter().any(|vw| vw.is_same(&e0) || vw.is_same(&e1)) {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

/// The index of the first edge with a vertex geometrically close to an end of the
/// wire.
fn close_vertex_edge(edge_list: &[Shape], v_wire: &[Vertex; 2]) -> Result<Option<usize>, Failure> {
    for (index, e) in edge_list.iter().enumerate() {
        let (e0, e1) = topexp::vertices(&topods::edge(e)?, false);
        for vw in v_wire {
            for ve in [&e0, &e1] {
                if geometrically_close(vw, ve)? {
                    return Ok(Some(index));
                }
            }
        }
    }
    Ok(None)
}

/// The wire end making the smallest angle with the end vector, from the current
/// vertex: its vertex, its wire index and whether it is the first vertex (the next
/// chain then starts from the last one).
fn nearest_turn(
    wire_seq: &[Shape],
    fin_vec: &Vector3,
    the_vertex: &Vertex,
) -> Result<(Vertex, usize, bool), Failure> {
    let (mut min_vtx, mut min_ind, mut is_last) = (Vertex::default(), 0usize, false);
    let mut min_angle: f64 = PI;
    let origin = BRepTool::pnt(the_vertex)?;
    for (i, w) in wire_seq.iter().enumerate() {
        let (vfirst, vlast) = topexp::wire_vertices(&topods::wire(w)?);
        let angle: f64 = fin_vec.angle(Vector3::between(origin, BRepTool::pnt(&vfirst)?))?;
        if angle < min_angle {
            (min_angle, min_vtx, min_ind, is_last) = (angle, vfirst.clone(), i, true);
        }
        let angle: f64 = fin_vec.angle(Vector3::between(origin, BRepTool::pnt(&vlast)?))?;
        if angle < min_angle {
            (min_angle, min_vtx, min_ind, is_last) = (angle, vlast, i, false);
        }
    }
    Ok((min_vtx, min_ind, is_last))
}

/// Standard_NoSuchObject / Standard_OutOfRange of an empty native sequence or list.
fn empty_list(what: &str) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("BRepFill_Filling {what}; expected a non-empty sequence"),
        "NCollection_Sequence",
    )
}
