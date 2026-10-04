//! Face-interior classifications from internal vertices and VF/EF interferences.
use super::{DataStructure, Failure, VertexIndexSet};
use crate::bop_ds::PaveBlockIndexMap;
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
}
