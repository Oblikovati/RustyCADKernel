use super::super::{
    BaseAllocator, BaseSequence, BaseSequenceRawIterator, Sequence, SequenceCursor,
    iteration::pointer_address, owner::take_owner,
};
use super::{Failure, no_such_node, out_of_range};

struct ClearMetadata<'a>(&'a mut BaseSequence);
impl Drop for ClearMetadata<'_> {
    fn drop(&mut self) {
        // A throwing Rust destructor must not leave safe lookup pointing at freed nodes.
        *self.0 = BaseSequence::new(Some(self.0.allocator().clone()));
    }
}
impl<T> Sequence<T> {
    /// Removes an existing item through native node/cache deletion; `sequence.remove(1)?`.
    pub fn remove(&mut self, index: usize) -> Result<(), Failure> {
        self.offset("Remove", index)?;
        let owners = &mut self.owners;
        unsafe {
            self.topology
                .exclusive()
                .remove(index, |node, _| drop(take_owner(owners, node)))
        }
    }
    /// Removes an inclusive one-based range in forward destruction order; `sequence.remove_range(2, 4)?`.
    pub fn remove_range(&mut self, first: usize, last: usize) -> Result<(), Failure> {
        if first == 0 || last > self.len() || first > last {
            return Err(out_of_range(
                "Remove",
                first,
                &format!("1 <= first <= last <= {} (last {last})", self.len()),
            ));
        }
        let owners = &mut self.owners;
        unsafe {
            self.topology
                .exclusive()
                .remove_range(first, last, |node, _| drop(take_owner(owners, node)))
        }
    }
    /// Removes current and advances the cursor with its previous link retained; `sequence.remove_cursor(&mut cursor)?`.
    pub fn remove_cursor(&mut self, cursor: &mut SequenceCursor) -> Result<(), Failure> {
        if !cursor.more() {
            return Ok(());
        }
        let (_, node) = self
            .cursor_node(cursor.address)
            .ok_or_else(|| no_such_node("Remove", *cursor))?;
        // Update cursor before invoking a destructor which may unwind.
        cursor.address = pointer_address(unsafe { (*node).next() });
        self.remove_owned_cursor(node);
        Ok(())
    }
    fn remove_owned_cursor(&mut self, node: *mut super::super::SeqNode) {
        let mut raw = BaseSequenceRawIterator {
            current: node,
            previous: std::ptr::null_mut(),
        };
        let owners = &mut self.owners;
        unsafe {
            self.topology
                .exclusive()
                .remove_cursor(&mut raw, |node, _| drop(take_owner(owners, node)))
        };
    }
    /// Frees items in native order and optionally replaces the allocator; `sequence.clear(Some(arena))`.
    pub fn clear(&mut self, allocator: Option<BaseAllocator>) {
        let mut owners = std::mem::take(&mut self.owners);
        let reset = ClearMetadata(self.topology.exclusive());
        unsafe {
            reset
                .0
                .clear_seq(|node, _| drop(take_owner(&mut owners, node)))
        };
        drop(reset);
        if let Some(allocator) = allocator {
            self.topology.exclusive().allocator = allocator;
        }
    }
}
impl<T> Drop for Sequence<T> {
    fn drop(&mut self) {
        self.clear(None);
    }
}
