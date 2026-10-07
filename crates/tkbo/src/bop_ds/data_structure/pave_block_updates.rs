//! Pave-block splitting, common-block regrouping and same-domain updates of `BOPDS_DS`.
use super::{DataStructure, Failure, checked_slot};
use crate::bop_ds::{
    CommonBlock, CommonBlockHandle, Pair, Pave, PaveBlock, PaveBlockHandle, common_block_handle,
    pave_block_handle,
};
use crate::bop_tools::point_on_edge;
use crate::int_tools::Tools;
use rustycad_tkbrep::{brep::BRepTool, topods::Edge};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::Point3;
use std::sync::Arc;

impl DataStructure {
    /// Splits every pave block that received extra paves, appending the new blocks at the end
    /// of the edge's list as native `UpdatePaveBlocks` does; `ds.update_pave_blocks();`.
    pub fn update_pave_blocks(&mut self) {
        for list in &mut self.pave_blocks_pool {
            let mut position = 0;
            while position < list.len() {
                if !lock_block(&list[position]).is_to_update() {
                    position += 1;
                    continue;
                }
                let splits = split_block(&list[position]);
                list.remove(position);
                list.extend(splits);
            }
        }
    }

    /// Splits one pave block in its edge list; appends then removes as native `UpdatePaveBlock`.
    /// `ds.update_pave_block(&block)?;`.
    pub fn update_pave_block(&mut self, block: &PaveBlockHandle) -> Result<(), Failure> {
        if !lock_block(block).is_to_update() {
            return Ok(());
        }
        let edge = lock_block(block).original_edge();
        let list = self.pave_block_list_mut(edge)?;
        let Some(position) = list.iter().position(|item| Arc::ptr_eq(item, block)) else {
            return Ok(());
        };
        let splits = split_block(&list[position]);
        list.extend(splits);
        list.remove(position);
        Ok(())
    }

    /// Splits the blocks of a common block and regroups the coincident parts into new common
    /// blocks with the same faces; `ds.update_common_block(&common, fuzzy)?;`.
    pub fn update_common_block(
        &mut self,
        common: &CommonBlockHandle,
        fuzzy_value: f64,
    ) -> Result<(), Failure> {
        let (blocks, faces) = common_block_parts(common)?;
        if !lock_block(&blocks[0]).is_to_update() {
            return Ok(());
        }
        let mut groups: DataMap<Pair, Vec<PaveBlockHandle>> = DataMap::default();
        for block in &blocks {
            self.split_common_part(block, &mut groups)?;
        }
        let mut pending: Vec<Vec<PaveBlockHandle>> = groups.values().cloned().collect();
        for group in &mut pending {
            self.regroup_coincident_blocks(group, &faces, fuzzy_value)?;
        }
        Ok(())
    }

    /// Returns true when an intermediate point of the first block projects inside the second
    /// block within both edges' vertex tolerances plus fuzz; `ds.check_coincidence(&a, &b, 0.)?`.
    pub(crate) fn check_coincidence(
        &self,
        first: &PaveBlockHandle,
        second: &PaveBlockHandle,
        fuzzy_value: f64,
    ) -> Result<bool, Failure> {
        let ((first_start, first_end), first_edge) = block_range_and_edge(first);
        let middle = Tools::intermediate_point(first_start, first_end);
        let edge1 = Edge::unchecked(self.shape(first_edge)?.clone());
        let mut point = Point3::default();
        point_on_edge(&edge1, middle, &mut point)?;
        let ((second_start, second_end), second_edge) = block_range_and_edge(second);
        let edge2 = Edge::unchecked(self.shape(second_edge)?.clone());
        let (curve, first_parameter, last_parameter) = BRepTool::curve(&edge2)?;
        let mut projector = ProjectPointOnCurve::new();
        projector.init_curve(&curve, first_parameter, last_parameter)?;
        projector.perform(point)?;
        if projector.nb_points() == 0 {
            return Ok(false);
        }
        let tolerance = BRepTool::max_tolerance(&edge1, ShapeEnum::Vertex)?
            + BRepTool::max_tolerance(&edge2, ShapeEnum::Vertex)?
            + fuzzy_value.max(rustycad_tkernel::precision::confusion());
        if projector.lower_distance()? >= tolerance {
            return Ok(false);
        }
        let parameter = projector.lower_distance_parameter()?;
        Ok(parameter > second_start && parameter < second_end)
    }

    /// Replaces both pave indices of every pool block by their same-domain representatives.
    /// `ds.update_pave_blocks_with_sd_vertices();`.
    pub fn update_pave_blocks_with_sd_vertices(&self) {
        for list in &self.pave_blocks_pool {
            for block in list {
                self.update_pave_block_with_sd_vertices(block);
            }
        }
    }

    /// Updates the paves of every block of a common block; `ds.update_common_block_with_sd_vertices(&cb)`.
    pub fn update_common_block_with_sd_vertices(&self, common: &CommonBlockHandle) {
        let blocks = common
            .lock()
            .expect("common-block lock must not be poisoned")
            .pave_blocks()
            .to_vec();
        for block in &blocks {
            self.update_pave_block_with_sd_vertices(block);
        }
    }

