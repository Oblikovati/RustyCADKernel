//! Copied immutable virtual code represents methods, while actual storage remains weakly selected.
#![allow(unsafe_code)] // Explicit pointer-release and callback-free receiver-capture contracts are tested.
use super::AllocatorInvocation;
use crate::standard::Failure;
use std::ptr::NonNull;
/// Virtual allocation body; `fn allocate(inv:&AllocatorInvocation,size:usize)->Option<NonNull<u8>>`.
pub type AllocatorAllocateCode = fn(&AllocatorInvocation, usize) -> Option<NonNull<u8>>;
/// Virtual nullable release body; `unsafe fn free(inv:&AllocatorInvocation,p:*mut u8)->Result<(),Failure>`.
pub type AllocatorFreeCode = unsafe fn(&AllocatorInvocation, *mut u8) -> Result<(), Failure>;
/// Native method pointers, not captured mutable object values; `AllocatorDispatchCode::default()`.
#[derive(Clone, Copy)]
pub struct AllocatorDispatchCode {
    /// Actual Allocate body; `code.allocate = derived_allocate`.
    pub allocate: AllocatorAllocateCode,
    /// Actual AllocateOptimal body; `code.allocate_optimal = derived_optimal`.
    pub allocate_optimal: AllocatorAllocateCode,
    /// Actual Free body; `code.free = derived_free`.
    pub free: AllocatorFreeCode,
}
impl Default for AllocatorDispatchCode {
    fn default() -> Self {
        Self {
            allocate: super::allocator_default_dispatch::allocate_default,
            allocate_optimal: super::allocator_default_dispatch::allocate_optimal_default,
            free: super::allocator_default_dispatch::free_default,
        }
    }
}
