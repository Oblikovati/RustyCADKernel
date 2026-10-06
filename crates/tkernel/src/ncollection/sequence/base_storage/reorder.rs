use super::{BaseSequence, SeqNode};
use crate::standard::Failure;

impl BaseSequence {
    /// Reverses node links while retaining the cached node identity; `unsafe { base.reverse() }`.
    /// # Safety
    /// All retained nodes must be live, exclusively owned, and linked consistently with size.
    pub unsafe fn reverse(&mut self) {
        let mut node = self.first;
        while !node.is_null() {
            let next = unsafe { (*node).next() };
            let previous = unsafe { (*node).previous() };
            unsafe {
                (*node).set_next(previous);
                (*node).set_previous(next);
            }
            node = next;
        }
        std::mem::swap(&mut self.first, &mut self.last);
        if self.size != 0 {
            self.current_index = self.size + 1 - self.current_index;
        }
    }
    /// Exchanges node identities at two checked indices; `unsafe { base.exchange(1, 3)? }`.
    /// # Safety
    /// Nodes, links and cached metadata must be live, consistent, and exclusively managed here.
    pub unsafe fn exchange(&mut self, first: usize, second: usize) -> Result<(), Failure> {
        self.check_index(first)?;
        self.check_index(second)?;
        if first == second {
            return Ok(());
        }
        let (first, second) = (first.min(second), first.max(second));
        let left = unsafe { self.find(first) };
        let right = unsafe { self.find(second) };
        unsafe { self.swap_nodes(left, right) };
        if self.current_index == first {
            self.current = right;
        }
        if self.current_index == second {
            self.current = left;
        }
        Ok(())
    }
    /// Transfers the indexed suffix without clearing or rebinding the destination; `unsafe { base.split(2, &mut tail)? }`.
    /// # Safety
    /// Source nodes/cache must be live and exclusive. Destination must be empty to retain node ownership.
    pub unsafe fn split(&mut self, index: usize, destination: &mut Self) -> Result<(), Failure> {
        self.check_index(index)?;
        let node = unsafe { self.find(index) };
        destination.first = node;
        destination.last = self.last;
        destination.size = self.size - index + 1;
        destination.cache_first();
        self.last = unsafe { (*node).previous() };
        unsafe {
            self.truncate_prefix(index);
            (*node).set_previous(std::ptr::null_mut());
        }
        Ok(())
    }
    /// Represents native self-split's checked DomainError without aliased mutable borrows; `base.split_self(1)`.
    pub fn split_self(&mut self, index: usize) -> Result<(), Failure> {
        self.check_index(index)?;
        Err(Failure::with_type(
            "Standard_DomainError",
            &format!("BaseSequence self split at {index}; expected a distinct destination"),
            "",
        ))
    }
    unsafe fn truncate_prefix(&mut self, index: usize) {
        if self.last.is_null() {
            self.nullify();
            return;
        }
        unsafe { (*self.last).set_next(std::ptr::null_mut()) };
        self.size = index - 1;
        if self.current_index >= index {
            self.cache_first();
        }
    }
    unsafe fn swap_nodes(&mut self, left: *mut SeqNode, right: *mut SeqNode) {
        let before = unsafe { (*left).previous() };
        let after = unsafe { (*right).next() };
        let left_next = unsafe { (*left).next() };
        let right_previous = unsafe { (*right).previous() };
        unsafe { self.swap_outer_links(before, after, left, right) };
        unsafe { Self::swap_inner_links(left, right, left_next, right_previous) };
        unsafe {
            (*right).set_previous(before);
            (*left).set_next(after);
        }
    }
    unsafe fn swap_inner_links(
        left: *mut SeqNode,
        right: *mut SeqNode,
        left_next: *mut SeqNode,
        right_previous: *mut SeqNode,
    ) {
        if left_next == right {
            unsafe {
                (*right).set_next(left);
                (*left).set_previous(right);
            }
        } else {
            unsafe {
                (*left_next).set_previous(right);
                (*right_previous).set_next(left);
                (*right).set_next(left_next);
                (*left).set_previous(right_previous);
            }
        }
    }
    unsafe fn swap_outer_links(
        &mut self,
        before: *mut SeqNode,
        after: *mut SeqNode,
        left: *mut SeqNode,
        right: *mut SeqNode,
    ) {
        if before.is_null() {
            self.first = right;
        } else {
            unsafe { (*before).set_next(right) };
        }
        if after.is_null() {
            self.last = left;
        } else {
            unsafe { (*after).set_previous(left) };
        }
    }
}
