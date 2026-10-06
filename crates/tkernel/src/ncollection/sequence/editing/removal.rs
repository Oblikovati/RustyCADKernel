//! Native metadata and cursor updates finish before each removal's destructor boundary.
use super::super::{
    BaseAllocator, BaseSequence, BaseSequenceRawIterator, SeqNode, Sequence, SequenceCursor,
    iteration::pointer_address, owner::take_owner, topology::SequenceTopology,
};
use super::{Failure, no_such_node, out_of_range};
struct ClearMetadata<'a>(&'a mut SequenceTopology);
impl Drop for ClearMetadata<'_> {
    fn drop(&mut self) {
        // Reset before remaining owners unwind: observers must never traverse freed metadata.
        self.0
            .edit(|base| *base = BaseSequence::new(Some(base.allocator().clone())));
    }
}
impl<T> Sequence<T> {
    /// Removes an existing item through native node/cache deletion; `sequence.remove(1)?`.
    pub fn remove(&mut self, index: usize) -> Result<(), Failure> {
        self.offset("Remove", index)?;
        let node = self
            .topology
            .edit(|base| unsafe { base.prepare_index_removal(index) })?;
        drop(take_owner(&mut self.owners, node));
        Ok(())
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
        let node = self
            .topology
            .edit(|base| unsafe { base.prepare_range_removal(first, last) })?;
        self.destroy_detached_span(node, last - first + 1);
        Ok(())
    }
    fn destroy_detached_span(&mut self, mut node: *mut SeqNode, count: usize) {
        // Retain detached owners in native order so unwinding also releases the entire span.
        let mut detached = Vec::with_capacity(count);
        for _ in 0..count {
            let next = unsafe { (*node).next() };
            detached.push(take_owner(&mut self.owners, node));
            node = next;
        }
        for owner in detached {
            drop(owner);
        }
    }
    /// Removes current and advances it with previous retained; `sequence.remove_cursor(&mut cursor)?`.
    pub fn remove_cursor(&mut self, cursor: &mut SequenceCursor) -> Result<(), Failure> {
        if !cursor.more() {
            return Ok(());
        }
        let node = self
            .owned_node(cursor.address)
            .ok_or_else(|| no_such_node("Remove", *cursor))?;
        let mut raw = BaseSequenceRawIterator {
            current: node,
            previous: std::ptr::null_mut(),
        };
        self.prepare_owned_cursor(&mut raw);
        cursor.address = pointer_address(raw.current);
        drop(take_owner(&mut self.owners, node));
        Ok(())
    }
    pub(crate) fn prepare_owned_cursor(&mut self, raw: &mut BaseSequenceRawIterator) {
        self.topology
            .edit(|base| unsafe { base.prepare_cursor_removal(raw) });
    }
    /// Frees items in native order and optionally replaces the allocator; `sequence.clear(Some(arena))`.
    pub fn clear(&mut self, allocator: Option<BaseAllocator>) {
        let mut owners = std::mem::take(&mut self.owners);
        let mut node = self.topology.endpoints().0;
        let reset = ClearMetadata(&mut self.topology);
        while !node.is_null() {
            let next = unsafe { (*node).next() };
            drop(take_owner(&mut owners, node));
            node = next;
        }
        drop(reset);
        if let Some(allocator) = allocator {
            self.topology.edit(|base| base.allocator = allocator);
        }
    }
}
impl<T> Drop for Sequence<T> {
    fn drop(&mut self) {
        self.clear(None);
    }
}
