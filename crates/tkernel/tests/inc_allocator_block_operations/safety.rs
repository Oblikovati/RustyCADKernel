//! Rust soundness boundaries: leases, detached typed lifetimes, alignment and concurrent reset.
use rustycad_tkernel::ncollection::{AllocatedBox, BaseAllocator};
use std::{cell::Cell, sync::Barrier};

struct DropObservation<'a> {
    arena: &'a BaseAllocator,
    observed: &'a Cell<bool>,
}
impl Drop for DropObservation<'_> {
    fn drop(&mut self) {
        let mut access = self.arena.incremental_extension().unwrap();
        let state = access.observation();
        self.observed
            .set(!state.blocks.is_empty() && access.clean().is_err());
    }
}

#[test]
fn user_destructor_observes_actual_headers_before_lease_release() {
    let arena = BaseAllocator::incremental(2048);
    let observed = Cell::new(false);
    let owner = arena.allocate_value(DropObservation {
        arena: &arena,
        observed: &observed,
    });
    drop(owner);
    assert!(observed.get());
    arena.incremental_extension().unwrap().clean().unwrap();
}

#[repr(align(256))]
struct Overaligned([u8; 3]);

#[test]
fn actual_header_payload_retains_layout_alignment_after_misaligned_requests() {
    let arena = BaseAllocator::incremental(2048);
    for count in [0, 1, 3, 7, 31, 2047] {
        let bytes = arena.allocate(count).unwrap();
        let mut typed = arena.allocate_value(Overaligned([1, 2, 3]));
        assert_eq!(std::ptr::from_ref(&*typed).addr() % 256, 0);
        typed.0[1] += 1;
        assert_eq!(typed.0, [1, 3, 3]);
        assert!(arena.reset(false).is_err());
        drop((typed, bytes));
        arena.reset(false).unwrap();
    }
    arena.incremental_extension().unwrap().clean().unwrap();
}

fn hold_typed_across_reset(arena: &BaseAllocator, barrier: &Barrier) {
    let owner: AllocatedBox<[u64; 8]> = arena.allocate_value([41; 8]);
    barrier.wait();
    barrier.wait();
    assert_eq!(*owner, [41; 8]);
}

#[test]
fn shared_thread_owner_lease_precedes_protected_reset_and_clean() {
    let arena = BaseAllocator::incremental(2048);
    let barrier = Barrier::new(2);
    std::thread::scope(|scope| {
        scope.spawn(|| hold_typed_across_reset(&arena, &barrier));
        barrier.wait();
        let mut access = arena.incremental_extension().unwrap();
        let head = access.observation().ordered_blocks;
        assert!(access.reset_block(head).is_err());
        assert!(access.clean().is_err());
        drop(access);
        barrier.wait();
    });
    arena.incremental_extension().unwrap().clean().unwrap();
}
