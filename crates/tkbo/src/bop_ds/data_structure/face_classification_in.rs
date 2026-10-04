//! Face-interior classifications from internal vertices and VF/EF interferences.
use super::{DataStructure, Failure, VertexIndexSet};
use crate::bop_ds::PaveBlockIndexMap;
use rustycad_tkernel::ncollection::Map;
use std::mem;

impl DataStructure {
    /// Appends interior classifications; `ds.face_info_in(face, &mut blocks, &mut vertices)?`.
    pub fn face_info_in(
        &self,
        face: i32,
        blocks: &mut PaveBlockIndexMap,
        vertices: &mut VertexIndexSet,
    ) -> Result<(), Failure> {
        self.add_internal_vertices(face, vertices)?;
        self.add_vertex_face_interferences(face, vertices);
        self.add_edge_face_interferences(face, blocks, vertices)
    }

    /// Replaces the cached In maps when a face record already exists.
    /// `ds.update_face_info_in(face)?`.
    pub fn update_face_info_in(&mut self, face: i32) -> Result<(), Failure> {
        if !self.has_face_info(face)? {
            return Ok(());
        }
        let slot = self.face_info_slot(face)?;
        let mut blocks = mem::take(self.face_info_pool[slot].change_pave_blocks_in());
        let mut vertices = mem::take(self.face_info_pool[slot].change_vertices_in());
        blocks.clear();
        vertices.clear();
        let result = self.face_info_in(face, &mut blocks, &mut vertices);
        *self.face_info_pool[slot].change_pave_blocks_in() = blocks;
        *self.face_info_pool[slot].change_vertices_in() = vertices;
        result
    }

    /// Refreshes In maps for every face in a native index set; `ds.update_face_info_in_map(&faces)?`.
    pub fn update_face_info_in_map(&mut self, faces: &Map<i32>) -> Result<(), Failure> {
        self.initialize_map_faces_in(faces)?;
        self.add_map_vertex_face_records(faces)?;
        self.add_map_edge_face_records(faces)
    }

    /// Refreshes On maps and lazily creates records for a native index set.
    /// `ds.update_face_info_on_map(&faces)?`.
    pub fn update_face_info_on_map(&mut self, faces: &Map<i32>) -> Result<(), Failure> {
        for face in faces.iter().copied() {
            self.refresh_map_face_on(face)?;
        }
        Ok(())
    }

    /// Rebuilds On maps and removes pave blocks without a result edge.
    /// `ds.refine_face_info_on()?`.
    pub fn refine_face_info_on(&mut self) -> Result<(), Failure> {
        for slot in 0..self.face_info_pool.len() {
            self.refine_face_on_slot(slot)?;
        }
        Ok(())
    }

    /// Adds direct face-child vertices after same-domain resolution.
    fn add_internal_vertices(
        &self,
        face: i32,
        vertices: &mut VertexIndexSet,
    ) -> Result<(), Failure> {
        for vertex in self.face_internal_vertices(face)? {
            vertices.add(vertex);
        }
        Ok(())
    }

    /// Adds VF vertices in native record order, resolving their same-domain chains.
    fn add_vertex_face_interferences(&self, face: i32, vertices: &mut VertexIndexSet) {
        for interference in self.interf_vf.iter() {
            if interference.contains(face) {
                let vertex = interference.opposite_index(face);
                vertices.add(self.same_domain_index(vertex));
            }
        }
    }

    /// Adds EF-generated vertices or common-block pave representatives.
    fn add_edge_face_interferences(
        &self,
        face: i32,
        blocks: &mut PaveBlockIndexMap,
        vertices: &mut VertexIndexSet,
    ) -> Result<(), Failure> {
        for interference in self.interf_ef.iter() {
            if !interference.contains(face) {
                continue;
            }
            self.add_edge_face_result(interference, face, blocks, vertices)?;
        }
        Ok(())
    }

    /// Selects the native `IndexNew` vertex branch or common-block pave branch.
    fn add_edge_face_result(
        &self,
        interference: &crate::bop_ds::InterfEF,
        face: i32,
        blocks: &mut PaveBlockIndexMap,
        vertices: &mut VertexIndexSet,
    ) -> Result<(), Failure> {
        if let Some(index) = interference.get_index_new() {
            vertices.add(self.same_domain_index(index));
            return Ok(());
        }
        let edge = interference.opposite_index(face);
        self.add_common_paves(edge, face, blocks)
    }

