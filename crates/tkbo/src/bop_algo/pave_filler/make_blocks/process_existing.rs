//! `BOPAlgo_PaveFiller::ProcessExistingPaveBlocks` (both overloads): existing On/In blocks
//! joining the section post treatment.
use super::super::{BOPAlgoPaveFiller, missing_structure};
use super::existing_blocks::add_block_face;
use super::{BlockFaces, BlockTree, CurveRef, SectionBlocks, SectionShapes};
use crate::bop_ds::{PaveBlockHandle, PaveBlockIndexMap};
use crate::bop_tools::BoxSelector;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::{BoundingBox3, box3_to_bvh};
use rustycad_tkmath::bvh::BvhSetTraverse;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    /// Adds the On/In blocks near a new section edge to the section post treatment: those on
    /// both faces always, the others when their recorded distance to the other face fits
    /// (`ProcessExistingPaveBlocks` for one edge).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn process_existing_blocks_near_edge(
        &mut self,
        curve: CurveRef,
        faces: (i32, i32),
        section_edge: &Edge,
        on_in: &PaveBlockIndexMap,
        tree: &mut BlockTree,
        sections: &mut SectionShapes,
        block_faces: &mut BlockFaces,
        added: &mut SectionBlocks,
    ) -> Result<(), Failure> {
        let mut edge_box = BoundingBox3::default();
        BRepBndLib::add(section_edge, &mut edge_box, false)?;
        let mut selector = BoxSelector::default();
        selector.set_box(box3_to_bvh(&edge_box)?);
        selector.set_bvh_set(Some(tree));
        if selector.select()? == 0 {
            return Ok(());
        }
        let selected: Vec<i32> = selector.indices().iter().copied().collect();
        let edge_tolerance = BRepTool::edge_tolerance(section_edge)?;
        for index in selected {
            let candidate = Arc::clone(on_in.find_key(index as usize)?);
            if added.contains(&candidate) {
                continue;
            }
            let (in_first, in_second) = self.block_in_faces(&candidate, faces)?;
            if in_first && in_second {
                added.add(Arc::clone(&candidate));
                self.prepare_post_treat_ff(curve, &candidate, sections)?;
                continue;
            }
            let face = if in_first { faces.1 } else { faces.0 };
            if self.block_close_to_face(&candidate, face, edge_tolerance)? {
                added.add(Arc::clone(&candidate));
                self.prepare_post_treat_ff(curve, &candidate, sections)?;
                add_block_face(block_faces, &candidate, face);
            }
        }
        Ok(())
    }
    /// Whether the EF distance recorded for the block's range and face is within the sum of
    /// the edge tolerances.
    pub(super) fn block_close_to_face(
        &self,
        block: &PaveBlockHandle,
        face: i32,
        edge_tolerance: f64,
    ) -> Result<bool, Failure> {
        let (original, range, edge) = {
            let locked = block.lock().expect("pave-block lock");
            (locked.original_edge(), locked.range(), locked.edge())
        };
        let Some(list) = self
            .distances
            .seek(&crate::bop_ds::Pair::new(original, face))
        else {
            return Ok(false);
        };
        let (t1, t2) = range;
        let distance = list
            .iter()
            .find(|item| {
                (t1 <= item.first && item.first <= t2)
                    || (t1 <= item.last && item.last <= t2)
                    || (item.first <= t1 && t1 <= item.last)
                    || (item.first <= t2 && t2 <= item.last)
            })
            .map_or(f64::MAX, |item| item.distance);
        if distance >= f64::MAX {
            return Ok(false);
        }
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let shape = Edge::unchecked(structure.shape(edge)?.clone());
        Ok(distance <= edge_tolerance + BRepTool::edge_tolerance(&shape)?)
    }
    /// Adds the On/In blocks passing through the new bound vertices of the curves to the
    /// section post treatment (`ProcessExistingPaveBlocks` for bound vertices).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn process_existing_blocks_at_bounds(
        &mut self,
        ff: usize,
        faces: (i32, i32),
        on_in: &PaveBlockIndexMap,
        tree: &mut BlockTree,
        bound_vertices: &rustycad_tkernel::ncollection::DataMap<i32, Vec<i32>>,
        sections: &mut SectionShapes,
        block_faces: &mut BlockFaces,
        added: &mut SectionBlocks,
    ) -> Result<(), Failure> {
        if bound_vertices.is_empty() {
            return Ok(());
        }
        let items: Vec<(i32, Vec<i32>)> = bound_vertices
            .items()
            .map(|(key, list)| (*key, list.clone()))
            .collect();
        for (curve_index, vertices) in items {
            let curve = (ff, curve_index as usize);
            for vertex in vertices {
                self.process_block_at_vertex(
                    curve,
                    faces,
                    vertex,
                    on_in,
                    tree,
                    sections,
                    block_faces,
                    added,
                )?;
            }
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn process_block_at_vertex(
        &mut self,
        curve: CurveRef,
        faces: (i32, i32),
        vertex: i32,
        on_in: &PaveBlockIndexMap,
        tree: &mut BlockTree,
        sections: &mut SectionShapes,
        block_faces: &mut BlockFaces,
        added: &mut SectionBlocks,
    ) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let info = structure.shape_info(vertex)?;
        let shape = Vertex::unchecked(info.shape().clone());
        if !sections
            .vertex_indices
            .is_bound(&shape.clone().into_shape())
        {
            return Ok(());
        }
        let mut selector = BoxSelector::default();
        selector.set_box(box3_to_bvh(info.box_bounds())?);
        selector.set_bvh_set(Some(tree));
        if selector.select()? == 0 {
            return Ok(());
        }
        let selected: Vec<i32> = selector.indices().iter().copied().collect();
        for index in selected {
            let candidate = Arc::clone(on_in.find_key(index as usize)?);
            let (bounds, edge) = {
                let locked = candidate.lock().expect("pave-block lock");
                (locked.indices(), locked.edge())
            };
            if bounds.0 == vertex || bounds.1 == vertex || added.contains(&candidate) {
                continue;
            }
            let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
            let edge_shape = Edge::unchecked(structure.shape(edge)?.clone());
            let (mut parameter, mut dummy) = (0., 0.);
            let fuzzy = self.options.fuzzy_value();
            if self.context_ref()?.compute_ve(
                &shape,
                &edge_shape,
                &mut parameter,
                &mut dummy,
                fuzzy,
            )? != 0
            {
                continue;
            }
            added.add(Arc::clone(&candidate));
            self.prepare_post_treat_ff(curve, &candidate, sections)?;
            let (in_first, in_second) = self.block_in_faces(&candidate, faces)?;
            if !in_first || !in_second {
                let face = if in_first { faces.1 } else { faces.0 };
                add_block_face(block_faces, &candidate, face);
            }
        }
        Ok(())
    }
}
