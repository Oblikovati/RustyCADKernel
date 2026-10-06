//! Raw construction/destruction shared by native container lifetime policies.
#![allow(unsafe_code)]

/// Starts one lifetime after successful construction; e.g. `construct_in_slot(slot, || 7)`.
/// # Safety
/// The slot must be aligned, exclusive, writable and uninitialized. Caller owns cleanup.
pub(super) unsafe fn construct_in_slot<T>(slot: *mut T, construct: impl FnOnce() -> T) {
    let value = construct();
    // SAFETY: the caller owns an uninitialized slot; failed construction never writes it.
    unsafe { slot.write(value) };
}

/// Ends one lifetime without freeing storage; e.g. `destroy_in_slot(slot)`.
/// # Safety
/// The slot must be exclusively owned and live, with automatic destruction suspended.
pub(super) unsafe fn destroy_in_slot<T>(slot: *mut T) {
    // SAFETY: the caller transfers the one live value's destruction obligation.
    unsafe { slot.drop_in_place() };
}
