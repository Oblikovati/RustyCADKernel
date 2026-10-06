use super::oracle::allocator;
use rustycad_tkernel::ncollection::BaseAllocator;
use std::sync::{Arc, Barrier};
fn synchronized_reset_phases(owner: BaseAllocator, gate: Arc<Barrier>) {
    gate.wait();
    assert!(owner.reset(true).is_err());
    gate.wait();
    gate.wait();
    owner.reset(true).unwrap();
}
#[test]
fn returned_owned_allocation_protects_bytes_until_release_across_threads() {
    let owner = allocator(2);
    let gate = Arc::new(Barrier::new(2));
    let cloned_gate = gate.clone();
    let cloned_owner = owner.clone();
    let thread = std::thread::spawn(move || synchronized_reset_phases(cloned_owner, cloned_gate));
    let mut bytes = owner.allocate(32).unwrap();
    bytes.as_mut_slice().fill(19);
    gate.wait();
    gate.wait();
    assert_eq!(bytes.as_slice(), &[19; 32]);
    drop(bytes);
    gate.wait();
    thread.join().unwrap();
}
fn concurrent_byte_allocations(owner: BaseAllocator) {
    for size in [0, 1, 8, 32].into_iter().cycle().take(64) {
        let mut bytes = owner.allocate(size).unwrap();
        assert_eq!(bytes.as_slice(), vec![0; size]);
        bytes.as_mut_slice().fill(13);
        assert_eq!(bytes.as_slice(), vec![13; size]);
    }
}
#[test]
fn owned_allocation_and_lease_are_atomic_against_concurrent_reset() {
    let owner = allocator(2);
    let cloned_owner = owner.clone();
    let thread = std::thread::spawn(move || concurrent_byte_allocations(cloned_owner));
    for _ in 0..64 {
        let _ = owner.reset(true);
        std::thread::yield_now();
    }
    thread.join().unwrap();
    owner.reset(true).unwrap();
}
