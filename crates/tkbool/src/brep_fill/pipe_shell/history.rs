// Rust translation of BRepFill_PipeShell::BuildHistory
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_PipeShell.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The history of a pipe shell: the shapes generated from the edges and vertices of the
//! profiles and of the spine.
use super::{BRepFillPipeShell, null};
use crate::brep_fill::shape_table::{ShapeTable, cell, copy_table};
use crate::brep_fill::sweep::BRepFillSweep;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Shell, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use std::collections::HashMap;

/// The children of a shape appended to a list.
fn append_children(shape: &Shape, list: &mut List<Shape>) -> Result<(), Failure> {
    let mut it = ShapeIterator::new(shape, true, true);
    while it.more() {
        list.append(it.value()?.clone());
        it.next();
    }
    Ok(())
}

/// +1 for a FORWARD shape, -1 otherwise.
fn sign_of(s: &Shape) -> i32 {
    if s.orientation() == Orientation::Forward {
        1
    } else {
        -1
    }
}

/// The edge of the vertex/edge map at `v` other than `skip` that touches `target`,
/// added to the wire; Standard_NoSuchObject when `v` is not in the map.
fn bridge(
    ve_map: &IndexedDataMapOfShapeListOfShape,
    v: &Vertex,
    skip: &Edge,
    target: &Vertex,
    a_wire: &mut Wire,
) -> Result<(), Failure> {
    for candidate in ve_map.find_from_key(v)?.iter() {
        if candidate.is_same(skip) {
            continue;
        }
        let (v1, v2) = topexp::vertices(&topods::edge(candidate)?, false);
        if v1.is_same(target) || v2.is_same(target) {
            return BRepBuilder::default().add(a_wire, candidate);
        }
    }
    Ok(())
}

/// The rail along the path through the u-edges of row `uindex`, its holes (round
/// corners) filled from the shell; `None` when its first u-edge is null.
fn rail_wire(
    u_edges: &ShapeTable,
    uindex: i32,
    ve_map: &IndexedDataMapOfShapeListOfShape,
    closed: bool,
) -> Result<Option<Wire>, Failure> {
    let builder = BRepBuilder::default();
    let mut a_wire = Wire::default();
    builder.make_wire(&mut a_wire);
    let first_edge: Edge = topods::edge(cell(u_edges, uindex, 1)?)?;
    if first_edge.is_null() {
        return Ok(None);
    }
    builder.add(&mut a_wire, &first_edge)?;
    let (first_vertex, mut cur_vertex) = topexp::vertices(&first_edge, false);
    let mut cur_edge = Edge::default();
    for jj in 2..=u_edges.upper_col() {
        cur_edge = topods::edge(cell(u_edges, uindex, jj)?)?;
        let (vfirst, vlast) = topexp::vertices(&cur_edge, false);
        if !cur_vertex.is_same(&vfirst) {
            bridge(ve_map, &vfirst, &cur_edge, &cur_vertex, &mut a_wire)?;
        }
        cur_vertex = vlast;
        builder.add(&mut a_wire, &cur_edge)?;
    }
    if closed && !cur_vertex.is_same(&first_vertex) {
        bridge(ve_map, &cur_vertex, &cur_edge, &first_vertex, &mut a_wire)?;
    }
    Ok(Some(a_wire))
}

/// The u-edge rows at the vertices of a profile edge: from the indices of the first
/// and last splits, shifted for a reversed edge, wrapped on a closed section and
/// exchanged when the edge or its split is reversed.
fn rail_indices(
    sec: &dyn crate::brep_fill::section_law::BRepFillSectionLaw,
    new_edges: &List<Shape>,
    (sign_of_an_edge, sign_of_a_new_edge, sign_of_index): (i32, i32, i32),
) -> Result<[i32; 2], Failure> {
    let missing = || {
        Failure::with_type(
            "Standard_NoSuchObject",
            "BRepFill_PipeShell::BuildHistory: an edge without split; expected at least one",
            "NCollection_List",
        )
    };
    let to_reverse: i32 = sign_of_an_edge * sign_of_a_new_edge * sign_of_index;
    let mut u_index: [i32; 2] = [
        sec.index_of_edge(new_edges.first().ok_or_else(missing)?)?
            .abs(),
        sec.index_of_edge(new_edges.last().ok_or_else(missing)?)?
            .abs()
            + to_reverse,
    ];
    if to_reverse == -1 {
        u_index[0] += 1;
        u_index[1] += 1;
    }
    if sec.is_u_closed() {
        let nb_law: i32 = sec.nb_law()?;
        for u in &mut u_index {
            if *u > nb_law {
                *u = 1;
            }
        }
    }
    if sign_of_an_edge == -1 || sign_of_a_new_edge == -1 {
        u_index.swap(0, 1);
    }
    Ok(u_index)
}

