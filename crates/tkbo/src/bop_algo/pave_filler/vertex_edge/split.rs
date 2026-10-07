//! `BOPAlgo_PaveFiller::SplitPaveBlocks` and its static `MakeNewCommonBlock`.
use super::super::{BOPAlgoPaveFiller, missing_structure};
use crate::bop_ds::{
    CommonBlock, CommonBlockHandle, CommonBlockHandleIdentity, Pair, PaveBlockHandle,
    common_block_handle, pave_block_handle,
};
use crate::bop_tools::{compute_vertices, point_on_edge};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::ncollection::{DefaultHashPolicy, IndexedDataMap, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use std::sync::Arc;

type NewBlocksOfCommonBlocks =
    IndexedDataMap<CommonBlockHandle, Vec<PaveBlockHandle>, CommonBlockHandleIdentity>;

impl BOPAlgoPaveFiller {
    /// Splits the pave blocks of the given edges by their extra paves; a new block without a
    /// valid range (or a tiny one whose vertices interfere) unifies its vertices instead, and
    /// the split parts of common blocks are regrouped into new common blocks.
    /// `filler.split_pave_blocks(&edges, true)?`.
    pub fn split_pave_blocks(
        &mut self,
        edges: &Map<i32>,
        add_interfs: bool,
    ) -> Result<(), Failure> {
        let mut unified: Map<Pair> = Map::new(1);
        let mut new_blocks = NewBlocksOfCommonBlocks::with_hasher(CommonBlockHandleIdentity, 1);
        let mut vertices_to_init: Map<i32> = Map::new(1);
        for edge in edges.iter().copied() {
            let mut list = self.structure_mut()?.change_pave_blocks(edge)?.clone();
            let mut position = 0;
            while position < list.len() {
                let block = Arc::clone(&list[position]);
                if !block.lock().expect("pave-block lock").is_to_update() {
                    position += 1;
                    continue;
                }
                let common = self.structure_mut()?.common_block(&block);
                let mut splits = Vec::new();
                block
                    .lock()
                    .expect("pave-block lock")
                    .update(&mut splits, true);
                for split in splits.into_iter().map(pave_block_handle) {
                    let keep = self.check_split_block(
                        &split,
                        add_interfs,
                        &mut unified,
                        &mut vertices_to_init,
                    )?;
                    if !keep {
                        continue;
                    }
                    list.push(Arc::clone(&split));
                    if let Some(common) = &common {
                        append_new_block(&mut new_blocks, common, split);
                    }
                }
                list.remove(position);
            }
            *self.structure_mut()?.change_pave_blocks(edge)? = list;
        }
        for index in 1..=new_blocks.extent() {
            let common = Arc::clone(new_blocks.find_key(index)?);
            let blocks = new_blocks.find_from_index(index)?.clone();
            self.make_split_common_blocks(&common, &blocks)?;
        }
        for vertex in vertices_to_init.iter().copied() {
            self.structure_mut()?.init_pave_blocks_for_vertex(vertex)?;
        }
        Ok(())
    }

    /// Updates a new split block's vertices and shrunk data; returns false when its vertices
    /// had to be unified (the block is dropped) or when it joins the same vertex twice.
    fn check_split_block(
        &mut self,
        split: &PaveBlockHandle,
        add_interfs: bool,
        unified: &mut Map<Pair>,
        vertices_to_init: &mut Map<i32>,
    ) -> Result<bool, Failure> {
        self.structure_mut()?
            .update_pave_block_with_sd_vertices(split);
        self.fill_shrunk_data_block(split)?;
        let (mut has_valid_range, splittable, (first, last)) = {
            let locked = split.lock().expect("pave-block lock");
            (
                locked.has_shrunk_data(),
                locked.is_splittable(),
                locked.indices(),
            )
        };
        let check_distance = has_valid_range && !splittable;
        if has_valid_range && !check_distance {
            return Ok(true);
        }
        if first == last {
            return Ok(false);
        }
        if check_distance && self.vertices_interfere(first, last)? {
            has_valid_range = false;
        }
        if has_valid_range {
            return Ok(true);
        }
        if unified.add(Pair::new(first, last)) {
            self.make_sd_vertices(&[first, last], add_interfs)?;
            vertices_to_init.add(first);
            vertices_to_init.add(last);
        }
        Ok(false)
    }

    fn vertices_interfere(&self, first: i32, last: i32) -> Result<bool, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let first = Vertex::unchecked(structure.shape(first)?.clone());
        let last = Vertex::unchecked(structure.shape(last)?.clone());
        Ok(compute_vertices(&first, &last, self.options.fuzzy_value())? == 0)
    }

    fn make_split_common_blocks(
        &mut self,
        common: &CommonBlockHandle,
        blocks: &[PaveBlockHandle],
    ) -> Result<(), Failure> {
        let mut by_bounds: IndexedDataMap<Pair, Vec<PaveBlockHandle>, DefaultHashPolicy> =
            IndexedDataMap::with_hasher(DefaultHashPolicy, 1);
        for block in blocks {
            let (first, last) = block.lock().expect("pave-block lock").indices();
            let key = Pair::new(first, last);
            if let Some(list) = by_bounds.change_seek(&key) {
                list.push(Arc::clone(block));
            } else {
                by_bounds.add(key, vec![Arc::clone(block)]);
            }
        }
        let (faces, closed) = {
            let locked = common.lock().expect("common-block lock");
            let (first, last) = locked
                .pave_block1()?
                .lock()
                .expect("pave-block lock")
                .indices();
            (locked.faces().to_vec(), first == last)
        };
        for index in 1..=by_bounds.extent() {
            let mut group = by_bounds.find_from_index(index)?.clone();
            if !closed {
                self.make_new_common_block(&group, &faces)?;
                continue;
            }
            while !group.is_empty() {
                let coinciding = self.take_coinciding_blocks(&mut group)?;
                self.make_new_common_block(&coinciding, &faces)?;
            }
        }
        Ok(())
    }

    /// Takes the first block and every block whose edge passes near the first block's middle
    /// point inside the block's own range (closed common blocks only).
    fn take_coinciding_blocks(
        &self,
        group: &mut Vec<PaveBlockHandle>,
    ) -> Result<Vec<PaveBlockHandle>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let context = self.context.get().ok_or_else(missing_structure)?;
        let head = group.remove(0);
        let (head_edge, head_middle) = {
            let locked = head.lock().expect("pave-block lock");
            let middle = (locked.pave1().parameter() + locked.pave2().parameter()) / 2.;
            (locked.original_edge(), middle)
        };
        let head_shape = Edge::unchecked(structure.shape(head_edge)?.clone());
        let head_tolerance = BRepTool::max_tolerance(&head_shape, ShapeEnum::Vertex)?;
        let mut middle = Point3::default();
        point_on_edge(&head_shape, head_middle, &mut middle)?;
        let mut coinciding = vec![head];
        let mut position = 0;
        while position < group.len() {
            let (edge, range) = {
                let locked = group[position].lock().expect("pave-block lock");
                (
                    locked.original_edge(),
                    (locked.pave1().parameter(), locked.pave2().parameter()),
                )
            };
            let shape = Edge::unchecked(structure.shape(edge)?.clone());
            let tolerance = BRepTool::max_tolerance(&shape, ShapeEnum::Vertex)?;
            let (mut parameter, mut distance) = (0., 0.);
            let limit = head_tolerance + tolerance + self.options.fuzzy_value();
            let status =
                context.compute_pe(middle, limit, &shape, &mut parameter, &mut distance)?;
            if status == 0 && parameter > range.0 && parameter < range.1 {
                coinciding.push(group.remove(position));
                continue;
            }
            position += 1;
        }
        Ok(coinciding)
    }

    /// Native static `MakeNewCommonBlock`: one new common block over the blocks, with faces.
    fn make_new_common_block(
        &mut self,
        blocks: &[PaveBlockHandle],
        faces: &[i32],
    ) -> Result<(), Failure> {
        let mut common = CommonBlock::new();
        common.set_pave_blocks(blocks);
        common.set_faces(faces);
        let common = common_block_handle(common);
        let structure = self.structure_mut()?;
        for block in blocks {
            structure.set_common_block(block, &common);
        }
        Ok(())
    }

    /// Mutably borrows the data structure or fails when Init has not run.
    pub(in crate::bop_algo::pave_filler) fn structure_mut(
        &mut self,
    ) -> Result<&mut crate::bop_ds::DataStructure, Failure> {
        self.data_structure.as_mut().ok_or_else(missing_structure)
    }
}

fn append_new_block(
    new_blocks: &mut NewBlocksOfCommonBlocks,
    common: &CommonBlockHandle,
    split: PaveBlockHandle,
) {
    if let Some(list) = new_blocks.change_seek(common) {
        list.push(split);
        return;
    }
    new_blocks.add(Arc::clone(common), vec![split]);
}