    /// Drops the pave-block reference of untouched source edges so no image is built for them.
    /// `ds.release_pave_blocks()?;`.
    pub fn release_pave_blocks(&mut self) -> Result<(), Failure> {
        for slot in 0..self.pave_blocks_pool.len() {
            let Some(edge) = self.untouched_single_block_edge(slot) else {
                continue;
            };
            if edge >= 0 {
                self.change_shape_info(edge)?.set_reference(-1);
            }
            self.pave_blocks_pool[slot].clear();
        }
        Ok(())
    }

    /// Returns the unique paves of an edge sorted by parameter with libstdc++ `std::sort`.
    /// `let paves = ds.paves(edge)?;`.
    pub fn paves(&self, edge: i32) -> Result<Vec<Pave>, Failure> {
        let blocks = self.pave_blocks(edge)?;
        if blocks.is_empty() {
            return Ok(Vec::new());
        }
        let mut paves: Vec<Pave> = Vec::with_capacity(blocks.len() + 1);
        for block in blocks {
            let block = lock_block(block);
            for pave in [block.pave1(), block.pave2()] {
                if !paves.iter().any(|known| known.is_equal(&pave)) {
                    paves.push(pave);
                }
            }
        }
        // The native array has PaveBlocks+1 slots; unfilled slots keep default paves.
        if paves.len() < blocks.len() + 1 {
            paves.resize(blocks.len() + 1, Pave::default());
        }
        rustycad_tkernel::stl::sort_by(&mut paves, |first, second| first.is_less(second));
        Ok(paves)
    }

    fn untouched_single_block_edge(&self, slot: usize) -> Option<i32> {
        let [block] = self.pave_blocks_pool[slot].as_slice() else {
            return None;
        };
        if self.is_common_block(block) {
            return None;
        }
        let block = lock_block(block);
        let (first, second) = block.indices();
        if self.is_new_shape(first) || self.is_new_shape(second) {
            return None;
        }
        Some(block.original_edge())
    }

    fn pave_block_list_mut(&mut self, edge: i32) -> Result<&mut Vec<PaveBlockHandle>, Failure> {
        let reference = self.shape_info(edge)?.reference();
        let slot = checked_slot(
            reference,
            self.pave_blocks_pool.len(),
            "BOPDS_DS::PaveBlocks",
        )?;
        Ok(&mut self.pave_blocks_pool[slot])
    }

    fn split_common_part(
        &mut self,
        block: &PaveBlockHandle,
        groups: &mut DataMap<Pair, Vec<PaveBlockHandle>>,
    ) -> Result<(), Failure> {
        let edge = lock_block(block).original_edge();
        let list = self.pave_block_list_mut(edge)?;
        let Some(position) = list.iter().position(|item| Arc::ptr_eq(item, block)) else {
            return Ok(());
        };
        for split in split_block(&list[position]) {
            list.push(Arc::clone(&split));
            let (first, second) = lock_block(&split).indices();
            let key = Pair::new(first, second);
            if let Some(group) = groups.change_seek(&key) {
                group.push(split);
            } else {
                groups.bind(key, vec![split]);
            }
        }
        list.remove(position);
        Ok(())
    }

    fn regroup_coincident_blocks(
        &mut self,
        group: &mut Vec<PaveBlockHandle>,
        faces: &[i32],
        fuzzy_value: f64,
    ) -> Result<(), Failure> {
        while !group.is_empty() {
            let mut coincident: Vec<PaveBlockHandle> = Vec::new();
            let mut position = 0;
            while position < group.len() {
                let accept = match coincident.first() {
                    None => true,
                    Some(head) => self.check_coincidence(&group[position], head, fuzzy_value)?,
                };
                if accept {
                    coincident.push(group.remove(position));
                } else {
                    position += 1;
                }
            }
            self.bind_new_common_block(&coincident, faces);
        }
        Ok(())
    }

    fn bind_new_common_block(&mut self, blocks: &[PaveBlockHandle], faces: &[i32]) {
        let mut common = CommonBlock::new();
        common.set_pave_blocks(blocks);
        common.set_faces(faces);
        let common = common_block_handle(common);
        for block in blocks {
            self.set_common_block(block, &common);
        }
    }
}

fn common_block_parts(
    common: &CommonBlockHandle,
) -> Result<(Vec<PaveBlockHandle>, Vec<i32>), Failure> {
    let common = common
        .lock()
        .expect("common-block lock must not be poisoned");
    common.pave_block1()?;
    Ok((common.pave_blocks().to_vec(), common.faces().to_vec()))
}

fn split_block(block: &PaveBlockHandle) -> Vec<PaveBlockHandle> {
    let mut splits: Vec<PaveBlock> = Vec::new();
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .update(&mut splits, true);
    splits.into_iter().map(pave_block_handle).collect()
}

fn block_range_and_edge(block: &PaveBlockHandle) -> ((f64, f64), i32) {
    let block = lock_block(block);
    (block.range(), block.original_edge())
}

fn lock_block(block: &PaveBlockHandle) -> std::sync::MutexGuard<'_, PaveBlock> {
    block.lock().expect("pave-block lock must not be poisoned")
}
