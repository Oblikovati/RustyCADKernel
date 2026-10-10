// Rust translation of BRepFill_OffsetWire::MakeWires and of the static IsSmallClosedEdge
// and CheckSmallParamOnEdge (ModelingAlgorithms/TKBool/BRepFill/BRepFill_OffsetWire.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The offset wires chained from the trimmed parallels through their shared vertices.
use super::BRepFillOffsetWire;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepGCurve, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Compound, Edge, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedDataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;

/// The edges at each vertex of the parallels (`MVE`).
type VertexEdges = IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The first and last parameters of the first curve representation of an edge.
fn first_rep_range(an_edge: &Edge) -> Result<Option<(f64, f64)>, Failure> {
    let curves = edge_tshape(an_edge, "BRepFill_OffsetWire")?.curves();
    let Some(first) = curves.first() else {
        return Ok(None);
    };
    let gcurve = first.cast::<dyn BRepGCurve>();
    let g = gcurve.get().ok_or_else(not_gcurve)?;
    Ok(Some((g.first(), g.last())))
}

/// `IsSmallClosedEdge(anEdge, aVertex)`: whether the ends and the middle of the first
/// p-curve of the edge are within 1.5 times the vertex tolerance of the vertex.
fn is_small_closed_edge(an_edge: &Edge, a_vertex: &Vertex) -> Result<bool, Failure> {
    let pv = BRepTool::pnt(a_vertex)?;
    let pv2d = Point2::new(pv.x(), pv.y());
    let curves = edge_tshape(an_edge, "BRepFill_OffsetWire::IsSmallClosedEdge")?.curves();
    let curve_rep = curves.first().ok_or_else(not_gcurve)?;
    let rep = curve_rep.get().ok_or_else(not_gcurve)?;
    let pcurve = rep.pcurve()?;
    let pc = pcurve.get().ok_or_else(not_gcurve)?;
    let (fpar, lpar) = first_rep_range(an_edge)?.ok_or_else(not_gcurve)?;
    let (pfirst, plast, pmid) = (
        pc.value(fpar)?,
        pc.value(lpar)?,
        pc.value((fpar + lpar) * 0.5)?,
    );
    let the_tol: f64 = BRepTool::vertex_tolerance(a_vertex)? * 1.5;
    Ok(pfirst.distance(pv2d) <= the_tol
        && plast.distance(pv2d) <= the_tol
        && pmid.distance(pv2d) <= the_tol)
}

/// `CheckSmallParamOnEdge(anEdge)`: false when the range of the first curve
/// representation is shorter than Precision::PConfusion().
fn check_small_param_on_edge(an_edge: &Edge) -> Result<bool, Failure> {
    Ok(match first_rep_range(an_edge)? {
        Some((f, l)) => (l - f).abs() >= precision::p_confusion(),
        None => true,
    })
}

/// The edge appended at the vertex.
fn add_at(mve: &mut VertexEdges, v: &Vertex, e: &Edge) {
    let index = mve.add(v.as_ref().clone(), List::default());
    if let Ok(list) = mve.change_from_index(index) {
        list.append(e.as_ref().clone());
    }
}

/// Removes the first occurrence of the edge from the list.
fn remove_edge(list: &mut List<Shape>, ce: &Edge) {
    let kept: Vec<Shape> = list.iter().cloned().collect();
    if let Some(at) = kept.iter().position(|s| s.is_same(ce)) {
        list.clear();
        for (i, s) in kept.into_iter().enumerate() {
            if i != at {
                list.append(s);
            }
        }
    }
}

