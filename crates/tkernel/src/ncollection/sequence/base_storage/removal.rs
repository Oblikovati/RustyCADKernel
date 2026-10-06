use super::{BaseAllocator, BaseSequence, BaseSequenceRawIterator, SeqNode};
use crate::standard::Failure;

impl BaseSequence {
    /// Deletes in forward order, passing the same mutable allocator handle each time; `unsafe { base.clear_seq(delete) }`.
    /// # Safety
    /// Nodes must be live and exclusively owned; callback must delete only its node and preserve other links.
    /// If the callback panics, native partially deleted metadata must not be traversed again.
    pub unsafe fn clear_seq(&mut self, mut delete: impl FnMut(*mut SeqNode, &mut BaseAllocator)) {
        let mut node = self.first;
        while !node.is_null() {
            let next = unsafe { (*node).next() };
            delete(node, &mut self.allocator);
            node = next;
        }
        self.nullify();
    }
    /// Removes current and advances it while retaining its previous link; `unsafe { base.remove_cursor(&mut cursor, delete) }`.
    /// # Safety
    /// Current must be null or a live owned node of this topology. The callback must delete only that node.
    pub unsafe fn remove_cursor(
        &mut self,
        cursor: &mut BaseSequenceRawIterator,
        mut delete: impl FnMut(*mut SeqNode, &mut BaseAllocator),
    ) {
        if let Some(node) = unsafe { self.prepare_cursor_removal(cursor) } {
            delete(node, &mut self.allocator);
        }
    }
    /// Removes a checked one-based node; `unsafe { base.remove(1, delete)? }`.
    /// # Safety
    /// Retained nodes/cache must be live and exclusive; callback must delete only its supplied node.
    pub unsafe fn remove(
        &mut self,
        index: usize,
        mut delete: impl FnMut(*mut SeqNode, &mut BaseAllocator),
    ) -> Result<(), Failure> {
        let node = unsafe { self.prepare_index_removal(index) }?;
        delete(node, &mut self.allocator);
        Ok(())
    }
    /// Removes an inclusive range in forward callback order; `unsafe { base.remove_range(1, 2, delete)? }`.
    /// # Safety
    /// Retained nodes/cache must be live and exclusive; callback must delete only its supplied node.
    pub unsafe fn remove_range(
        &mut self,
        from: usize,
        to: usize,
        mut delete: impl FnMut(*mut SeqNode, &mut BaseAllocator),
    ) -> Result<(), Failure> {
        let mut node = unsafe { self.prepare_range_removal(from, to) }?;
        for _ in from..=to {
            let next = unsafe { (*node).next() };
            delete(node, &mut self.allocator);
            node = next;
        }
        Ok(())
    }
    pub(crate) unsafe fn prepare_cursor_removal(
        &mut self,
        cursor: &mut BaseSequenceRawIterator,
    ) -> Option<*mut SeqNode> {
        let node = cursor.current;
        if node.is_null() {
            return None;
        }
        cursor.current = unsafe { self.detach_span(node, node, 1) };
        self.current = self.last;
        self.current_index = self.size;
        Some(node)
    }
    pub(crate) unsafe fn prepare_index_removal(
        &mut self,
        index: usize,
    ) -> Result<*mut SeqNode, Failure> {
        self.check_index(index)?;
        let node = unsafe { self.find(index) };
        let after = unsafe { self.detach_span(node, node, 1) };
        self.cache_after_removal(index, index, after);
        Ok(node)
    }
    pub(crate) unsafe fn prepare_range_removal(
        &mut self,
        from: usize,
        to: usize,
    ) -> Result<*mut SeqNode, Failure> {
        self.check_range(from, to)?;
        let node = unsafe { self.find(from) };
        let last = unsafe { self.find(to) };
        let after = unsafe { self.detach_span(node, last, to - from + 1) };
        self.cache_after_removal(from, to, after);
        Ok(node)
    }
    unsafe fn detach_span(
        &mut self,
        first: *mut SeqNode,
        last: *mut SeqNode,
        count: usize,
    ) -> *mut SeqNode {
        let before = unsafe { (*first).previous() };
        let after = unsafe { (*last).next() };
        unsafe { self.connect_neighbors(before, after) };
        self.size -= count;
        after
    }
    unsafe fn connect_neighbors(&mut self, before: *mut SeqNode, after: *mut SeqNode) {
        if before.is_null() {
            self.first = after;
        } else {
            unsafe { (*before).set_next(after) };
        }
        if after.is_null() {
            self.last = before;
        } else {
            unsafe { (*after).set_previous(before) };
        }
    }
    fn cache_after_removal(&mut self, from: usize, to: usize, after: *mut SeqNode) {
        if self.current_index > to {
            self.current_index -= to - from + 1;
            return;
        }
        if self.current_index < from {
            return;
        }
        if after.is_null() {
            self.current = self.last;
            self.current_index = self.size;
        } else {
            self.current = after;
            self.current_index = from;
        }
    }
    fn check_range(&self, from: usize, to: usize) -> Result<(), Failure> {
        if from == 0 || to > self.size || from > to {
            return Err(Failure::out_of_range(&format!(
                "BaseSequence range {from}..={to}; expected 1 <= from <= to <= {}",
                self.size
            )));
        }
        Ok(())
    }
}
