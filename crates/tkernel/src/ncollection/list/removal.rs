//! Native destructor observations occur after detachment and before final metadata updates.
use super::owner::{catch_list_owner_drop, take_owner};
use super::{
    BaseListRawIterator, BaseListStorage, List, ListCursor, ListLinkNode, topology::ListTopology,
};
use crate::standard::Failure;
struct ListClearMetadata<'a>(&'a mut ListTopology);
impl Drop for ListClearMetadata<'_> {
    fn drop(&mut self) {
        // Reset before remaining owners unwind, so later observations never traverse freed nodes.
        self.0
            .edit(|base| *base = BaseListStorage::new(Some(base.allocator().clone())));
    }
}
impl<T> List<T> {
    /// Removes current and advances to its successor; `list.remove(&mut cursor)?`.
    pub fn remove(&mut self, cursor: &mut ListCursor) -> Result<(), Failure> {
        let raw = self.locate_cursor(*cursor, "Remove")?;
        let (next, outcome) = self.remove_in_phases(raw);
        // Membership repair must not rewrite the safe cursor's remembered predecessor.
        cursor.address = super::address(next);
        resume_drop_outcome(outcome);
        Ok(())
    }
    pub(super) fn remove_in_phases(
        &mut self,
        raw: BaseListRawIterator,
    ) -> (*mut ListLinkNode, std::thread::Result<()>) {
        let phase = self
            .topology
            .edit(|base| unsafe { base.detach_removal(raw) });
        // No lock or metadata borrow survives this boundary into arbitrary user destruction.
        let outcome = catch_list_owner_drop(&mut self.owners, phase.node);
        self.topology.edit(|base| base.finish_removal(phase));
        (phase.next, outcome)
    }
    pub(super) fn clear_owned_chain(&mut self) {
        let mut owners = std::mem::take(&mut self.owners);
        let mut node = self.base_storage().first;
        let reset = ListClearMetadata(&mut self.topology);
        while !node.is_null() {
            let next = unsafe { (*node).next() };
            // Native Clear keeps original metadata until every destructor has run.
            drop(take_owner(&mut owners, node));
            node = next;
        }
        drop(reset);
    }
}
pub(super) fn resume_drop_outcome(outcome: std::thread::Result<()>) {
    if let Err(payload) = outcome {
        std::panic::resume_unwind(payload);
    }
}
