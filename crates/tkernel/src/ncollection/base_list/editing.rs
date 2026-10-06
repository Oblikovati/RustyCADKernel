//! Individual-node edits preserve native cursor and callback ordering.
use super::{
    BaseAllocator, BaseListRawIterator, BaseListStorage, Failure, ListLinkNode, require_position,
};
use std::ptr::null_mut;
impl BaseListStorage {
    /// Appends an existing node and resets its next link; `unsafe { storage.append_node(node) }`.
    ///
    /// # Safety
    /// Node must be live, writable and disjoint from this valid chain; all touched links require exclusive access.
    pub unsafe fn append_node(&mut self, node: *mut ListLinkNode) {
        // SAFETY: caller provides writable node and existing last node when length is nonzero.
        unsafe {
            if self.length != 0 {
                *(*self.last).next_mut() = node;
            } else {
                self.first = node;
            }
            *(*node).next_mut() = null_mut();
        }
        self.last = node;
        self.length = self.length.wrapping_add(1);
    }
    /// Appends and updates the output cursor's two links; `unsafe { storage.append_at(node, &mut cursor) }`.
    ///
    /// # Safety
    /// The node and chain must meet `append_node`'s validity and exclusivity requirements.
    pub unsafe fn append_at(&mut self, node: *mut ListLinkNode, cursor: &mut BaseListRawIterator) {
        let previous = self.last;
        // SAFETY: caller forwards append's node contracts.
        unsafe {
            self.append_node(node);
            cursor.init_after(self, previous);
        }
    }
    /// Prepends an existing node; `unsafe { storage.prepend_node(node) }`.
    ///
    /// # Safety
    /// Node must be live, writable and disjoint from this valid chain, with exclusive link access.
    pub unsafe fn prepend_node(&mut self, node: *mut ListLinkNode) {
        // SAFETY: caller provides an exclusive live node.
        unsafe {
            *(*node).next_mut() = self.first;
        }
        self.first = node;
        if self.last.is_null() {
            self.last = node;
        }
        self.length = self.length.wrapping_add(1);
    }
    /// Calls the native deleter in chain order; `unsafe { storage.clear(delete_node) }`.
    ///
    /// # Safety
    /// The chain must be readable and acyclic. The callback may free the current node but must preserve unvisited links.
    pub unsafe fn clear(
        &mut self,
        mut delete_node: impl FnMut(*mut ListLinkNode, &mut BaseAllocator),
    ) {
        let mut current = self.first;
        while !current.is_null() {
            // SAFETY: caller preserves the next node through each callback.
            let next = unsafe { (*current).next() };
            delete_node(current, &mut self.allocator);
            current = next;
        }
        self.reset_links();
    }
    /// Removes the head, raising native NoSuchObject when empty; `unsafe { storage.remove_first(delete_node)? }`.
    ///
    /// # Safety
    /// Nonempty storage must own a valid readable head; the callback may free it and must preserve remaining nodes.
    pub unsafe fn remove_first(
        &mut self,
        mut delete_node: impl FnMut(*mut ListLinkNode, &mut BaseAllocator),
    ) -> Result<(), Failure> {
        require_position(&BaseListRawIterator::new(self), "pRemoveFirst")?;
        let removed = self.first;
        // SAFETY: the head is live and readable by the caller's contract.
        self.first = unsafe { (*removed).next() };
        delete_node(removed, &mut self.allocator);
        self.length = self.length.wrapping_sub(1);
        if self.length == 0 {
            self.last = null_mut();
        }
        Ok(())
    }
    /// Removes the positioned node and retains its predecessor; `unsafe { storage.remove(&mut cursor, delete_node)? }`.
    ///
    /// # Safety
    /// A non-end cursor must be valid in this exclusive chain with its actual predecessor; callback may free only current.
    pub unsafe fn remove(
        &mut self,
        cursor: &mut BaseListRawIterator,
        delete_node: impl FnMut(*mut ListLinkNode, &mut BaseAllocator),
    ) -> Result<(), Failure> {
        require_position(cursor, "pRemove")?;
        if cursor.previous.is_null() {
            // SAFETY: caller guarantees current is the head and callback is valid.
            unsafe {
                self.remove_first(delete_node)?;
            }
            cursor.current = self.first;
            return Ok(());
        }
        // SAFETY: non-head current and predecessor are live and exclusive.
        unsafe { self.remove_linked(cursor, delete_node) };
        Ok(())
    }
    /// Reverses links without moving values; `unsafe { storage.reverse() }`.
    ///
    /// # Safety
    /// The chain must be live, exclusively writable and acyclic, matching the stored endpoints and count.
    pub unsafe fn reverse(&mut self) {
        if self.length <= 1 {
            return;
        }
        let mut previous = null_mut();
        let mut current = self.first;
        self.last = self.first;
        while !current.is_null() {
            // SAFETY: each node is visited once in the caller-provided exclusive chain.
            let next = unsafe { (*current).next() };
            unsafe {
                *(*current).next_mut() = previous;
            }
            previous = current;
            current = next;
        }
        self.first = previous;
    }
    unsafe fn remove_linked(
        &mut self,
        cursor: &mut BaseListRawIterator,
        mut delete_node: impl FnMut(*mut ListLinkNode, &mut BaseAllocator),
    ) {
        // SAFETY: caller guarantees both current and previous are exclusive live nodes.
        let next = unsafe { (*cursor.current).next() };
        unsafe {
            *(*cursor.previous).next_mut() = next;
        }
        delete_node(cursor.current, &mut self.allocator);
        cursor.current = next;
        if next.is_null() {
            self.last = cursor.previous;
        }
        self.length = self.length.wrapping_sub(1);
    }
}
