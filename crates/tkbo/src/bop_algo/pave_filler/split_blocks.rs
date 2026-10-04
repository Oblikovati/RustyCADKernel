use super::BOPAlgoPaveFiller;
use crate::bop_ds::{
    CommonBlock, CommonBlockHandle, Pair, PaveBlockHandle, common_block_handle, pave_block_handle,
};
use crate::bop_tools::compute_vertices;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorCurve,
    topods::{Edge, Vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use std::sync::Arc;

struct CommonBlockUpdate {
    original: CommonBlockHandle,
    split_blocks: Vec<PaveBlockHandle>,
}

#[derive(Default)]
struct SplitBlockChanges {
    common_updates: Vec<CommonBlockUpdate>,
    merged_pairs: Vec<Pair>,
    vertices_to_initialize: Vec<i32>,
}

impl BOPAlgoPaveFiller {
    pub(super) fn split_intersected_edges(&mut self, edges: &[i32]) -> Result<(), Failure> {
        let mut changes = SplitBlockChanges::default();
        for edge in edges {
            self.split_edge_blocks(*edge, &mut changes)?;
        }
        self.rebuild_common_blocks(changes.common_updates)?;
        for vertex in changes.vertices_to_initialize {
            self.init_vertex_paves(vertex)?;
        }
        Ok(())
    }

    fn split_edge_blocks(
        &mut self,
        edge: i32,
        changes: &mut SplitBlockChanges,
    ) -> Result<(), Failure> {
        let source_blocks = self.edge_blocks(edge)?;
        let mut replacement = Vec::new();
        for block in source_blocks {
            self.split_one_block(edge, block, changes, &mut replacement)?;
        }
        self.set_edge_blocks(edge, replacement)?;
        Ok(())
    }

    fn edge_blocks(&self, edge: i32) -> Result<Vec<PaveBlockHandle>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::SplitPaveBlocks requires initialized data structure")
        })?;
        Ok(structure.pave_blocks(edge)?.to_vec())
    }

    fn set_edge_blocks(&mut self, edge: i32, blocks: Vec<PaveBlockHandle>) -> Result<(), Failure> {
        let structure = self.data_structure.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::SplitPaveBlocks requires initialized data structure")
        })?;
        *structure.change_pave_blocks(edge)? = blocks;
        Ok(())
    }

    fn split_one_block(
        &mut self,
        edge: i32,
        block: PaveBlockHandle,
        changes: &mut SplitBlockChanges,
        output: &mut Vec<PaveBlockHandle>,
    ) -> Result<(), Failure> {
        if !block_needs_split(&block) {
            output.push(block);
            return Ok(());
        }
        let common = self.common_block_for(&block);
        let split_blocks = split_source_block(&block);
        for split in split_blocks {
            self.prepare_split_block(edge, split, common.as_ref(), changes, output)?;
        }
        Ok(())
    }

    fn common_block_for(&self, block: &PaveBlockHandle) -> Option<CommonBlockHandle> {
        self.data_structure.as_ref()?.common_block(block)
    }

    fn prepare_split_block(
        &mut self,
        edge: i32,
        block: PaveBlockHandle,
        common: Option<&CommonBlockHandle>,
        changes: &mut SplitBlockChanges,
        output: &mut Vec<PaveBlockHandle>,
    ) -> Result<(), Failure> {
        self.update_split_endpoints(&block)?;
        self.fill_split_block_shrunk_data(edge, &block)?;
        if self.merge_invalid_split_block(
            &block,
            &mut changes.merged_pairs,
            &mut changes.vertices_to_initialize,
        )? {
            return Ok(());
        }
        output.push(Arc::clone(&block));
        append_common_block_split(&mut changes.common_updates, common, &block);
        Ok(())
    }

    fn update_split_endpoints(&self, block: &PaveBlockHandle) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::SplitPaveBlocks requires initialized data structure")
        })?;
        structure.update_pave_block_with_sd_vertices(block);
        Ok(())
    }

    fn fill_split_block_shrunk_data(
        &self,
        edge_index: i32,
        block: &PaveBlockHandle,
    ) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::SplitPaveBlocks requires initialized data structure")
        })?;
        let edge = Edge::unchecked(structure.shape(edge_index)?.clone());
        super::shrunk_data::fill_shrunk_block(structure, &self.options, &self.context, &edge, block)
    }

    fn merge_invalid_split_block(
        &mut self,
        block: &PaveBlockHandle,
        merged_pairs: &mut Vec<Pair>,
        vertices_to_initialize: &mut Vec<i32>,
    ) -> Result<bool, Failure> {
        let (first, last) = block_endpoints(block);
        let needs_check = !block_has_usable_range(block) || !block_is_splittable(block);
        if !needs_check {
            return Ok(false);
        }
        if first == last {
            return Ok(true);
        }
        if !should_merge_endpoints(self, first, last, block, merged_pairs)? {
            return Ok(false);
        }
        self.make_same_domain_vertices(&[first, last])?;
        append_unique(vertices_to_initialize, first);
        append_unique(vertices_to_initialize, last);
        Ok(true)
    }

    fn init_vertex_paves(&mut self, vertex: i32) -> Result<(), Failure> {
        let structure = self.data_structure.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::SplitPaveBlocks requires initialized data structure")
        })?;
        structure.init_pave_blocks_for_vertex(vertex)
    }

    fn rebuild_common_blocks(&mut self, updates: Vec<CommonBlockUpdate>) -> Result<(), Failure> {
        for update in updates {
            self.rebuild_common_block(update)?;
        }
        Ok(())
    }

    fn rebuild_common_block(&mut self, update: CommonBlockUpdate) -> Result<(), Failure> {
        let (faces, is_closed) = common_block_source(&update.original)?;
        let groups = if is_closed {
            self.closed_common_block_groups(update.split_blocks)?
        } else {
            open_common_block_groups(update.split_blocks)
        };
        for blocks in groups {
            self.bind_common_block(blocks, &faces)?;
        }
        Ok(())
    }

    fn closed_common_block_groups(
        &self,
        blocks: Vec<PaveBlockHandle>,
    ) -> Result<Vec<Vec<PaveBlockHandle>>, Failure> {
        let mut pair_groups = Vec::new();
        for block in blocks {
            append_pair_group(&mut pair_groups, block);
        }
        let mut groups = Vec::new();
        for (_, mut candidates) in pair_groups {
            while !candidates.is_empty() {
                let first = candidates.remove(0);
                let (center, tolerance) = self.closed_block_probe(&first)?;
                let mut group = vec![Arc::clone(&first)];
                collect_coincident_blocks(self, &center, tolerance, &mut candidates, &mut group)?;
                groups.push(group);
            }
        }
        Ok(groups)
    }

    fn closed_block_probe(&self, block: &PaveBlockHandle) -> Result<(Point3, f64), Failure> {
        let (edge_index, first, last) = block_edge_range(block);
        let edge = self.source_edge(edge_index)?;
        let curve = BRepAdaptorCurve::with_edge(&edge)?;
        let point = curve.value((first + last) / 2.0)?;
        let tolerance = BRepTool::max_tolerance(&edge.clone().into_shape(), ShapeEnum::Vertex)?;
        Ok((point, tolerance))
    }

    fn source_edge(&self, index: i32) -> Result<Edge, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::SplitPaveBlocks requires initialized data structure")
        })?;
        Ok(Edge::unchecked(structure.shape(index)?.clone()))
    }

    fn bind_common_block(
        &mut self,
        blocks: Vec<PaveBlockHandle>,
        faces: &[i32],
    ) -> Result<(), Failure> {
        if blocks.is_empty() {
            return Ok(());
        }
        let mut common = CommonBlock::default();
        common.set_pave_blocks(&blocks);
        common.set_faces(faces);
        let common = common_block_handle(common);
        let structure = self.data_structure.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::SplitPaveBlocks requires initialized data structure")
        })?;
        for block in blocks {
            structure.set_common_block(&block, &common);
        }
        Ok(())
    }
}

