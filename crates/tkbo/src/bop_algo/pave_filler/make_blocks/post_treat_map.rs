//! The mapping of the nested pave filler results back into the DS (`PostTreatFF`).
use super::super::{BOPAlgoPaveFiller, missing_structure};
use super::MakeBlocksState;
use super::post_treat::PostTreatContext;
use crate::bop_algo::tools::compute_tolerance_of_cb;
use crate::bop_ds::{
    CommonBlockHandle, DataStructure, Pave, PaveBlock, PaveBlockHandle, pave_block_handle,
};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    /// Maps one nested argument back; returns vertices to append to the argument list when a
    /// section edge became a micro edge.
    pub(super) fn map_nested_shape(
        &mut self,
        shape: &Shape,
        context: &mut PostTreatContext<'_>,
    ) -> Result<Option<Vec<Shape>>, Failure> {
        let nested = context
            .nested
            .data_structure()
            .ok_or_else(missing_structure)?;
        let index = nested.index(shape);
        match nested.shape_info(index)?.shape_type() {
            ShapeEnum::Vertex => {
                self.map_nested_vertex(shape, index, nested, context.state)?;
                Ok(None)
            }
            ShapeEnum::Edge => self.map_nested_edge(shape, index, context),
            _ => Ok(None),
        }
    }
    pub(super) fn map_nested_vertex(
        &mut self,
        shape: &Shape,
        index: i32,
        nested: &DataStructure,
        state: &mut MakeBlocksState,
    ) -> Result<(), Failure> {
        let is_point = state.sections.couples.contains(shape);
        let mut representative = index;
        let vertex = if nested.has_shape_sd_into(index, &mut representative) {
            nested.shape(representative)?.clone()
        } else {
            shape.clone()
        };
        let mut target = self.structure_mut()?.index(&vertex);
        if target < 0 {
            target = self.append_shape_of_type(&vertex, ShapeEnum::Vertex)?;
        }
        if !is_point {
            let source = self.structure_mut()?.index(shape);
            if source != target {
                state.new_sd.bind(source, target);
                self.structure_mut()?.add_shape_sd(source, target);
            }
            return Ok(());
        }
        let couple = state.sections.couples.find_from_key(shape)?.clone();
        let structure = self.structure_mut()?;
        structure.interf_ff()[couple.interference_index() as usize].change_points()
            [couple.index() as usize]
            .set_index(target);
        Ok(())
    }
    pub(super) fn map_nested_edge(
        &mut self,
        shape: &Shape,
        index: i32,
        context: &mut PostTreatContext<'_>,
    ) -> Result<Option<Vec<Shape>>, Failure> {
        let nested = context
            .nested
            .data_structure()
            .ok_or_else(missing_structure)?;
        let has_blocks = nested.has_pave_blocks(index)?;
        let couple = context.state.sections.couples.find_from_key(shape)?.clone();
        let curve = (
            couple.interference_index() as usize,
            couple.index() as usize,
        );
        let block = couple
            .pave_block1()
            .cloned()
            .expect("section edges carry a pave block");
        let old = block.lock().expect("pave-block lock").has_edge();
        if old {
            context
                .state
                .existing_edges
                .bind(Arc::clone(&block), Vec::new());
        }
        if !has_blocks {
            if old {
                context
                    .state
                    .existing_edges
                    .change_find(&block)?
                    .push(Arc::clone(&block));
            } else {
                let edge = self.append_shape_of_type(shape, ShapeEnum::Edge)?;
                block.lock().expect("pave-block lock").set_edge(edge);
            }
            return Ok(None);
        }
        let images: Vec<PaveBlockHandle> = nested.pave_blocks(index)?.to_vec();
        let micro = images.is_empty()
            || (images.len() == 1 && !images[0].lock().expect("pave-block lock").has_shrunk_data());
        if micro {
            remove_curve_block(self.ff_curve_mut(curve)?.change_pave_blocks(), &block);
            // The edge became a micro edge: its vertices are checked for same domain.
            let mut vertices = Vec::new();
            let mut items = ShapeIterator::new(shape, true, true);
            while items.more() {
                vertices.push(items.value()?.clone());
                items.next();
            }
            return Ok(Some(vertices));
        }
        if !old {
            remove_curve_block(self.ff_curve_mut(curve)?.change_pave_blocks(), &block);
        }
        for image in images {
            self.map_nested_block(&image, &block, old, curve, context)?;
        }
        Ok(None)
    }
    pub(super) fn map_nested_block(
        &mut self,
        image: &PaveBlockHandle,
        block: &PaveBlockHandle,
        old: bool,
        curve: super::CurveRef,
        context: &mut PostTreatContext<'_>,
    ) -> Result<(), Failure> {
        let nested = context
            .nested
            .data_structure()
            .ok_or_else(missing_structure)?;
        let real = nested.real_pave_block(image)?;
        let paves = {
            let locked = image.lock().expect("pave-block lock");
            [locked.pave1(), locked.pave2()]
        };
        let block_paves = {
            let locked = block.lock().expect("pave-block lock");
            [locked.pave1(), locked.pave2()]
        };
        for (position, pave) in paves.iter().enumerate() {
            self.map_nested_pave(
                *pave,
                block_paves[position],
                nested,
                &mut context.state.new_sd,
            )?;
        }
        let edge_shape = nested
            .shape(real.lock().expect("pave-block lock").edge())?
            .clone();
        let mut edge = self.structure_mut()?.index(&edge_shape);
        if edge < 0 {
            edge = self.append_shape_of_type(&edge_shape, ShapeEnum::Edge)?;
        }
        if let Some(common) = nested.common_block(&real) {
            let tolerance = self.common_block_tolerance(&common, context)?;
            let stored = self.ff_curve_mut(curve)?;
            if stored.tolerance() < tolerance {
                stored.set_tolerance(tolerance);
            }
        }
        let result = self.edge_pave_block(edge, &real, nested, context)?;
        if old {
            result
                .lock()
                .expect("pave-block lock")
                .set_original_edge(block.lock().expect("pave-block lock").original_edge());
            context
                .state
                .existing_edges
                .change_find(block)?
                .push(result);
        } else {
            self.ff_curve_mut(curve)?.change_pave_blocks().push(result);
        }
        Ok(())
    }
    /// Maps a pave vertex of the nested DS; binds the section block's vertex to it as
    /// same-domain when they sit at the same parameter or are same domain in the nested DS.
    pub(super) fn map_nested_pave(
        &mut self,
        pave: Pave,
        block_pave: Pave,
        nested: &DataStructure,
        new_sd: &mut DataMap<i32, i32>,
    ) -> Result<(), Failure> {
        let vertex_shape = nested.shape(pave.index())?.clone();
        let mut target = self.structure_mut()?.index(&vertex_shape);
        if target < 0 {
            target = self.append_shape_of_type(&vertex_shape, ShapeEnum::Vertex)?;
        }
        if block_pave.index() == target {
            return Ok(());
        }
        let bind = if block_pave.parameter() == pave.parameter() {
            true
        } else {
            let old_shape = self.structure_mut()?.shape(block_pave.index())?.clone();
            let mut sd = -1;
            nested.has_shape_sd_into(nested.index(&old_shape), &mut sd) && sd == pave.index()
        };
        if bind {
            new_sd.bind(block_pave.index(), target);
            self.structure_mut()?
                .add_shape_sd(block_pave.index(), target);
        }
        Ok(())
    }
    pub(super) fn common_block_tolerance(
        &self,
        common: &CommonBlockHandle,
        context: &mut PostTreatContext<'_>,
    ) -> Result<f64, Failure> {
        let key = Arc::as_ptr(common) as usize;
        if let Some(tolerance) = context.tolerances.seek(&key) {
            return Ok(*tolerance);
        }
        let nested = context
            .nested
            .data_structure()
            .ok_or_else(missing_structure)?;
        let tolerance = compute_tolerance_of_cb(Some(common), nested, context.nested.context())?;
        context.tolerances.bind(key, tolerance);
        Ok(tolerance)
    }
    /// One pave block per new edge, with the real block's paves mapped to this DS.
    pub(super) fn edge_pave_block(
        &self,
        edge: i32,
        real: &PaveBlockHandle,
        nested: &DataStructure,
        context: &mut PostTreatContext<'_>,
    ) -> Result<PaveBlockHandle, Failure> {
        if let Some(block) = context.edge_blocks.seek(&edge) {
            return Ok(Arc::clone(block));
        }
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let (first, last) = {
            let locked = real.lock().expect("pave-block lock");
            (locked.pave1(), locked.pave2())
        };
        let mut block = PaveBlock::default();
        block.set_pave1(Pave::new(
            structure.index(nested.shape(first.index())?),
            first.parameter(),
        ));
        block.set_pave2(Pave::new(
            structure.index(nested.shape(last.index())?),
            last.parameter(),
        ));
        block.set_edge(edge);
        let block = pave_block_handle(block);
        context.edge_blocks.bind(edge, Arc::clone(&block));
        Ok(block)
    }
}

fn remove_curve_block(blocks: &mut Vec<PaveBlockHandle>, block: &PaveBlockHandle) {
    if let Some(position) = blocks.iter().position(|item| Arc::ptr_eq(item, block)) {
        blocks.remove(position);
    }
}
