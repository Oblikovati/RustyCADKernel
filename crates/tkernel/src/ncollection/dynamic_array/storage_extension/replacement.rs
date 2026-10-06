//! Explicit OCCT replacement capability branches, without inferring C++ traits from Rust traits.
use super::DynamicArrayStorageExtension;

impl<T> DynamicArrayStorageExtension<'_, T> {
    /// Runs OCCT's assignable branch: construct a temporary, assign, then destroy the temporary.
    /// `unsafe { DynamicArrayStorageExtension::<i32>::replace_at_assign(slot, || 7, |a, b| *a = *b) };`
    /// The assignment hook receives the live target and temporary, including a mutable donor.
    /// A constructor panic leaves the target untouched; an assignment panic preserves its live,
    /// possibly changed value and destroys the temporary. Rust assignment is not a C++ move hook.
    /// # Safety
    /// Slot must be aligned, writable, exclusively accessible, and contain one live T. No active
    /// references may alias the target. The assignment hook must leave both arguments valid even
    /// on unwind. Destructors must not unwind when matching OCCT's noexcept destruction boundary.
    pub unsafe fn replace_at_assign(
        slot: *mut T,
        construct: impl FnOnce() -> T,
        assign: impl FnOnce(&mut T, &mut T),
    ) {
        let mut replacement = construct();
        // SAFETY: the caller supplies an exclusive live target; the local temporary is separate.
        assign(unsafe { &mut *slot }, &mut replacement);
    }

    /// Runs OCCT's nonassignable, copy/move-constructible replacement branch.
    /// `unsafe { DynamicArrayStorageExtension::<i32>::replace_at_reconstruct(slot, || 7, |v| *v) };`
    /// Select the transfer hook explicitly: native move_if_noexcept uses move when it cannot throw
    /// or copying is unavailable, and copy otherwise. No Rust trait implies that C++ capability.
    /// The temporary is constructed before destroying the old target, then transferred and dropped.
    /// # Safety
    /// Slot must be aligned, writable, exclusive and live, with no conflicting references. Its
    /// automatic Drop ownership must be suspended: a transfer panic leaves the slot dead. Do not
    /// read, destroy or republish that slot until reconstruction. A constructor panic leaves it live.
    /// Transfer must leave its donor valid on unwind; destructors must not unwind. Address-dependent
    /// in-place constructors are outside this value-returning transfer contract.
    pub unsafe fn replace_at_reconstruct(
        slot: *mut T,
        construct: impl FnOnce() -> T,
        transfer: impl FnOnce(&mut T) -> T,
    ) {
        let mut replacement = construct();
        // OCCT 3d097a0 destroys the old value only after temporary construction succeeds.
        unsafe { Self::destroy_at(slot) };
        // SAFETY: destruction ended the target lifetime; caller owns repair if transfer panics.
        unsafe { Self::construct_at(slot, || transfer(&mut replacement)) };
    }

    /// Runs OCCT's nonassignable, noncopyable, nonmovable direct-construction branch.
    /// `unsafe { DynamicArrayStorageExtension::<i32>::replace_at_direct(slot, || 7) };`
    /// The old value is destroyed before calling the constructor; no temporary transfer hook runs.
    /// # Safety
    /// Slot must be aligned, writable, exclusive and live, with no conflicting references. Suspend
    /// its automatic Drop ownership: a constructor panic leaves the slot dead. Never read, destroy
    /// or republish it without reconstruction. Destructors must not unwind. Returning a Rust value
    /// does not model an address-dependent C++ in-place constructor or an implicit user move hook.
    pub unsafe fn replace_at_direct(slot: *mut T, construct: impl FnOnce() -> T) {
        // OCCT 3d097a0 selects destruction-before-construction only for this capability branch.
        unsafe { Self::destroy_at(slot) };
        // SAFETY: the target is now uninitialized and remains caller-owned on constructor panic.
        unsafe { Self::construct_at(slot, construct) };
    }
}
