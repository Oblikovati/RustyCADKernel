// Rust translation of BRepFill_Pipe::BuildHistory and of the static UpdateMap
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Pipe.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The history of a pipe: the tapes of the profile edges, the rails of their vertices,
//! the faces of the spine edges and the sections at the spine vertices.
use super::BRepFillPipe;
use crate::brep_fill::shape_table::{ShapeTable, cell, copy_table};
use crate::brep_fill::sweep::BRepFillSweep;
use crate::brep_fill::trim_shell_corner::HistoryMap;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `UpdateMap(theKey, theValue, theMap)`: the value appended to the list of the key
/// unless an equal (`IsSame`) one is there; whether it was appended.
pub(super) fn update_map(
    the_key: &Shape,
    the_value: &Shape,
    the_map: &mut HistoryMap,
) -> Result<bool, Failure> {
    if !the_map.is_bound(the_key) {
        the_map.bind(the_key.clone(), List::default());
    }
    let a_list = the_map.change_find(the_key)?;
    if a_list.iter().any(|s| the_value.is_same(s)) {
        return Ok(false);
    }
    a_list.append(the_value.clone());
    Ok(true)
}

impl BRepFillPipe {
    /// `BuildHistory(theSweep, theSection)` (private): for a wire section, the tape of
    /// each new edge and the rails of its vertices; then the faces of each spine edge
    /// and the sections at each spine vertex.
    pub(super) fn build_history(
        &mut self,
        the_sweep: &BRepFillSweep,
        the_section: &Shape,
    ) -> Result<(), Failure> {
        let an_u_edges: ShapeTable = copy_table(&the_sweep.inter_faces(), "BRepFill_Pipe")?;
        if the_section.shape_type()? == ShapeEnum::Wire {
            let mut wexp_sec = WireExplorer::with_wire(&topods::wire(the_section)?)?;
            let mut inde: i32 = 0;
            while wexp_sec.more() {
                inde += 1;
                let an_edge: Edge = wexp_sec.current().clone();
                if !BRepTool::degenerated(&an_edge)? && !self.my_gen_map.is_bound(&an_edge) {
                    self.edge_history(the_sweep, &an_edge, inde, &an_u_edges)?;
                }
                wexp_sec.next()?;
            }
        }
        self.spine_history(the_sweep)
    }

    /// The tape of a section edge and the rails of its vertices.
    fn edge_history(
        &mut self,
        the_sweep: &BRepFillSweep,
        an_edge: &Edge,
        inde: i32,
        an_u_edges: &ShapeTable,
    ) -> Result<(), Failure> {
        let a_vertex: [Vertex; 2] = topexp::vertices(an_edge, false).into();
        let a_tape: Shape = topods::shell(&the_sweep.tape(inde)?)?.into_shape();
        let mut u_index: [i32; 2] = [inde, inde + 1];
        if an_edge.orientation() == Orientation::Reversed {
            u_index.swap(0, 1);
        }
        for kk in 0..2 {
            if self.my_gen_map.is_bound(&a_vertex[kk]) {
                continue;
            }
            let elist = self
                .my_gen_map
                .bound(a_vertex[kk].as_ref().clone(), List::default());
            for jj in 1..=an_u_edges.upper_col() {
                let an_uedge: &Shape = cell(an_u_edges, u_index[kk], jj)?;
                if !an_uedge.is_null() {
                    elist.append(an_uedge.clone());
                }
            }
        }
        let flist = self
            .my_gen_map
            .bound(an_edge.as_ref().clone(), List::default());
        let mut itsh = ShapeIterator::new(&a_tape, true, true);
        while itsh.more() {
            flist.append(itsh.value()?.clone());
            itsh.next();
        }
        Ok(())
    }

    /// The faces of each spine edge and the v-shapes at each spine vertex.
    fn spine_history(&mut self, the_sweep: &BRepFillSweep) -> Result<(), Failure> {
        let a_faces: ShapeTable = copy_table(&the_sweep.sub_shape(), "BRepFill_Pipe")?;
        let a_v_edges: ShapeTable = copy_table(&the_sweep.sections(), "BRepFill_Pipe")?;
        let mut wexp = WireExplorer::with_wire(&self.my_spine)?;
        let mut inde: i32 = 0;
        loop {
            let to_exit: bool = !wexp.more();
            inde += 1;
            if !to_exit {
                let an_edge_of_spine: Shape = wexp.current().as_ref().clone();
                for i in 1..=a_faces.upper_row() {
                    update_map(
                        &an_edge_of_spine,
                        cell(&a_faces, i, inde)?,
                        &mut self.my_gen_map,
                    )?;
                }
            }
            let a_vertex_of_spine: Shape = wexp.current_vertex().as_ref().clone();
            for i in 1..=a_v_edges.upper_row() {
                update_map(
                    &a_vertex_of_spine,
                    cell(&a_v_edges, i, inde)?,
                    &mut self.my_gen_map,
                )?;
            }
            if to_exit {
                return Ok(());
            }
            wexp.next()?;
        }
    }
}