fn block_needs_split(block: &PaveBlockHandle) -> bool {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .is_to_update()
}

fn split_source_block(block: &PaveBlockHandle) -> Vec<PaveBlockHandle> {
    let mut source = block.lock().expect("pave-block lock must not be poisoned");
    let mut split = Vec::new();
    source.update(&mut split, true);
    split.into_iter().map(pave_block_handle).collect()
}

fn block_endpoints(block: &PaveBlockHandle) -> (i32, i32) {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .indices()
}

fn block_has_usable_range(block: &PaveBlockHandle) -> bool {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .has_shrunk_data()
}

fn block_is_splittable(block: &PaveBlockHandle) -> bool {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .is_splittable()
}

fn should_merge_endpoints(
    filler: &BOPAlgoPaveFiller,
    first: i32,
    last: i32,
    block: &PaveBlockHandle,
    merged_pairs: &mut Vec<Pair>,
) -> Result<bool, Failure> {
    if block_has_usable_range(block) && !endpoints_interfere(filler, first, last)? {
        return Ok(false);
    }
    let pair = Pair::new(first, last);
    if merged_pairs.contains(&pair) {
        return Ok(true);
    }
    merged_pairs.push(pair);
    Ok(true)
}

fn endpoints_interfere(filler: &BOPAlgoPaveFiller, first: i32, last: i32) -> Result<bool, Failure> {
    let structure = filler.data_structure.as_ref().ok_or_else(|| {
        Failure::new("BOPAlgo_PaveFiller::SplitPaveBlocks requires initialized data structure")
    })?;
    let first_vertex = Vertex::unchecked(structure.shape(first)?.clone());
    let last_vertex = Vertex::unchecked(structure.shape(last)?.clone());
    Ok(compute_vertices(&first_vertex, &last_vertex, filler.options.fuzzy_value())? == 0)
}

