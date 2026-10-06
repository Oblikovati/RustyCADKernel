//! Native backward construction/destruction can leave caller-owned holes on unwind.
#![allow(unsafe_code)]
use super::{ArrayError, LinearVector};
use crate::ncollection::raw_element_lifetime::{construct_in_slot, destroy_in_slot};

impl<T> LinearVector<T> {
    /// Inserts with explicit native growth, shift and value constructors; e.g.
    /// `unsafe { vector.insert_before_with_constructors(0, |v| *v, |v| *v, || 7)? };`.
    /// Growth selects native move_if_noexcept through `relocate`; shifts always use `shift`.
    /// # Safety
    /// Enter with all registered elements live and no unpublished live tail. Match a
    /// nontrivial native element policy. Destructors must not unwind. Each transfer
    /// must leave its source valid even on unwind; captured donors cannot alias receiver slots.
    /// Unless destructive callbacks are proven non-panicking, enclose the receiver in
    /// ManuallyDrop before this call, including during propagation to a catch boundary.
    /// If shifting source `s` fails, [0,s] stays live (possibly modified), slot s+1 is dead,
    /// and [s+2,old Size] is live. If the final constructor fails, [0,index) and
    /// [index+1,old Size] are live and index is dead. Empty ranges contain no live values.
    /// Thus a failure before any successful shift has no live unpublished tail; inserting at
    /// the end never creates an interior hole on constructor failure. Size stays old.
    /// Track completed shifts; before slices, normal Drop, or releasing ManuallyDrop,
    /// reconstruct each dead slot below Size and destroy a live extra tail exactly once.
    /// Querying len/capacity and acquiring raw storage pointers does not read dead elements.
    /// Never read/drop a dead slot or leak the extra tail.
    /// Returning a value does not model address-dependent in-place constructors. Failed growth
    /// retains the existing reserve_with_relocation cleanup policy before any destructive shift.
    pub unsafe fn insert_before_with_constructors(
        &mut self,
        index: usize,
        relocate: impl FnMut(&mut T) -> T,
        shift: impl FnMut(&mut T) -> T,
        construct: impl FnOnce() -> T,
    ) -> Result<(), ArrayError> {
        if index > self.len() {
            return Err(self.index_error(index));
        }
        self.reserve_with_relocation(self.len() + 1, relocate);
        // SAFETY: the caller owns hole/tail recovery; successful construction fills the gap.
        unsafe { self.insert_into_reserved(index, shift, construct) };
        Ok(())
    }

    /// Inserts after an existing item using the same explicit native constructor policy;
    /// `unsafe { vector.insert_after_with_constructors(0, |v| *v, |v| *v, || 7)? };`.
    /// # Safety
    /// All lifetime, no-unwinding destruction, donor exclusivity and ManuallyDrop recovery
    /// obligations of `insert_before_with_constructors` apply at insertion index `index + 1`.
    pub unsafe fn insert_after_with_constructors(
        &mut self,
        index: usize,
        relocate: impl FnMut(&mut T) -> T,
        shift: impl FnMut(&mut T) -> T,
        construct: impl FnOnce() -> T,
    ) -> Result<(), ArrayError> {
        self.value(index)?;
        // SAFETY: the caller assumes the identical policy obligations for the shifted index.
        unsafe { self.insert_before_with_constructors(index + 1, relocate, shift, construct) }
    }

    unsafe fn insert_into_reserved(
        &mut self,
        index: usize,
        mut shift: impl FnMut(&mut T) -> T,
        construct: impl FnOnce() -> T,
    ) {
        let old_len = self.len();
        for from in (index..old_len).rev() {
            // SAFETY: backward traversal leaves the following slot uninitialized.
            unsafe { self.shift_one_right(from, &mut shift) };
        }
        // SAFETY: completed shifts leave exactly the insertion gap; publish only after success.
        unsafe {
            construct_in_slot(self.elements.as_mut_ptr().add(index), construct);
            self.elements.set_len(old_len + 1);
        }
    }

    unsafe fn shift_one_right(&mut self, from: usize, shift: &mut impl FnMut(&mut T) -> T) {
        // SAFETY: this source is still live, its successor is exclusive uninitialized storage,
        // and the caller suppresses automatic Drop if construction leaves a later gap.
        unsafe {
            let source = self.elements.as_mut_ptr().add(from);
            construct_in_slot(source.add(1), || shift(&mut *source));
            destroy_in_slot(source);
        }
    }
}
