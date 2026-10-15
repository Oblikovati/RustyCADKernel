// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_BuildWires.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_BuildWires`: the wires chained from a list of edges, open ones running from
//! border vertex to border vertex (a vertex of one edge, or lying on the shape), closed
//! ones as loops.
use super::WiresOnShape;
use crate::errors::{construction_error, not_done};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Compound, Edge, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `LocOpe_BuildWires`: the wires built from edges.
///
/// ```
/// use rustycad_tkfeat::loc_ope::BuildWires;
/// let build = BuildWires::new();
/// assert!(!build.is_done());
/// ```
#[derive(Clone, Default)]
pub struct BuildWires {
    done: bool,
    res: List<Shape>,
}

/// `FindFirstEdge`: the first vertex with edges left that is a border, else the first
/// vertex with edges left; past the end when none has.
fn find_first_edge(
    map_ve: &IndexedDataMapOfShapeListOfShape,
    bord: &MapOfShape,
) -> Result<usize, Failure> {
    let n = map_ve.extent();
    let Some(first) = (1..=n).find(|&i| map_ve.find_from_index(i).is_ok_and(|l| !l.is_empty()))
    else {
        return Ok(n + 1);
    };
    for i in first..=n {
        if !map_ve.find_from_index(i)?.is_empty() && bord.contains(map_ve.find_key(i)?) {
            return Ok(i);
        }
    }
    Ok(first)
}

/// The edges of `l` once each, FORWARD in a compound (orientation importante pour
/// appel a TopExp::Vertices).
fn edge_compound(l: &List<Shape>) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut c = Compound::default();
    b.make_compound(&mut c);
    let mut c = c.into_shape();
    let mut the_map = MapOfShape::default();
    for edg in l.iter() {
        if the_map.add(edg.clone()) && edg.shape_type()? == ShapeEnum::Edge {
            b.add(&mut c, &edg.oriented(Orientation::Forward))?;
        }
    }
    Ok(c)
}

/// The border vertices: those of one edge, or bound to a vertex or an edge of the shape.
fn border_vertices(
    map_ve: &IndexedDataMapOfShapeListOfShape,
    pw: &WiresOnShape,
) -> Result<MapOfShape, Failure> {
    let mut bords = MapOfShape::default();
    for i in 1..=map_ve.extent() {
        //  Modified by skv - Mon May 31 13:07:50 2004 OCC5865 Begin
        let vtx = topods::vertex(map_ve.find_key(i)?)?;
        let (mut etmp, mut partmp) = (Edge::default(), 0.0);
        if map_ve.find_from_index(i)?.len() == 1
            || pw.on_vertex(&vtx)?.is_some()
            || pw.on_edge_of_vertex(&vtx, &mut etmp, &mut partmp)?
        {
            bords.add(vtx.into_shape());
        }
    }
    Ok(bords)
}

/// The chain of edges from the vertex `i`: oriented edges in order, the vertices met,
/// and the last vertex (on a border, or met before for a loop).
fn chain_from(
    i: usize,
    map_ve: &IndexedDataMapOfShapeListOfShape,
    bords: &MapOfShape,
) -> Result<(IndexedMapOfShape, MapOfShape, Vertex), Failure> {
    let mut map_e = IndexedMapOfShape::default();
    let mut map_v = MapOfShape::default();
    let edgf =
        topods::edge(map_ve.find_from_index(i)?.first().ok_or_else(|| {
            construction_error("LocOpe_BuildWires::Perform", "vertex without edge")
        })?)?;
    let (mut vf, mut vl) = topexp::vertices(&edgf, false);
    if bords.contains(&vl) && !bords.contains(&vf) {
        map_e.add(edgf.oriented(Orientation::Reversed));
        std::mem::swap(&mut vf, &mut vl);
    } else {
        map_e.add(edgf.oriented(Orientation::Forward));
    }
    map_v.add(vf.into_shape());
    while !(map_v.contains(&vl) || bords.contains(&vl)) {
        let ind = map_ve.find_index(&vl);
        let the_edge = map_ve
            .find_from_index(ind)?
            .iter()
            .find(|e| !map_e.contains(e))
            .cloned()
            .ok_or_else(|| {
                construction_error("LocOpe_BuildWires::Perform", "dead end in the edge chain")
            })?;
        let the_edge = topods::edge(&the_edge)?;
        let (vf2, vl2) = topexp::vertices(&the_edge, false);
        map_v.add(vl.as_ref().clone());
        if vf2.is_same(&vl) {
            map_e.add(the_edge.oriented(Orientation::Forward));
            vl = vl2;
        } else {
            // on doit avoir Vl == VL
            map_e.add(the_edge.oriented(Orientation::Reversed));
            vl = vf2;
        }
    }
    Ok((map_e, map_v, vl))
}

