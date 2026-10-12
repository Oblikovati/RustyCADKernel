//! Borrow caller handle fields through allocation/destruction callbacks, using real native nodes.
use super::clip_sequence_storage::{clear_native_clip_items, retire_clip_node};
use super::{
    Graphic3dClipPlane, Graphic3dClipPlaneHandleField, Graphic3dClipPlaneSource,
    Graphic3dSequenceOfHClipPlane, Graphic3dSequenceOfHClipPlaneIterator,
};
use rustycad_tkernel::ncollection::SequencePreparedNode;
use rustycad_tkernel::standard::{Failure, Handle, HandleReference, raise_native_failure};

impl Graphic3dSequenceOfHClipPlane {
    /// Borrow the original caller field across native Allocate; `planes.append_reference(&plane)`.
    pub fn append_reference(
        &self,
        plane: &dyn HandleReference<dyn Graphic3dClipPlaneSource>,
    ) -> bool {
        if self.find_clip_node(plane).is_some() {
            return false;
        }
        let lease = self.clip_plane_items.read().allocator().storage_lease();
        let node = SequencePreparedNode::new(&lease, || {
            raise_native_failure(Graphic3dClipPlaneHandleField::from_reference(plane))
        });
        self.clip_plane_items.write().append_prepared(node);
        true
    }
    /// Explicit owning convenience delegates to borrowed native insertion; `planes.append(plane)`.
    pub fn append(&self, plane: Handle<Graphic3dClipPlane>) -> bool {
        // Only this by-value convenience owns its parameter; native mappings use append_reference.
        self.append_reference(&plane)
    }
    /// Borrow the actual caller field and retire outside guards; `planes.remove_reference(&plane)`.
    pub fn remove_reference(
        &self,
        plane: &dyn HandleReference<dyn Graphic3dClipPlaneSource>,
    ) -> bool {
        let Some(mut cursor) = self.find_clip_node(plane) else {
            return false;
        };
        raise_native_failure(self.remove_at(&mut cursor));
        true
    }
    /// Concrete borrowed convenience delegates to native source transport; `planes.remove(&plane)`.
    pub fn remove(&self, plane: &Handle<Graphic3dClipPlane>) -> bool {
        // Projection rereads this same caller slot without introducing a temporary owning handle.
        self.remove_reference(plane)
    }
    /// Inspect live native identity without owning snapshots; `planes.contains_reference(&plane)`.
    pub fn contains_reference(
        &self,
        plane: &dyn HandleReference<dyn Graphic3dClipPlaneSource>,
    ) -> bool {
        // V3d's pre-insertion scan follows the same original nodes and caller handle field.
        self.find_clip_node(plane).is_some()
    }
    fn find_clip_node(
        &self,
        plane: &dyn HandleReference<dyn Graphic3dClipPlaneSource>,
    ) -> Option<Graphic3dSequenceOfHClipPlaneIterator> {
        let mut cursor = Graphic3dSequenceOfHClipPlaneIterator::new(self);
        while cursor.more() {
            if raise_native_failure(cursor.value_ref()).object_address() == plane.object_address() {
                return Some(cursor);
            }
            cursor.next();
        }
        None
    }
    /// Native end is a no-op; unlink and advance before payload callbacks; `planes.remove_at(&mut it)?`.
    pub fn remove_at(
        &self,
        cursor: &mut Graphic3dSequenceOfHClipPlaneIterator,
    ) -> Result<(), Failure> {
        // OCCT 3d097a0 BaseSequence::Remove returns before touching null CurrentNode.
        let node = self
            .clip_plane_items
            .write()
            .detach_native(&mut cursor.cursor)?;
        if let Some(node) = node {
            retire_clip_node(&self.clip_plane_items, node);
        }
        Ok(())
    }
    /// Preserve saved-next and old metadata through callbacks; `planes.clear()`.
    pub fn clear(&self) {
        // The real payload retires while First is registered, then current Free runs guard-free.
        raise_native_failure(clear_native_clip_items(&self.clip_plane_items));
    }
}
impl Drop for Graphic3dSequenceOfHClipPlane {
    fn drop(&mut self) {
        // Native collection destruction uses the same ordered myItems destruction as Clear.
        raise_native_failure(clear_native_clip_items(&self.clip_plane_items));
    }
}
