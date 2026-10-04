//! Lazy face classifications preserve the native distinction from edge pool initialization.
use super::{
    DataStructure, FaceInfo, Failure, ShapeEnum, ShapeIterator, VertexIndexSet, checked_slot,
};
use crate::bop_ds::PaveBlockIndexMap;
use std::sync::LazyLock;

static EMPTY_FACE_INFO: LazyLock<FaceInfo> = LazyLock::new(FaceInfo::default);

impl DataStructure {
    /// Reports whether the shape has a pool reference; `ds.has_face_info(face)?`.
    pub fn has_face_info(&self, index: i32) -> Result<bool, Failure> {
        Ok(self.shape_info(index)?.has_reference())
    }

    /// Borrows cached classification or the shared empty record; `ds.face_info(face)?`.
    pub fn face_info(&self, index: i32) -> Result<&FaceInfo, Failure> {
        if !self.has_face_info(index)? {
            return Ok(&EMPTY_FACE_INFO);
        }
        Ok(&self.face_info_pool[self.face_info_slot(index)?])
    }

    /// Initializes a missing face record before mutable access; `ds.change_face_info(face)?`.
    pub fn change_face_info(&mut self, index: i32) -> Result<&mut FaceInfo, Failure> {
        if !self.has_face_info(index)? {
            self.init_face_info(index)?;
        }
        let slot = self.face_info_slot(index)?;
        Ok(&mut self.face_info_pool[slot])
    }

    /// Appends and initializes a classification record; `ds.init_face_info(face)?`.
    pub fn init_face_info(&mut self, index: i32) -> Result<(), Failure> {
        let slot = self.face_info_pool.len() as i32;
        self.change_shape_info(index)?.set_reference(slot);
        let mut info = FaceInfo::default();
        info.set_index(index);
        self.face_info_pool.push(info);
        self.init_face_info_in(index)?;
        self.update_face_info_on(index)
    }

    /// Adds direct vertex children to an existing record; `ds.init_face_info_in(face)?`.
    pub fn init_face_info_in(&mut self, index: i32) -> Result<(), Failure> {
        if !self.has_face_info(index)? {
            return Ok(());
        }
        let vertices = self.face_internal_vertices(index)?;
        let slot = self.face_info_slot(index)?;
        for vertex in vertices {
            self.face_info_pool[slot].change_vertices_in().add(vertex);
        }
        Ok(())
    }

    /// Rebuilds On maps without initializing edge pave blocks; `ds.update_face_info_on(face)?`.
    pub fn update_face_info_on(&mut self, index: i32) -> Result<(), Failure> {
        if !self.has_face_info(index)? {
            return Ok(());
        }
        let slot = self.face_info_slot(index)?;
        let mut blocks = std::mem::take(self.face_info_pool[slot].change_pave_blocks_on());
        let mut vertices = std::mem::take(self.face_info_pool[slot].change_vertices_on());
        blocks.clear();
        vertices.clear();
        let result = self.face_info_on(index, &mut blocks, &mut vertices);
        *self.face_info_pool[slot].change_pave_blocks_on() = blocks;
        *self.face_info_pool[slot].change_vertices_on() = vertices;
        result
    }

    /// Appends boundary classifications to caller maps; `ds.face_info_on(face, &mut blocks, &mut vertices)?`.
    pub fn face_info_on(
        &self,
        index: i32,
        blocks: &mut PaveBlockIndexMap,
        vertices: &mut VertexIndexSet,
    ) -> Result<(), Failure> {
        for child in self.shape_info(index)?.sub_shapes() {
            if self.shape_info(*child)?.shape_type() == ShapeEnum::Edge {
                self.face_boundary_blocks(*child, blocks, vertices)?;
            } else {
                vertices.add(self.same_domain_index(*child));
            }
        }
        Ok(())
    }

    fn face_boundary_blocks(
        &self,
        edge: i32,
        blocks: &mut PaveBlockIndexMap,
        vertices: &mut VertexIndexSet,
    ) -> Result<(), Failure> {
        for block in self.pave_blocks(edge)? {
            let (first, last) = block
                .lock()
                .expect("pave-block lock must not be poisoned")
                .indices();
            vertices.add(first);
            vertices.add(last);
            blocks.add(self.real_pave_block(block)?);
        }
        Ok(())
    }

    fn face_internal_vertices(&self, index: i32) -> Result<Vec<i32>, Failure> {
        let mut vertices = Vec::new();
        let mut children = ShapeIterator::new(self.shape(index)?, true, true);
        while children.more() {
            let child = children.value()?;
            if child.shape_type()? == ShapeEnum::Vertex {
                vertices.push(self.same_domain_index(self.index(child)));
            }
            children.next();
        }
        Ok(vertices)
    }

    fn face_info_slot(&self, index: i32) -> Result<usize, Failure> {
        checked_slot(
            self.shape_info(index)?.reference(),
            self.face_info_pool.len(),
            "BOPDS_DS::FaceInfo",
        )
    }
}
