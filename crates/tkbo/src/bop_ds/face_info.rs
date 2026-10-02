//! Per-face pave-block and vertex classifications used by BOPDS_DS.
use super::PaveBlockHandle;
use rustycad_tkernel::ncollection::{HashPolicy, IndexedMap, IndexedMapIter, Map, MapIter};
use rustycad_tkernel::standard::Failure;
use std::fmt::Debug;
use std::sync::Arc;

const INITIAL_BUCKETS: usize = 100;

#[derive(Clone, Copy, Debug, Default)]
struct PaveBlockHandleIdentity;

impl HashPolicy<PaveBlockHandle> for PaveBlockHandleIdentity {
    fn hash(&self, block: &PaveBlockHandle) -> usize {
        Arc::as_ptr(block) as usize
    }

    fn equal(&self, first: &PaveBlockHandle, second: &PaveBlockHandle) -> bool {
        Arc::ptr_eq(first, second)
    }
}

/// An insertion-ordered set of pave-block handles, keyed by shared identity.
pub struct PaveBlockIndexMap(IndexedMap<PaveBlockHandle, PaveBlockHandleIdentity>);

impl Default for PaveBlockIndexMap {
    fn default() -> Self {
        Self(IndexedMap::with_hasher(
            PaveBlockHandleIdentity,
            INITIAL_BUCKETS,
        ))
    }
}

impl PaveBlockIndexMap {
    /// Inserts a handle once and returns its one-based native index.
    pub fn add(&mut self, block: PaveBlockHandle) -> usize {
        self.0.add(block)
    }

    /// Tests whether this exact shared handle is present.
    pub fn contains(&self, block: &PaveBlockHandle) -> bool {
        self.0.contains(block)
    }

    /// Returns the number of unique handles.
    pub fn extent(&self) -> usize {
        self.0.extent()
    }

    /// Reads a handle by one-based index, raising Standard_OutOfRange when absent.
    pub fn find_key(&self, index: usize) -> Result<&PaveBlockHandle, Failure> {
        self.0.find_key(index)
    }

    /// Clears handles while retaining allocated bucket storage.
    pub fn clear(&mut self) {
        self.0.clear(false);
    }

    /// Iterates handles in one-based index order.
    pub fn iter(&self) -> IndexedMapIter<'_, PaveBlockHandle> {
        self.0.iter()
    }
}

/// A unique set of vertex indices backed by OCCT's integer map implementation.
pub struct VertexIndexSet(Map<i32>);

impl Default for VertexIndexSet {
    fn default() -> Self {
        Self(Map::new(INITIAL_BUCKETS))
    }
}

impl VertexIndexSet {
    /// Adds a vertex once and reports whether it was new.
    pub fn add(&mut self, index: i32) -> bool {
        self.0.add(index)
    }

    /// Tests whether the index is present.
    pub fn contains(&self, index: i32) -> bool {
        self.0.contains(&index)
    }

    /// Returns the number of unique vertex indices.
    pub fn extent(&self) -> usize {
        self.0.extent()
    }

    /// Clears all indices while retaining allocated bucket storage.
    pub fn clear(&mut self) {
        self.0.clear(false);
    }

    /// Iterates vertex indices in native bucket traversal order.
    pub fn iter(&self) -> MapIter<'_, i32> {
        self.0.iter()
    }
}

/// `BOPDS_FaceInfo`; stores In, On and section classifications for one face.
pub struct FaceInfo {
    index: i32,
    pave_blocks_in: PaveBlockIndexMap,
    vertices_in: VertexIndexSet,
    pave_blocks_on: PaveBlockIndexMap,
    vertices_on: VertexIndexSet,
    pave_blocks_section: PaveBlockIndexMap,
    vertices_section: VertexIndexSet,
}

impl Default for FaceInfo {
    fn default() -> Self {
        Self {
            index: -1,
            pave_blocks_in: PaveBlockIndexMap::default(),
            vertices_in: VertexIndexSet::default(),
            pave_blocks_on: PaveBlockIndexMap::default(),
            vertices_on: VertexIndexSet::default(),
            pave_blocks_section: PaveBlockIndexMap::default(),
            vertices_section: VertexIndexSet::default(),
        }
    }
}

impl FaceInfo {
    /// Clears In and On maps; native `Clear()` deliberately retains section maps.
    pub fn clear(&mut self) {
        self.pave_blocks_in.clear();
        self.vertices_in.clear();
        self.pave_blocks_on.clear();
        self.vertices_on.clear();
    }

    /// Sets the associated face index without validation.
    pub fn set_index(&mut self, index: i32) {
        self.index = index;
    }

    /// Reads the associated face index; the default sentinel is -1.
    pub fn index(&self) -> i32 {
        self.index
    }

    /// Borrows pave blocks classified inside the face.
    pub fn pave_blocks_in(&self) -> &PaveBlockIndexMap {
        &self.pave_blocks_in
    }

    /// Mutably borrows pave blocks classified inside the face.
    pub fn change_pave_blocks_in(&mut self) -> &mut PaveBlockIndexMap {
        &mut self.pave_blocks_in
    }

    /// Borrows vertices classified inside the face.
    pub fn vertices_in(&self) -> &VertexIndexSet {
        &self.vertices_in
    }

    /// Mutably borrows vertices classified inside the face.
    pub fn change_vertices_in(&mut self) -> &mut VertexIndexSet {
        &mut self.vertices_in
    }

    /// Borrows pave blocks classified on the face.
    pub fn pave_blocks_on(&self) -> &PaveBlockIndexMap {
        &self.pave_blocks_on
    }

    /// Mutably borrows pave blocks classified on the face.
    pub fn change_pave_blocks_on(&mut self) -> &mut PaveBlockIndexMap {
        &mut self.pave_blocks_on
    }

    /// Borrows vertices classified on the face.
    pub fn vertices_on(&self) -> &VertexIndexSet {
        &self.vertices_on
    }

    /// Mutably borrows vertices classified on the face.
    pub fn change_vertices_on(&mut self) -> &mut VertexIndexSet {
        &mut self.vertices_on
    }

    /// Borrows section-edge pave blocks.
    pub fn pave_blocks_section(&self) -> &PaveBlockIndexMap {
        &self.pave_blocks_section
    }

    /// Mutably borrows section-edge pave blocks.
    pub fn change_pave_blocks_section(&mut self) -> &mut PaveBlockIndexMap {
        &mut self.pave_blocks_section
    }

    /// Borrows section vertices.
    pub fn vertices_section(&self) -> &VertexIndexSet {
        &self.vertices_section
    }

    /// Mutably borrows section vertices.
    pub fn change_vertices_section(&mut self) -> &mut VertexIndexSet {
        &mut self.vertices_section
    }
}

impl Debug for PaveBlockIndexMap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_list()
            .entries(self.iter().map(Arc::as_ptr))
            .finish()
    }
}