impl BRepFillPipeShell {
    /// `BuildHistory(theSweep)` (private): the tapes of the profile edges, the rails of
    /// their vertices, and the faces and section shapes of the spine edges and vertices.
    pub(super) fn build_history(&mut self, the_sweep: &BRepFillSweep) -> Result<(), Failure> {
        let an_u_edges: ShapeTable = copy_table(&the_sweep.inter_faces(), "BRepFill_PipeShell")?;
        let mut ind_wire_map: HashMap<i32, Wire> = HashMap::new();
        for indw in 0..self.my_seq.len() {
            let section: Shape = self.my_seq[indw].original_shape().clone();
            if self.my_seq[indw].is_punctual() {
                let elist = self.my_gen_map.bound(section, List::default());
                for i in 1..=an_u_edges.upper_row() {
                    for j in 1..=an_u_edges.upper_col() {
                        elist.append(cell(&an_u_edges, i, j)?.clone());
                    }
                }
                continue;
            }
            self.profile_history(the_sweep, indw, &an_u_edges, &mut ind_wire_map)?;
        }
        self.spine_history(the_sweep)
    }

    /// The history of the edges of one profile and of their vertices.
    fn profile_history(
        &mut self,
        the_sweep: &BRepFillSweep,
        indw: usize,
        an_u_edges: &ShapeTable,
        ind_wire_map: &mut HashMap<i32, Wire>,
    ) -> Result<(), Failure> {
        let a_section: Wire = topods::wire(self.my_seq[indw].original_shape())?;
        let ind_of_w: i32 = *self
            .my_ind_of_sec
            .get(indw)
            .ok_or_else(|| null("section index"))?;
        let the_wire: Wire = topods::wire(
            self.w_seq
                .get((ind_of_w - 1) as usize)
                .ok_or_else(|| null("placed section"))?,
        )?;
        let mut wexp_sec = WireExplorer::with_wire(&a_section)?;
        let mut inde: i32 = 1;
        while wexp_sec.more() {
            let an_original_edge: Edge = wexp_sec.current().clone();
            let an_edge: Edge =
                topods::edge(&self.my_seq[indw].modified_shape(&an_original_edge)?)?;
            if !BRepTool::degenerated(&an_edge)? {
                self.edge_history(
                    the_sweep,
                    &an_original_edge,
                    (&the_wire, inde),
                    an_u_edges,
                    ind_wire_map,
                )?;
                inde += 1;
            }
            wexp_sec.next()?;
        }
        Ok(())
    }

    /// The edge `inde` of the placed wire (the last one when there are fewer; the native
    /// degeneracy test reads the profile edge, which is not degenerated here).
    fn placed_edge(the_wire: &Wire, inde: i32) -> Result<Edge, Failure> {
        let mut wexp = WireExplorer::with_wire(the_wire)?;
        let mut the_edge = Edge::default();
        let mut i: i32 = 1;
        while wexp.more() {
            the_edge = wexp.current().clone();
            if i == inde {
                break;
            }
            i += 1;
            wexp.next()?;
        }
        Ok(the_edge)
    }

