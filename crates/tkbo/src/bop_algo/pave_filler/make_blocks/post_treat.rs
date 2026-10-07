//! `BOPAlgo_PaveFiller::PostTreatFF`: the section shapes are intersected among themselves by a
//! nested pave filler and its results are mapped back into this data structure.
use super::super::{BOPAlgoPaveFiller, missing_structure};
use super::MakeBlocksState;
use crate::bop_algo::AlertPostTreatFF;
use crate::bop_ds::{PaveBlockHandle, ShapeInfo};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::{DataMap, IndexedMap, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    pub(super) fn post_treat_ff(&mut self, state: &mut MakeBlocksState) -> Result<(), Failure> {
        let count = state.sections.couples.extent();
        if count == 0 {
            return Ok(());
        }
        let unused = self.unused_section_vertices()?;
        if count == 1
            && state.micro_blocks.extent() == 0
            && state.vertices_on_rejected.extent() == 0
            && unused.extent() == 0
        {
            return self.post_treat_single_section(state);
        }
        let mut arguments = self.post_treat_arguments(state, &unused)?;
        let mut nested = BOPAlgoPaveFiller::new();
        nested.set_is_primary(false);
        nested.set_non_destructive(self.non_destructive);
        nested
            .options_mut()
            .set_run_parallel(self.options.run_parallel());
        nested.set_arguments(&arguments.iter().cloned().collect());
        nested.perform();
        if nested.options().has_errors() {
            let alert: AlertHandle = Arc::new(AlertPostTreatFF);
            self.options.add_error(&alert);
            return Ok(());
        }
        let mut tolerances: DataMap<usize, f64> = DataMap::default();
        let mut edge_blocks: DataMap<i32, PaveBlockHandle> = DataMap::default();
        let mut position = 0;
        while position < arguments.len() {
            let shape = arguments[position].clone();
            position += 1;
            if shape.shape_type()? == ShapeEnum::Compound {
                let mut items = ShapeIterator::new(&shape, true, true);
                while items.more() {
                    arguments.push(items.value()?.clone());
                    items.next();
                }
                continue;
            }
            let mut context = PostTreatContext {
                nested: &nested,
                state,
                tolerances: &mut tolerances,
                edge_blocks: &mut edge_blocks,
            };
            if let Some(more) = self.map_nested_shape(&shape, &mut context)? {
                arguments.extend(more);
            }
        }
        self.chain_new_same_domain(&mut state.new_sd)
    }

    /// The vertices put on no curve of any face pair that appear for exactly one pair
    /// (the native "unused vertices" map).
    pub(super) fn unused_section_vertices(
        &self,
    ) -> Result<IndexedMap<Shape, ShapeMapHasher>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut unused: IndexedMap<Shape, ShapeMapHasher> =
            IndexedMap::with_hasher(ShapeMapHasher, 1);
        let mut seen: Map<i32> = Map::new(1);
        for ff in 0..structure.interf_ff_records().len() {
            let faces = structure.interf_ff_records()[ff].indices();
            let (mut stick, mut ef, mut shapes) = (Map::new(1), Map::new(1), Map::new(1));
            self.get_stick_vertices(faces, &mut stick, &mut ef, &mut shapes)?;
            self.remove_used_vertices(ff, &mut stick)?;
            for vertex in stick.iter().copied() {
                let shape = structure.shape(vertex)?.clone();
                if seen.add(vertex) {
                    unused.add(shape);
                } else {
                    unused.remove_key(&shape);
                }
            }
        }
        Ok(unused)
    }

    /// A lone section shape needs no nested intersection: a vertex is appended to the DS, an
    /// existing edge is recorded as is, a new edge is appended.
    pub(super) fn post_treat_single_section(
        &mut self,
        state: &mut MakeBlocksState,
    ) -> Result<(), Failure> {
        let shape = state.sections.couples.find_key(1)?.clone();
        let couple = state.sections.couples.find_from_index(1)?.clone();
        if shape.shape_type()? == ShapeEnum::Vertex {
            let index = self.append_shape_of_type(&shape, ShapeEnum::Vertex)?;
            let structure = self.structure_mut()?;
            structure.interf_ff()[couple.interference_index() as usize].change_points()
                [couple.index() as usize]
                .set_index(index);
            return Ok(());
        }
        let block = couple
            .pave_block1()
            .cloned()
            .expect("section edges carry a pave block");
        if block.lock().expect("pave-block lock").has_edge() {
            state.existing_edges.bind(Arc::clone(&block), vec![block]);
            return Ok(());
        }
        let index = self.append_shape_of_type(&shape, ShapeEnum::Edge)?;
        block.lock().expect("pave-block lock").set_edge(index);
        Ok(())
    }

    /// The nested arguments: new section shapes in reverse order with their same-domain
    /// vertices, the existing edges as one compound, the micro edges' (tolerance-joined)
    /// vertices, and the vertices on rejected blocks and unused vertices.
    pub(super) fn post_treat_arguments(
        &mut self,
        state: &mut MakeBlocksState,
        unused: &IndexedMap<Shape, ShapeMapHasher>,
    ) -> Result<Vec<Shape>, Failure> {
        let builder = BRepBuilder::default();
        let mut arguments: Vec<Shape> = Vec::new();
        let mut existing = Compound::default();
        builder.make_compound(&mut existing);
        let mut existing = existing.into_shape();
        let mut has_existing = false;
        let mut added: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
        for index in (1..=state.sections.couples.extent()).rev() {
            let shape = state.sections.couples.find_key(index)?.clone();
            let block = state
                .sections
                .couples
                .find_from_index(index)?
                .pave_block1()
                .cloned();
            if block.is_some_and(|block| block.lock().expect("pave-block lock").has_edge()) {
                builder.add(&mut existing, &shape)?;
                has_existing = true;
            } else {
                arguments.push(shape.clone());
            }
            self.add_sd_candidates(&shape, &state.new_sd, &mut added, &mut arguments)?;
        }
        if has_existing {
            arguments.push(existing);
        }
        self.add_micro_edge_vertices(state, &mut added, &mut arguments)?;
        for map in [&state.vertices_on_rejected, unused] {
            for vertex in map.iter() {
                let mut vertex = vertex.clone();
                let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
                if let Some(sd) = state.new_sd.seek(&structure.index(&vertex)) {
                    vertex = structure.shape(*sd)?.clone();
                }
                if added.add(vertex.clone()) {
                    arguments.push(vertex);
                }
            }
        }
        Ok(arguments)
    }

    pub(super) fn add_sd_candidates(
        &self,
        shape: &Shape,
        new_sd: &DataMap<i32, i32>,
        added: &mut Map<Shape, ShapeMapHasher>,
        arguments: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut vertices = ShapeIterator::new(shape, true, true);
        while vertices.more() {
            if let Some(sd) = new_sd.seek(&structure.index(vertices.value()?)) {
                let candidate = structure.shape(*sd)?.clone();
                if added.add(candidate.clone()) {
                    arguments.push(candidate);
                }
            }
            vertices.next();
        }
        Ok(())
    }

    /// Adds the micro edges' vertices, growing their tolerances until they meet.
    pub(super) fn add_micro_edge_vertices(
        &mut self,
        state: &MakeBlocksState,
        added: &mut Map<Shape, ShapeMapHasher>,
        arguments: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        for block in state.micro_blocks.iter() {
            let bounds = block.lock().expect("pave-block lock").indices();
            let mut vertices = Vec::with_capacity(2);
            for vertex in [bounds.0, bounds.1] {
                let target = state.new_sd.seek(&vertex).copied().unwrap_or(vertex);
                let shape = self.vertex_shape(target)?;
                if added.add(shape.clone().into_shape()) {
                    arguments.push(shape.clone().into_shape());
                }
                vertices.push(shape);
            }
            if vertices[0].is_same(&vertices[1]) {
                continue;
            }
            let (t1, t2) = (
                BRepTool::vertex_tolerance(&vertices[0])?,
                BRepTool::vertex_tolerance(&vertices[1])?,
            );
            let mut distance = BRepTool::pnt(&vertices[0])?.distance(&BRepTool::pnt(&vertices[1])?);
            distance -= t1 + t2;
            if distance > 0. {
                distance /= 2.;
                builder.update_vertex_tolerance(&vertices[0], t1 + distance)?;
                builder.update_vertex_tolerance(&vertices[1], t2 + distance)?;
            }
        }
        Ok(())
    }

    pub(super) fn append_shape_of_type(
        &mut self,
        shape: &Shape,
        kind: ShapeEnum,
    ) -> Result<i32, Failure> {
        let mut info = ShapeInfo::default();
        info.set_shape_type(kind);
        info.set_shape(shape);
        Ok(self.structure_mut()?.append_shape_info(&info))
    }

    /// Resolves chains in the new same-domain map in its native iteration order.
    pub(super) fn chain_new_same_domain(
        &mut self,
        new_sd: &mut DataMap<i32, i32>,
    ) -> Result<(), Failure> {
        let keys: Vec<i32> = new_sd.items().map(|(key, _)| *key).collect();
        for key in keys {
            let value = *new_sd.find(&key)?;
            if let Some(next) = new_sd.seek(&value).copied() {
                *new_sd.change_find(&key)? = next;
                self.structure_mut()?.add_shape_sd(key, next);
            }
        }
        Ok(())
    }
}

/// Borrowed state of the mapping of the nested results.
pub(super) struct PostTreatContext<'a> {
    pub(super) nested: &'a BOPAlgoPaveFiller,
    pub(super) state: &'a mut MakeBlocksState,
    pub(super) tolerances: &'a mut DataMap<usize, f64>,
    pub(super) edge_blocks: &'a mut DataMap<i32, PaveBlockHandle>,
}

impl BOPAlgoPaveFiller {}
