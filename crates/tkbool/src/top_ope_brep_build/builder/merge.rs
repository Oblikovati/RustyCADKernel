// Rust translation of TopOpeBRepBuild_Builder::MakeFaces and the static
// CorrectEdgeOrientation and CorrectUnclosedWire of
// ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_Merge.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The faces of a face builder made with the build tool, their new wires corrected
//! when the wire check fails.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::TopOpeBRepBuildFaceBuilder;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_check::{BRepCheckAnalyzer, BRepCheckStatus};

/// The direct children of `s` (no cumulated orientation or location).
fn children(s: &Shape) -> Vec<Shape> {
    ShapeIterator::new(s, false, false).collect()
}

/// The first and last vertices of an edge with its orientation.
fn oriented_vertices(e: &Shape) -> Result<(Vertex, Vertex), Failure> {
    Ok(topexp::vertices(&rustycad_tkbrep::topods::edge(e)?, true))
}

/// One pass of CorrectEdgeOrientation over `edges`: each edge connected at an end of
/// the chain `(vf, vl)` appended (reversed when needed), the others returned; the
/// flag is the connection of the last edge of the pass, as natively.
fn chain_pass(
    edges: &[Shape],
    ends: &mut (Vertex, Vertex),
    chain: &mut Vec<Shape>,
) -> Result<(Vec<Shape>, bool), Failure> {
    let (mut aux, mut append) = (Vec::new(), true);
    for cur in edges {
        let (v1f, v1l) = oriented_vertices(cur)?;
        let (end, last, edge) = if v1f.as_ref().is_same(ends.1.as_ref()) {
            (&mut ends.1, v1l, cur.clone())
        } else if v1l.as_ref().is_same(ends.0.as_ref()) {
            (&mut ends.0, v1f, cur.clone())
        } else if v1l.as_ref().is_same(ends.1.as_ref()) {
            (&mut ends.1, v1f, cur.reversed())
        } else if v1f.as_ref().is_same(ends.0.as_ref()) {
            (&mut ends.0, v1l, cur.reversed())
        } else {
            append = false;
            aux.push(cur.clone());
            continue;
        };
        (*end, append) = (last, true);
        chain.push(edge);
    }
    Ok((aux, append))
}

/// `CorrectEdgeOrientation(aWire)` (static): the edges of the wire chained from the
/// first one, reversed where needed, the unconnected ones last, in a new wire.
fn correct_edge_orientation(a_wire: &mut Shape) -> Result<(), Failure> {
    let mut edges = children(a_wire);
    if edges.len() <= 1 {
        return Ok(());
    }
    let mut ends = oriented_vertices(&edges[0])?;
    let mut chain = vec![edges[0].clone()];
    edges.remove(0);
    let mut append = true;
    // The first pass starts after the first edge, the next ones on the rest.
    while !edges.is_empty() && append {
        (edges, append) = chain_pass(&edges, &mut ends, &mut chain)?;
    }
    chain.extend(edges);
    let bb = BRepBuilder::default();
    let mut w = Wire::default();
    bb.make_wire(&mut w);
    *a_wire = w.into_shape();
    for e in &chain {
        bb.add(a_wire, e)?;
    }
    Ok(())
}

/// `CorrectUnclosedWire(aWire)` (static): the edges with at most one vertex removed,
/// then the edges at a vertex of connexity 1. Natively the removal happens while the
/// children are iterated; the iteration then goes on with the next child, as here.
fn correct_unclosed_wire(a_wire: &mut Shape) -> Result<(), Failure> {
    let bb = BRepBuilder::default();
    for ed in children(a_wire) {
        if ed.nb_children() <= 1 {
            bb.remove(a_wire, &ed)?;
        }
    }
    let mut v_elists = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(a_wire, ShapeEnum::Vertex, ShapeEnum::Edge, &mut v_elists)?;
    for i in 1..=v_elists.len() {
        let elist: &List<Shape> = v_elists.find_from_index(i)?;
        if elist.len() == 1
            && let Some(an_edge) = elist.first()
        {
            bb.remove(a_wire, an_edge)?;
        }
    }
    Ok(())
}

/// Whether the check of the wire reports a bad orientation of a sub-shape.
fn has_bad_orientation(bca: &BRepCheckAnalyzer, w: &Shape) -> Result<bool, Failure> {
    let bcr = bca.result(w)?;
    Ok(bcr
        .status()?
        .iter()
        .any(|s| *s == BRepCheckStatus::BadOrientationOfSubshape))
}

impl TopOpeBRepBuildBuilder {
    /// The new wire of the current block loop of `fabu` (with the edges moved onto the
    /// new surface when `hns`), corrected when its check fails (IFV).
    fn make_new_wire(
        &self,
        a_face: &Shape,
        new_face: &Shape,
        fabu: &mut TopOpeBRepBuildFaceBuilder,
        hns: bool,
    ) -> Result<Shape, Failure> {
        let mut new_wire = Shape::default();
        self.my_build_tool.make_wire(&mut new_wire)?;
        fabu.init_edge()?;
        while fabu.more_edge() {
            let e = fabu.edge()?.clone();
            if hns {
                self.my_build_tool
                    .update_surface_of_edge(&e, a_face, new_face)?;
            }
            self.my_build_tool.add_wire_edge(&mut new_wire, &e)?;
            fabu.next_edge()?;
        }
        let bca = BRepCheckAnalyzer::new(&new_wire, false, false, false)?;
        if !bca.is_valid()? {
            new_wire.set_free(true)?;
            correct_unclosed_wire(&mut new_wire)?;
            if has_bad_orientation(&bca, &new_wire)? {
                correct_edge_orientation(&mut new_wire)?;
            }
        }
        Ok(new_wire)
    }

    /// The new surface of `a_face` in the data structure, set on `new_face`.
    fn update_new_surface(&self, a_face: &Shape, new_face: &Shape) -> Result<bool, Failure> {
        let Some(hds) = self.my_data_structure.get() else {
            return Ok(false);
        };
        let ds = hds.ds();
        if !ds.has_new_surface(a_face) {
            return Ok(false);
        }
        self.my_build_tool
            .update_surface(new_face, ds.new_surface(a_face))?;
        Ok(true)
    }

    /// `MakeFaces(aFace, FABU, L)`: per area of `fabu`, a copy of `a_face` (on the new
    /// surface of the data structure) with the old wires and the new ones, closed,
    /// appended to `l`; `b.make_faces(&face, &mut fb, &mut faces)?`.
    pub fn make_faces(
        &self,
        a_face: &Shape,
        fabu: &mut TopOpeBRepBuildFaceBuilder,
        l: &mut List<Shape>,
    ) -> Result<(), Failure> {
        fabu.init_face();
        while fabu.more_face() {
            let mut new_face = Shape::default();
            self.my_build_tool.copy_face(a_face, &mut new_face)?;
            let hns = self.update_new_surface(a_face, &new_face)?;
            fabu.init_wire();
            while fabu.more_wire() {
                let mut new_wire = if fabu.is_old_wire()? {
                    fabu.old_wire()?
                } else {
                    self.make_new_wire(a_face, &new_face, fabu, hns)?
                };
                self.my_build_tool.closed(&mut new_wire, true)?; // NYI : check exact du caractere closed du wire
                self.my_build_tool.add_face_wire(&mut new_face, &new_wire)?;
                fabu.next_wire();
            }
            l.append(new_face);
            fabu.next_face();
        }
        Ok(())
    }
}