/// The wire of the chain: a loop from the edge starting at `vl` (the vertices before it
/// forgotten) when `vl` was met, else the whole open chain.
fn chain_wire(
    map_e: &IndexedMapOfShape,
    map_v: &mut MapOfShape,
    vl: &Vertex,
) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut new_wire = Wire::default();
    b.make_wire(&mut new_wire);
    let mut new_wire = new_wire.into_shape();
    let mut j = 1;
    let closed = map_v.contains(vl);
    if closed {
        // on sort avec une boucle a recreer
        while j <= map_e.extent() {
            let edg = topods::edge(map_e.find_key(j)?)?;
            let vf = if edg.orientation() == Orientation::Forward {
                topexp::first_vertex(&edg, false)
            } else {
                topexp::last_vertex(&edg, false)
            };
            if vf.is_same(vl) {
                break;
            }
            map_v.remove(&vf);
            j += 1;
        }
    } else {
        // on sort sur un bord : wire ouvert...
        map_v.add(vl.as_ref().clone());
    }
    for k in j..=map_e.extent() {
        b.add(&mut new_wire, map_e.find_key(k)?)?;
    }
    new_wire.set_closed(closed)?;
    Ok(new_wire)
}

impl BuildWires {
    /// `LocOpe_BuildWires()`: not done; `BuildWires::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_BuildWires(Ledges, PW)`: performed; `BuildWires::with_edges(&edges, &pw)?`.
    pub fn with_edges(ledges: &List<Shape>, pw: &WiresOnShape) -> Result<Self, Failure> {
        let mut build = Self::new();
        build.perform(ledges, pw)?;
        Ok(build)
    }

    /// `Perform(Ledges, PW)`: the wires chained from the edges of `ledges`, the borders
    /// being the vertices of one edge and those `pw` binds to the shape;
    /// `build.perform(&edges, &pw)?`.
    pub fn perform(&mut self, l: &List<Shape>, pw: &WiresOnShape) -> Result<(), Failure> {
        self.done = false;
        self.res.clear();
        let c = edge_compound(l)?;
        let mut map_ve = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(&c, ShapeEnum::Vertex, ShapeEnum::Edge, &mut map_ve)?;
        let mut bords = border_vertices(&map_ve, pw)?;
        loop {
            let i = find_first_edge(&map_ve, &bords)?;
            if i > map_ve.extent() {
                break;
            }
            let (map_e, mut map_v, vl) = chain_from(i, &map_ve, &bords)?;
            self.res.append(chain_wire(&map_e, &mut map_v, &vl)?);
            for vtx in map_v.iter() {
                bords.add(vtx.clone());
                let ind = map_ve.find_index(vtx);
                map_ve
                    .change_from_index(ind)?
                    .retain(|e| !map_e.contains(e));
            }
        }
        self.done = true;
        Ok(())
    }

    /// `IsDone()`; `build.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `Result()`: the wires; StdFail_NotDone before Perform; `build.result()?`.
    pub fn result(&self) -> Result<&List<Shape>, Failure> {
        if !self.done {
            return Err(not_done("LocOpe_BuildWires::Result"));
        }
        Ok(&self.res)
    }
}
