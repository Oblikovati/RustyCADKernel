use super::{BaseSequence, BaseSequenceRawIterator, SeqNode};
use crate::standard::Failure;

impl BaseSequence {
    /// Links one node at the tail; `unsafe { base.append_node(node) }`.
    /// # Safety
    /// Node must be nonnull, exclusively owned and initially detached; existing links must be valid.
    pub unsafe fn append_node(&mut self, node: *mut SeqNode) {
        if self.size == 0 {
            self.first = node;
            self.last = node;
            self.size = 1;
            self.cache_first();
            return;
        }
        unsafe {
            (*self.last).set_next(node);
            (*node).set_previous(self.last);
            (*node).set_next(std::ptr::null_mut());
        }
        self.last = node;
        self.size += 1;
    }
    /// Links one node at the head; `unsafe { base.prepend_node(node) }`.
    /// # Safety
    /// Node must be nonnull, exclusively owned and initially detached; existing links must be valid.
    pub unsafe fn prepend_node(&mut self, node: *mut SeqNode) {
        if self.size == 0 {
            unsafe { self.append_node(node) };
            return;
        }
        unsafe {
            (*self.first).set_previous(node);
            (*node).set_next(self.first);
            (*node).set_previous(std::ptr::null_mut());
        }
        self.first = node;
        self.size += 1;
        self.current_index += 1;
    }
    /// Transfers another base's topology without allocator checks or copying; `unsafe { base.append_sequence(&mut other) }`.
    /// # Safety
    /// Both bases must have disjoint live exclusive nodes and valid links; callers retain allocation provenance.
    pub unsafe fn append_sequence(&mut self, other: &mut Self) {
        if other.size == 0 {
            return;
        }
        if self.size == 0 {
            self.first = other.first;
            self.cache_first();
        } else {
            unsafe {
                (*self.last).set_next(other.first);
                (*other.first).set_previous(self.last);
            }
        }
        self.last = other.last;
        self.size += other.size;
        other.nullify();
    }
    /// Transfers another base before the head, preserving this allocator; `unsafe { base.prepend_sequence(&mut other) }`.
    /// # Safety
    /// Both bases must have disjoint live exclusive nodes and valid links; callers retain allocation provenance.
    pub unsafe fn prepend_sequence(&mut self, other: &mut Self) {
        if other.size == 0 {
            return;
        }
        if self.size == 0 {
            unsafe { self.append_sequence(other) };
            return;
        }
        unsafe {
            (*other.last).set_next(self.first);
            (*self.first).set_previous(other.last);
        }
        self.first = other.first;
        self.size += other.size;
        self.current_index += other.size;
        other.nullify();
    }
    /// Inserts after a raw iterator; null-current prepends; `unsafe { base.insert_after_cursor(&cursor, node) }`.
    /// # Safety
    /// Cursor must name this topology or its null end; node and existing links must be valid and exclusively owned.
    pub unsafe fn insert_after_cursor(
        &mut self,
        cursor: &BaseSequenceRawIterator,
        node: *mut SeqNode,
    ) {
        if cursor.current.is_null() {
            unsafe { self.prepend_node(node) };
            return;
        }
        unsafe { self.link_after(cursor.current, node, node) };
        self.size += 1;
        self.cache_first();
    }
    /// Inserts after a one-based index, with zero meaning prepend; `unsafe { base.insert_after(0, node)? }`.
    /// # Safety
    /// Node must be detached and exclusive; all existing nodes and cache links must be valid.
    pub unsafe fn insert_after(&mut self, index: usize, node: *mut SeqNode) -> Result<(), Failure> {
        self.check_insertion(index)?;
        if index == 0 {
            unsafe { self.prepend_node(node) };
            return Ok(());
        }
        let previous = unsafe { self.find(index) };
        unsafe { self.link_after(previous, node, node) };
        self.size += 1;
        if index < self.current_index {
            self.current_index += 1;
        }
        Ok(())
    }
    /// Transfers another sequence after an index; validates even empty sources; `unsafe { base.insert_sequence_after(0, &mut other)? }`.
    /// # Safety
    /// Both bases must have disjoint live exclusive nodes and valid cache/link metadata.
    pub unsafe fn insert_sequence_after(
        &mut self,
        index: usize,
        other: &mut Self,
    ) -> Result<(), Failure> {
        self.check_insertion(index)?;
        if other.size == 0 {
            return Ok(());
        }
        if index == 0 {
            unsafe { self.prepend_sequence(other) };
            return Ok(());
        }
        unsafe { self.transfer_after(index, other) };
        Ok(())
    }
    unsafe fn transfer_after(&mut self, index: usize, other: &mut Self) {
        let previous = unsafe { self.find(index) };
        unsafe { self.link_after(previous, other.first, other.last) };
        self.size += other.size;
        if index < self.current_index {
            self.current_index += other.size;
        }
        other.nullify();
    }
    unsafe fn link_after(&mut self, before: *mut SeqNode, first: *mut SeqNode, last: *mut SeqNode) {
        let after = unsafe { (*before).next() };
        unsafe {
            (*first).set_previous(before);
            (*last).set_next(after);
            (*before).set_next(first);
        }
        if after.is_null() {
            self.last = last;
        } else {
            unsafe { (*after).set_previous(last) };
        }
    }
    fn check_insertion(&self, index: usize) -> Result<(), Failure> {
        if index > self.size {
            return Err(Failure::out_of_range(&format!(
                "BaseSequence insertion {index}; expected 0..={}",
                self.size
            )));
        }
        Ok(())
    }
}
