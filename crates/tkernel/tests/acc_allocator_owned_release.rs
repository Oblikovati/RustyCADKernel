//! A Rust-owned typed pointer must release its count beyond native findBlock's two-key window.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::BaseAllocator;

#[repr(align(64))]
struct OwnedAligned64([u64; 8]);

fn oversized_small_key_arena() -> BaseAllocator {
    let arena = BaseAllocator::accumulating(16).unwrap();
    arena
        .accumulating_extension()
        .unwrap()
        .allocate_new_block(512)
        .unwrap();
    arena
}
#[test]
fn typed_owner_releases_interior_pointer_outside_native_lookup_window() {
    let arena = oversized_small_key_arena();
    let owner = arena.allocate_value(OwnedAligned64([42; 8]));
    assert_eq!(owner.0, [42; 8]);
    assert!(
        arena
            .accumulating_block((&*owner as *const OwnedAligned64).addr())
            .is_none()
    );
    assert_eq!(arena.accumulating_chain()[0].alloc_count, 1);
    drop(owner);
    let chain = arena.accumulating_chain();
    assert_eq!(
        (chain.len(), chain[0].alloc_count, chain[0].free_size),
        (1, 0, 16)
    );
}
#[test]
fn raw_free_keeps_native_two_key_lookup_failure_and_count() {
    let arena = oversized_small_key_arena();
    let raw = arena.allocate_raw(64).unwrap();
    assert!(arena.accumulating_block(raw.as_ptr().addr()).is_none());
    // SAFETY: this live pointer belongs to arena; native lookup reports the defined lookup failure.
    let failure = unsafe { arena.free_raw(raw) }.unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert_eq!(arena.accumulating_chain()[0].alloc_count, 1);
}
#[test]
fn owned_bytes_at_the_fresh_block_endpoint_release_their_count() {
    let arena = BaseAllocator::accumulating(64).unwrap();
    let empty = arena.allocate(0).unwrap();
    assert_eq!(arena.accumulating_chain()[0].alloc_count, 1);
    drop(empty);
    assert_eq!(arena.accumulating_chain()[0].alloc_count, 0);
}
