//! Native iterator links observe original clipping nodes rather than an owning Vec snapshot.
use super::clip_sequence_storage::ClipPlaneNodeLease;
use super::{Graphic3dClipPlane, Graphic3dSequenceOfHClipPlane};
use rustycad_tkernel::ncollection::{Sequence, SequenceNativeCursor};
use rustycad_tkernel::standard::{
    Failure, Handle, SharedState, exceptions::no_such_object, raise_native_failure,
};
use std::sync::{Arc, Weak};

/// Live `Graphic3d_SequenceOfHClipPlane::Iterator`; `let mut it=Graphic3dSequenceOfHClipPlaneIterator::new(&planes)`.
#[derive(Clone, Default)]
pub struct Graphic3dSequenceOfHClipPlaneIterator {
    pub(super) items: Weak<SharedState<Sequence<super::Graphic3dClipPlaneHandleField>>>,
    pub(super) cursor: SequenceNativeCursor,
}
impl Graphic3dSequenceOfHClipPlaneIterator {
    /// Position at the actual first node without owning planes; `Self::new(&planes)`.
    pub fn new(planes: &Graphic3dSequenceOfHClipPlane) -> Self {
        Self {
            items: Arc::downgrade(&planes.clip_plane_items),
            cursor: SequenceNativeCursor::new(&planes.clip_plane_items.read(), true),
        }
    }
    /// Null initializes an empty native iterator; `Self::from_handle(&handle)`.
    pub fn from_handle(planes: &Handle<Graphic3dSequenceOfHClipPlane>) -> Self {
        // The native constructor borrows the handle; no collection ownership is copied.
        planes.get().map(Self::new).unwrap_or_default()
    }
    /// Restart at the actual first node; `iterator.init(&planes)`.
    pub fn init(&mut self, planes: &Graphic3dSequenceOfHClipPlane) {
        // Assignment copies native links into this iterator, without cloning target handles.
        *self = Self::new(planes);
    }
    /// Restart with null producing default links; `iterator.init_handle(&handle)`.
    pub fn init_handle(&mut self, planes: &Handle<Graphic3dSequenceOfHClipPlane>) {
        // Native null Init resets both current and remembered predecessor links.
        *self = Self::from_handle(planes);
    }
    /// Observe the native current link; `iterator.more()`.
    pub fn more(&self) -> bool {
        // Expired Rust storage is safe-empty outside the native valid-reference lifetime.
        self.items.strong_count() != 0 && self.cursor.observer().more()
    }
    /// Advance through the current real node after callbacks; `iterator.next()`.
    pub fn next(&mut self) {
        let Some(items) = self.items.upgrade() else {
            return;
        };
        raise_native_failure(self.cursor.next(&items.read()));
    }
    /// Borrow the actual current handle-field storage without a plane owner; `iterator.value_ref()?`.
    pub fn value_ref(&self) -> Result<ClipPlaneNodeLease, Failure> {
        let items = self.items.upgrade().ok_or_else(missing_clip_cursor)?;
        let items = items.read();
        let field = self
            .cursor
            .position()
            .value(&items)
            .ok_or_else(missing_clip_cursor)?;
        Ok(field.storage_lease())
    }
    /// Explicitly owning convenience snapshot, null at end; `iterator.value()`.
    pub fn value(&self) -> Handle<Graphic3dClipPlane> {
        self.value_ref().map_or_else(
            |_| Handle::default(),
            |field| field.observe().observe_as::<Graphic3dClipPlane>().clone(),
        )
    }
}
fn missing_clip_cursor() -> Failure {
    no_such_object("clipping iterator current node is absent; expected a live current handle field")
}