impl BRepFillOffsetWire {
    /// The edges of the parallels at their vertices, without small closed edges and
    /// edges of a null range.
    fn vertex_edges(&self) -> Result<VertexEdges, Failure> {
        let mut mve = VertexEdges::default();
        for i in 1..=self.my_map.extent() {
            for s in self.my_map.find_from_index(i)?.iter() {
                let e: Edge = topods::edge(s)?;
                let (v1, v2) = topexp::vertices(&e, false);
                if v1.is_same(&v2) && is_small_closed_edge(&e, &v1)? {
                    continue; // Removes small closed edges.
                }
                if !check_small_param_on_edge(&e)? {
                    continue;
                }
                add_at(&mut mve, &v1, &e);
                add_at(&mut mve, &v2, &e);
            }
        }
        Ok(mve)
    }

    /// `MakeWires()` (private): the wires chained from a vertex with one edge (else the
    /// first vertex) through the vertices; the result is the wire, or a compound of the
    /// wires.
    pub(super) fn make_wires(&mut self) -> Result<(), Failure> {
        let mut mve: VertexEdges = self.vertex_edges()?;
        let mut the_wires: Vec<Wire> = Vec::new();
        while !mve.is_empty() {
            the_wires.push(self.chain_wire(&mut mve)?);
        }
        let b = BRepBuilder::default();
        if the_wires.len() == 1 {
            self.my_shape = the_wires.remove(0).into_shape();
            return Ok(());
        }
        let mut r = Compound::default();
        b.make_compound(&mut r);
        for w in &the_wires {
            b.add(&mut r, w)?;
        }
        self.my_shape = r.into_shape();
        Ok(())
    }

    /// One wire chained from the first vertex with one edge (else the first vertex).
    fn chain_wire(&self, mve: &mut VertexEdges) -> Result<Wire, Failure> {
        let b = BRepBuilder::default();
        let mut nw = Wire::default();
        b.make_wire(&mut nw);
        let mut i: usize = (1..=mve.extent())
            .find(|&i| mve.find_from_index(i).is_ok_and(|l| l.len() == 1))
            .unwrap_or(1);
        if i > mve.extent() {
            i = 1;
        }
        let vf: Vertex = topods::vertex(mve.find_key(i)?)?;
        let mut cv: Vertex = vf.clone();
        let mut ce: Edge = topods::edge(mve.find_from_index(i)?.first().ok_or_else(not_gcurve)?)?;
        mve.change_from_key(cv.as_ref())?.remove_first()?;
        if self.my_is_open_result && mve.find_from_key(cv.as_ref())?.is_empty() {
            mve.remove_key(cv.as_ref());
        }
        let mut is_closed = false;
        let mut end = false;
        while !end {
            let (v1, v2) = topexp::vertices(&ce, false);
            cv = if !cv.is_same(&v1) { v1 } else { v2 };
            b.add(&mut nw, &ce)?;
            if vf.is_same(&cv) || !mve.contains(cv.as_ref()) {
                is_closed = vf.is_same(&cv);
                end = true;
                mve.remove_key(vf.as_ref());
            }
            if !end {
                end = self.next_edge(mve, &cv, &mut ce)?;
            }
        }
        nw.set_closed(is_closed)?;
        Ok(nw)
    }

    /// The edge after `ce` at `cv` (the vertex removed once all its edges are used);
    /// true at an open end.
    fn next_edge(
        &self,
        mve: &mut VertexEdges,
        cv: &Vertex,
        ce: &mut Edge,
    ) -> Result<bool, Failure> {
        let mut end = false;
        let list = mve.change_from_key(cv.as_ref())?;
        remove_edge(list, ce);
        if let Some(first) = list.first().cloned() {
            *ce = topods::edge(&first)?;
            list.remove_first()?;
        } else if self.my_is_open_result {
            // CV was a vertex with one edge.
            end = true;
        }
        if mve.find_from_key(cv.as_ref())?.is_empty() {
            mve.remove_key(cv.as_ref());
        }
        Ok(end)
    }
}

/// Standard_NullObject of a curve representation that is not a GCurve.
fn not_gcurve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_OffsetWire: edge without geometric curve representation; expected a BRep_GCurve",
        "BRep_TEdge",
    )
}
