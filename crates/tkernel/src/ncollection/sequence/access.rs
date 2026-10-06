use super::{SeqNode, Sequence, SequenceBaseAccess, owner};
use crate::standard::Failure;

impl<T> Sequence<T> {
    /// Locks the canonical native base for inspection and raw cursor construction.
    /// Example: `let base = sequence.base_access(); assert_eq!(base.size(), 2)`.
    /// Drop this guard before reentering any sequence operation that reads its cache.
    pub fn base_access(&self) -> SequenceBaseAccess<'_> {
        self.topology.access()
    }
    pub(super) fn checked_node(
        &self,
        member: &str,
        index: usize,
        cache: bool,
    ) -> Result<*mut SeqNode, Failure> {
        self.offset(member, index)?;
        Ok(self.topology.find(index, cache))
    }
    pub(super) fn read_indexed(&self, index: usize) -> Result<&T, Failure> {
        let node = self.checked_node("Value", index, true)?;
        // SAFETY: this shared sequence borrow keeps its owned node live and immutable.
        Ok(unsafe { owner::item_ref(node) })
    }
    pub(super) fn write_indexed(&mut self, index: usize) -> Result<&mut T, Failure> {
        let node = self.checked_node("ChangeValue", index, true)?;
        // SAFETY: exclusive sequence borrowing excludes all other item references.
        Ok(unsafe { owner::item_mut(node) })
    }
    fn endpoint(&self, first: bool) -> *mut SeqNode {
        let (head, tail) = self.topology.endpoints();
        if first { head } else { tail }
    }
    pub(super) fn endpoint_value(&self, first: bool) -> Option<&T> {
        let node = self.endpoint(first);
        if node.is_null() {
            return None;
        }
        Some(unsafe { owner::item_ref(node) })
    }
    pub(super) fn endpoint_mut(&mut self, first: bool) -> Option<&mut T> {
        let node = self.endpoint(first);
        if node.is_null() {
            return None;
        }
        Some(unsafe { owner::item_mut(node) })
    }
}
