//! Public emplacement selects native replacement capabilities without Rust trait inference.
use super::{ArrayError, DynamicArray, DynamicArrayStorageExtension, checked_index};
use std::fmt::Debug;

impl<T: Default> DynamicArray<T> {
    /// Uses native temporary construction and move assignment for an existing slot.
    /// `array.emplace_value_with_assignment(0, || 7, |target, donor| *target = *donor)?`.
    /// New indices default-construct gaps and construct directly without assignment.
    pub fn emplace_value_with_assignment<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        construct: impl FnOnce() -> T,
        assign: impl FnOnce(&mut T, &mut T),
    ) -> Result<&mut T, ArrayError> {
        // Safe hooks cannot invalidate their arguments; the target stays live after unwind.
        unsafe {
            self.emplace_value_with_policy(index, construct, |slot, construct| {
                DynamicArrayStorageExtension::replace_at_assign(slot, construct, assign);
            })
        }
    }

    /// Uses OCCT's nonassignable temporary-then-reconstruct branch for existing slots.
    /// `unsafe { array.emplace_value_with_reconstruction(0, || 7, |temporary| *temporary)? };`
    /// Select move_if_noexcept's move/copy transfer explicitly; Rust traits do not select it.
    /// New indices use ordinary gap construction and never call the transfer hook.
    /// # Safety
    /// Destructors must not unwind. A transfer panic leaves the target dead while Size and
    /// registered ownership stay unchanged, exactly as native EmplaceValue. Enclose the array
    /// in ManuallyDrop before this call, or guarantee the destructive callbacks cannot unwind.
    /// After a destructive failure, repair the slot through its retained raw pointer before
    /// element access, release of ManuallyDrop, or automatic cleanup can occur. Size remains
    /// queryable without reading elements. Never read or
    /// destroy a dead slot. Even unwinding before a catch must not automatically drop the array.
    /// Transfer must leave its donor valid on unwind. A failed initial constructor leaves the
    /// old target live; callers must distinguish this from a failed transfer before repairing.
    pub unsafe fn emplace_value_with_reconstruction<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        construct: impl FnOnce() -> T,
        transfer: impl FnOnce(&mut T) -> T,
    ) -> Result<&mut T, ArrayError> {
        // Caller assumes the dead-slot repair obligation of this explicitly selected policy.
        unsafe {
            self.emplace_value_with_policy(index, construct, |slot, construct| {
                DynamicArrayStorageExtension::replace_at_reconstruct(slot, construct, transfer);
            })
        }
    }

    /// Destroys an existing item before directly constructing its replacement, without transfer.
    /// `unsafe { array.emplace_value_with_direct_replacement(0, || 7)? };`
    /// This selects native's nonassignable, nonmovable, noncopyable replacement branch.
    /// # Safety
    /// Destructors must not unwind. A constructor panic for an existing slot leaves that slot
    /// dead with Size unchanged. Enclose the array in ManuallyDrop before this call, or prove
    /// destructive callbacks cannot unwind. Repair through the retained raw slot pointer before
    /// element access or automatic Drop, including cleanup during unwinding before a catch. Size
    /// remains queryable without accessing elements. Never
    /// read or destroy the dead slot. New-index construction retains the existing staging policy.
    pub unsafe fn emplace_value_with_direct_replacement<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, ArrayError> {
        // The caller prevents dead-slot access and cleanup until reconstruction succeeds.
        unsafe {
            self.emplace_value_with_policy(index, construct, |slot, construct| {
                DynamicArrayStorageExtension::replace_at_direct(slot, construct);
            })
        }
    }

    unsafe fn emplace_value_with_policy<I: TryInto<usize> + Copy + Debug, C: FnOnce() -> T>(
        &mut self,
        index: I,
        construct: C,
        replace: impl FnOnce(*mut T, C),
    ) -> Result<&mut T, ArrayError> {
        let index = checked_index(index)?;
        self.ensure_storage_for_index(index);
        if index >= self.used {
            return self.emplace_checked(index, construct);
        }
        let slot = self.change_value(index)? as *mut T;
        replace(slot, construct);
        self.change_value(index)
    }
}
