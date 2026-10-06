//! Native nonassignable SetValue first constructs from its internal source, then rebuilds the target.
use super::{ArrayError, DynamicArray, DynamicArrayStorageExtension};
use std::fmt::Debug;
impl<T: Default> DynamicArray<T> {
    /// Copies from an internal source before replacing a nonassignable existing item.
    /// `unsafe { array.set_value_from_index_with_reconstruction(0, 1, |s| *s, |s| *s)? };`
    /// Select native move_if_noexcept's move/copy transfer explicitly. New targets construct gaps
    /// and the requested item directly; their transfer callback is never invoked.
    /// # Safety
    /// Destructors must not unwind. A transfer panic leaves the existing target dead with Size and
    /// automatic ownership unchanged. Enclose the array in ManuallyDrop before the call, or prove
    /// the destructive transfer cannot unwind. Repair the slot through its retained raw pointer
    /// before element access or any automatic cleanup, including unwinding before a catch. Never
    /// read/drop a dead slot. An initial source-constructor panic leaves the old target live; callers
    /// must distinguish these phases before repair. Callbacks leave their source/temporary live.
    pub unsafe fn set_value_from_index_with_reconstruction<
        I: TryInto<usize> + Copy + Debug,
        S: TryInto<usize> + Copy + Debug,
    >(
        &mut self,
        index: I,
        source_index: S,
        copy_construct: impl FnOnce(&T) -> T,
        transfer: impl FnOnce(&mut T) -> T,
    ) -> Result<&mut T, ArrayError> {
        // Caller accepts reconstruction's dead-slot repair and unwind obligations.
        unsafe {
            self.reconstruct_internal_source(index, source_index, |s| copy_construct(s), transfer)
        }
    }
    /// Move-constructs from an internal donor before rebuilding a nonassignable existing target.
    /// `unsafe { array.set_value_moved_from_index_with_reconstruction(0, 1, std::mem::take, std::mem::take)? };`
    /// Self-source construction finishes before target destruction; new targets skip transfer.
    /// # Safety
    /// Destructors must not unwind and callbacks leave donors live. Enclose the array in ManuallyDrop
    /// before any potentially panicking transfer: failure after target destruction leaves a dead
    /// slot still counted and owned. Repair via a retained raw pointer before element access or
    /// automatic cleanup, including unwind cleanup before a catch. Initial constructor failure
    /// retains the old live target (possibly modified as donor); distinguish it from transfer failure.
    pub unsafe fn set_value_moved_from_index_with_reconstruction<
        I: TryInto<usize> + Copy + Debug,
        S: TryInto<usize> + Copy + Debug,
    >(
        &mut self,
        index: I,
        source_index: S,
        move_construct: impl FnOnce(&mut T) -> T,
        transfer: impl FnOnce(&mut T) -> T,
    ) -> Result<&mut T, ArrayError> {
        // The caller prevents dead-slot access and automatic Drop until repair succeeds.
        unsafe { self.reconstruct_internal_source(index, source_index, move_construct, transfer) }
    }
    unsafe fn reconstruct_internal_source<
        I: TryInto<usize> + Copy + Debug,
        S: TryInto<usize> + Copy + Debug,
    >(
        &mut self,
        index: I,
        source_index: S,
        construct: impl FnOnce(&mut T) -> T,
        transfer: impl FnOnce(&mut T) -> T,
    ) -> Result<&mut T, ArrayError> {
        self.set_internal_constructed(index, source_index, construct, |array, index, temporary| {
            let slot = array.change_value(index)? as *mut T;
            // Construction already finished without retaining a donor borrow, even for self-source.
            unsafe {
                DynamicArrayStorageExtension::replace_at_reconstruct(slot, || temporary, transfer)
            };
            Ok(())
        })
    }
}
