//! Qualified parent storage access ends before inherited virtual forwarding can destroy the receiver.
#![allow(unsafe_code)] // Explicit pointer-release and callback-free receiver-capture contracts are tested.
use super::{AllocatorInvocation, AllocatorPolicyInvocation, AllocatorSource};
use crate::standard::{Failure, raise_native_failure};
use std::ptr::NonNull;
pub(super) fn default_policy(source: &dyn AllocatorSource) -> AllocatorPolicyInvocation {
    // Shared physical policy is independent of the containing object's logical lifetime.
    source
        .allocator_object_base()
        .resource()
        .policy_invocation()
}
fn forwards_optimal(source: &dyn AllocatorSource) -> bool {
    source.allocator_object_base().allocate_calls_optimal()
}
fn forwards_allocate(source: &dyn AllocatorSource) -> bool {
    source.allocator_object_base().optimal_calls_allocate()
}
pub(super) fn allocate_default(
    invocation: &AllocatorInvocation,
    size: usize,
) -> Option<NonNull<u8>> {
    // SAFETY: these internal selectors only read immutable policy/physical backing, without callbacks.
    if raise_native_failure(unsafe { invocation.capture(forwards_optimal) }) {
        return invocation.allocate_optimal(size);
    }
    let policy = raise_native_failure(unsafe { invocation.capture(default_policy) });
    policy.allocate(size, true)
}
pub(super) fn allocate_optimal_default(
    invocation: &AllocatorInvocation,
    size: usize,
) -> Option<NonNull<u8>> {
    // SAFETY: forwarding and backing selectors finish before any derived virtual body runs.
    if raise_native_failure(unsafe { invocation.capture(forwards_allocate) }) {
        return invocation.allocate(size);
    }
    let policy = raise_native_failure(unsafe { invocation.capture(default_policy) });
    policy.allocate_optimal(size)
}
pub(super) unsafe fn free_default(
    invocation: &AllocatorInvocation,
    pointer: *mut u8,
) -> Result<(), Failure> {
    // SAFETY: pure field access cannot retire the receiver; pointer validity comes from this body contract.
    let policy = unsafe { invocation.capture(default_policy) }?;
    unsafe { policy.free_nullable(pointer) }
}