fn append_common_block_split(
    updates: &mut Vec<CommonBlockUpdate>,
    original: Option<&CommonBlockHandle>,
    split: &PaveBlockHandle,
) {
    let Some(original) = original else {
        return;
    };
    if let Some(update) = updates
        .iter_mut()
        .find(|update| Arc::ptr_eq(&update.original, original))
    {
        update.split_blocks.push(Arc::clone(split));
        return;
    }
    updates.push(CommonBlockUpdate {
        original: Arc::clone(original),
        split_blocks: vec![Arc::clone(split)],
    });
}

fn common_block_source(block: &CommonBlockHandle) -> Result<(Vec<i32>, bool), Failure> {
    let block = block
        .lock()
        .expect("common-block lock must not be poisoned");
    let first = block
        .pave_block1()?
        .lock()
        .expect("pave-block lock must not be poisoned");
    let (first_vertex, last_vertex) = first.indices();
    Ok((block.faces().to_vec(), first_vertex == last_vertex))
}

fn open_common_block_groups(blocks: Vec<PaveBlockHandle>) -> Vec<Vec<PaveBlockHandle>> {
    let mut groups: Vec<(Pair, Vec<PaveBlockHandle>)> = Vec::new();
    for block in blocks {
        let pair = Pair::new(block_endpoints(&block).0, block_endpoints(&block).1);
        if let Some((_, group)) = groups.iter_mut().find(|(key, _)| *key == pair) {
            group.push(block);
        } else {
            groups.push((pair, vec![block]));
        }
    }
    groups.into_iter().map(|(_, group)| group).collect()
}

fn append_pair_group(groups: &mut Vec<(Pair, Vec<PaveBlockHandle>)>, block: PaveBlockHandle) {
    let (first, last) = block_endpoints(&block);
    let pair = Pair::new(first, last);
    if let Some((_, group)) = groups.iter_mut().find(|(key, _)| *key == pair) {
        group.push(block);
        return;
    }
    groups.push((pair, vec![block]));
}

fn collect_coincident_blocks(
    filler: &BOPAlgoPaveFiller,
    center: &Point3,
    first_tolerance: f64,
    remaining: &mut Vec<PaveBlockHandle>,
    group: &mut Vec<PaveBlockHandle>,
) -> Result<(), Failure> {
    let context = filler.context.get().ok_or_else(|| {
        Failure::new("BOPAlgo_PaveFiller::SplitPaveBlocks requires initialized context")
    })?;
    let mut cursor = 0;
    while cursor < remaining.len() {
        if is_coincident_block(filler, context, center, first_tolerance, &remaining[cursor])? {
            group.push(remaining.remove(cursor));
        } else {
            cursor += 1;
        }
    }
    Ok(())
}

fn is_coincident_block(
    filler: &BOPAlgoPaveFiller,
    context: &crate::int_tools::Context,
    center: &Point3,
    first_tolerance: f64,
    block: &PaveBlockHandle,
) -> Result<bool, Failure> {
    let (edge_index, first, last) = block_edge_range(block);
    let edge = filler.source_edge(edge_index)?;
    let tolerance = BRepTool::max_tolerance(&edge.clone().into_shape(), ShapeEnum::Vertex)?;
    let mut parameter = -1.0;
    let mut distance = -1.0;
    let status = context.compute_pe(
        *center,
        first_tolerance + tolerance + filler.options.fuzzy_value(),
        &edge,
        &mut parameter,
        &mut distance,
    )?;
    Ok(status == 0 && parameter > first && parameter < last)
}

fn block_edge_range(block: &PaveBlockHandle) -> (i32, f64, f64) {
    let block = block.lock().expect("pave-block lock must not be poisoned");
    let (first, last) = block.range();
    (block.original_edge(), first, last)
}

fn append_unique(values: &mut Vec<i32>, value: i32) {
    if !values.contains(&value) {
        values.push(value);
    }
}