    /// Adds the first pave block of each common block that contains the face.
    fn add_common_paves(
        &self,
        edge: i32,
        face: i32,
        blocks: &mut PaveBlockIndexMap,
    ) -> Result<(), Failure> {
        for block in self.pave_blocks(edge)? {
            let Some(common) = self.common_block(block) else {
                continue;
            };
            let common = common
                .lock()
                .expect("common-block lock must not be poisoned");
            if common.contains_face(face) {
                blocks.add(common.pave_block1()?.clone());
            }
        }
        Ok(())
    }

    /// Lazily creates each listed record, clears only its In fields and adds internal vertices.
    fn initialize_map_faces_in(&mut self, faces: &Map<i32>) -> Result<(), Failure> {
        for face in faces.iter().copied() {
            let slot = self.ensure_face_info_entry(face)?;
            self.clear_face_info_in(slot);
            let vertices = self.face_internal_vertices(face)?;
            for vertex in vertices {
                self.face_info_pool[slot].change_vertices_in().add(vertex);
            }
        }
        Ok(())
    }

    /// Clears an existing record's In maps without changing its On or section maps.
    fn clear_face_info_in(&mut self, slot: usize) {
        self.face_info_pool[slot].change_pave_blocks_in().clear();
        self.face_info_pool[slot].change_vertices_in().clear();
    }

    /// Adds VF map-overload records using native `Index2` face selection.
    fn add_map_vertex_face_records(&mut self, faces: &Map<i32>) -> Result<(), Failure> {
        for interference in self.interf_vf.iter() {
            let face = interference.index2();
            if !faces.contains(&face) {
                continue;
            }
            let slot = self.face_info_slot(face)?;
            let vertex = self.same_domain_index(interference.index1());
            self.face_info_pool[slot].change_vertices_in().add(vertex);
        }
        Ok(())
    }

    /// Adds EF map-overload records using native `Index1` edge and `Index2` face roles.
    fn add_map_edge_face_records(&mut self, faces: &Map<i32>) -> Result<(), Failure> {
        let records: Vec<_> = self
            .interf_ef
            .iter()
            .map(|record| (record.index1(), record.index2(), record.get_index_new()))
            .collect();
        for (edge, face, new_index) in records {
            if !faces.contains(&face) {
                continue;
            }
            let slot = self.face_info_slot(face)?;
            self.add_map_edge_face_record(edge, face, new_index, slot)?;
        }
        Ok(())
    }

    /// Adds one generated vertex or its common-block pave representatives to a face record.
    fn add_map_edge_face_record(
        &mut self,
        edge: i32,
        face: i32,
        new_index: Option<i32>,
        slot: usize,
    ) -> Result<(), Failure> {
        if let Some(index) = new_index {
            let vertex = self.same_domain_index(index);
            self.face_info_pool[slot].change_vertices_in().add(vertex);
            return Ok(());
        }
        let mut blocks = PaveBlockIndexMap::default();
        self.add_common_paves(edge, face, &mut blocks)?;
        for block in blocks.iter() {
            self.face_info_pool[slot]
                .change_pave_blocks_in()
                .add(block.clone());
        }
        Ok(())
    }

    /// Refreshes one face's On maps before its RefineFaceInfoOn filtering step.
    fn refresh_map_face_on(&mut self, face: i32) -> Result<(), Failure> {
        let slot = self.ensure_face_info_entry(face)?;
        let mut blocks = mem::take(self.face_info_pool[slot].change_pave_blocks_on());
        let mut vertices = mem::take(self.face_info_pool[slot].change_vertices_on());
        blocks.clear();
        vertices.clear();
        let result = self.face_info_on(face, &mut blocks, &mut vertices);
        *self.face_info_pool[slot].change_pave_blocks_on() = blocks;
        *self.face_info_pool[slot].change_vertices_on() = vertices;
        result
    }

    /// Refreshes one cached On set and removes block handles without a generated edge.
    fn refine_face_on_slot(&mut self, slot: usize) -> Result<(), Failure> {
        let face = self.face_info_pool[slot].index();
        self.update_face_info_on(face)?;
        let blocks = self.face_info_pool[slot].change_pave_blocks_on();
        for index in (1..=blocks.extent()).rev() {
            let block = blocks.find_key(index)?.clone();
            if !block
                .lock()
                .expect("pave-block lock must not be poisoned")
                .has_edge()
            {
                blocks.remove_from_index(index)?;
            }
        }
        Ok(())
    }
}
