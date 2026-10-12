//! A native Edge alias binds the actual explorer slot, including in-place reset/reuse after pop.
use rustycad_tkbrep::{
    topexp::{Explorer, ExplorerShapeSlot},
    topods::Edge,
};
use rustycad_tkernel::standard::{Failure, SharedState};
/// Borrowed edge storage without a traversal-blocking lock; `tool.edge_ref()?.snapshot()?`.
#[derive(Clone, Copy)]
pub struct StdPrsToolRFaceEdgeRef<'a> {
    explorer: &'a SharedState<Explorer>,
    slot: ExplorerShapeSlot,
}
impl<'a> StdPrsToolRFaceEdgeRef<'a> {
    pub(super) fn new(explorer: &'a SharedState<Explorer>, slot: ExplorerShapeSlot) -> Self {
        // This is a reader of real storage, not an independently owned current-edge cache.
        Self { explorer, slot }
    }
    /// Copy metadata only when observed; `let edge = alias.snapshot()?`.
    pub fn snapshot(&self) -> Result<Edge, Failure> {
        let explorer = self.explorer.read();
        // Native reference reads do not repeat TopoDS::Edge validation after that storage changes.
        Ok(Edge::unchecked(
            explorer.reference_shape(self.slot)?.clone(),
        ))
    }
    /// Compare real source slots, independent of their values; `first.is_same_reference(&second)`.
    pub fn is_same_reference(&self, other: &StdPrsToolRFaceEdgeRef<'_>) -> bool {
        // A popped slot is retained and reused by the pinned Explorer implementation.
        std::ptr::eq(self.explorer, other.explorer) && self.slot == other.slot
    }
}