    /// The tape of a profile edge (the shell of the tapes of its splits) and the rails
    /// of its vertices.
    fn edge_history(
        &mut self,
        the_sweep: &BRepFillSweep,
        an_original_edge: &Edge,
        (the_wire, inde): (&Wire, i32),
        an_u_edges: &ShapeTable,
        ind_wire_map: &mut HashMap<i32, Wire>,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut a_shell = Shell::default();
        builder.make_shell(&mut a_shell);
        let a_vertex: [Vertex; 2] = topexp::vertices(an_original_edge, false).into();
        let the_edge: Edge = Self::placed_edge(the_wire, inde)?;
        let new_edges: List<Shape> = self.my_edge_new_edges.find(&the_edge)?.clone();
        let (mut sign_of_a_new_edge, mut sign_of_index) = (0, 0);
        for a_new_edge in new_edges.iter() {
            sign_of_a_new_edge = sign_of(a_new_edge);
            let an_ind_e: i32 = self.section()?.index_of_edge(a_new_edge)?;
            sign_of_index = if an_ind_e > 0 { 1 } else { -1 };
            let a_tape: Shape = the_sweep.tape(an_ind_e.abs())?;
            let mut itsh = ShapeIterator::new(&a_tape, true, true);
            while itsh.more() {
                builder.add(&mut a_shell, itsh.value()?)?;
                itsh.next();
            }
        }
        let u_index = rail_indices(
            self.section()?,
            &new_edges,
            (sign_of(an_original_edge), sign_of_a_new_edge, sign_of_index),
        )?;
        let mut ve_map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &a_shell,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut ve_map,
        )?;
        let closed: bool = self.location()?.is_closed()?;
        for kk in 0..2 {
            self.vertex_history(
                &a_vertex[kk],
                u_index[kk],
                (an_u_edges, &ve_map, closed),
                ind_wire_map,
            )?;
        }
        let flist = self
            .my_gen_map
            .bound(an_original_edge.as_ref().clone(), List::default());
        append_children(&a_shell, flist)
    }

    /// The rail of a profile vertex (reused for the same row), unless it has one.
    fn vertex_history(
        &mut self,
        v: &Vertex,
        u_index: i32,
        (an_u_edges, ve_map, closed): (&ShapeTable, &IndexedDataMapOfShapeListOfShape, bool),
        ind_wire_map: &mut HashMap<i32, Wire>,
    ) -> Result<(), Failure> {
        if self.my_gen_map.is_bound(v) {
            return Ok(());
        }
        if let Some(wire) = ind_wire_map.get(&u_index) {
            let elist = self.my_gen_map.bound(v.as_ref().clone(), List::default());
            return append_children(wire, elist);
        }
        let Some(a_wire) = rail_wire(an_u_edges, u_index, ve_map, closed)? else {
            return Ok(());
        };
        let elist = self.my_gen_map.bound(v.as_ref().clone(), List::default());
        append_children(&a_wire, elist)?;
        ind_wire_map.insert(u_index, a_wire);
        Ok(())
    }

    /// The faces of each spine edge and the section shapes (edges and faces, the
    /// children of compounds and wires) at each spine vertex.
    fn spine_history(&mut self, the_sweep: &BRepFillSweep) -> Result<(), Failure> {
        let a_faces: ShapeTable = copy_table(&the_sweep.sub_shape(), "BRepFill_PipeShell")?;
        let a_v_edges: ShapeTable = copy_table(&the_sweep.sections(), "BRepFill_PipeShell")?;
        let mut wexp = WireExplorer::with_wire(&self.my_spine)?;
        let mut inde: i32 = 0;
        loop {
            let to_exit: bool = !wexp.more();
            inde += 1;
            if !to_exit {
                let flist = self
                    .my_gen_map
                    .bound(wexp.current().as_ref().clone(), List::default());
                for i in 1..=a_faces.upper_row() {
                    let a_face: &Shape = cell(&a_faces, i, inde)?;
                    if a_face.shape_type()? == ShapeEnum::Face {
                        flist.append(a_face.clone());
                    }
                }
            }
            let list_vshapes = self
                .my_gen_map
                .bound(wexp.current_vertex().as_ref().clone(), List::default());
            for i in 1..=a_v_edges.upper_row() {
                append_section_shapes(cell(&a_v_edges, i, inde)?, list_vshapes)?;
            }
            if to_exit {
                return Ok(());
            }
            wexp.next()?;
        }
    }
}

/// The edges and faces of a v-shape: itself, or its children (those of a wire child).
fn append_section_shapes(a_vshape: &Shape, list: &mut List<Shape>) -> Result<(), Failure> {
    if a_vshape.is_null() {
        return Ok(());
    }
    if matches!(a_vshape.shape_type()?, ShapeEnum::Edge | ShapeEnum::Face) {
        list.append(a_vshape.clone());
        return Ok(());
    }
    let mut it = ShapeIterator::new(a_vshape, true, true);
    while it.more() {
        let a_subshape: &Shape = it.value()?;
        if matches!(a_subshape.shape_type()?, ShapeEnum::Edge | ShapeEnum::Face) {
            list.append(a_subshape.clone());
        } else {
            append_children(a_subshape, list)?;
        }
        it.next();
    }
    Ok(())
}
