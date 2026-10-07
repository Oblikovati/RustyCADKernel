#![allow(unsafe_code)]
use super::fixture::*;
use rustycad_tkernel::ncollection::{BaseAllocator, BoundingNode, BoundingVolume, UBTree};
#[test]
fn implicit_destructor_failure_drains_every_payload_and_block() {
    let (mut tree, allocator) = populated(true);
    tree.gemmate_ref(0, &Payload(99), &Payload(3), &Payload(30));
    start(&allocator, ('D', 'O', 1));
    assert!(caught(|| drop(tree)));
    let events = finish();
    assert_eq!(events.matches("DO").count(), 5);
    assert_eq!(events.matches("DB").count(), 5);
    assert_eq!(allocations(&allocator), 0);
}
#[derive(Clone, Default)]
struct EmptyBounds;
impl BoundingVolume for EmptyBounds {
    fn add(&mut self, _: &Self) {}
    fn is_out(&self, _: &Self) -> bool {
        false
    }
    fn square_extent(&self) -> f64 {
        0.0
    }
}
#[repr(align(128))]
#[derive(Clone, Default)]
struct AlignedObject(u8);
fn policies() -> [BaseAllocator; 5] {
    [
        BaseAllocator::common(),
        BaseAllocator::global_heap(),
        BaseAllocator::incremental(1024),
        BaseAllocator::accumulating(1024).unwrap(),
        BaseAllocator::aligned(32),
    ]
}
fn check_aligned_policy(allocator: BaseAllocator) {
    let mut tree = UBTree::<AlignedObject, EmptyBounds>::with_allocator(allocator.clone());
    tree.add(AlignedObject(1), EmptyBounds);
    tree.gemmate_ref(0, &EmptyBounds, &AlignedObject(2), &EmptyBounds);
    let first = tree.child(tree.root().unwrap(), 0).unwrap();
    let second = tree.child(tree.root().unwrap(), 1).unwrap();
    assert_eq!(std::ptr::from_ref(first).addr() % 128, 0);
    assert_eq!(std::ptr::from_ref(second).addr() % 128, 0);
    assert_eq!(
        std::ptr::from_ref(second).addr() - std::ptr::from_ref(first).addr(),
        std::mem::size_of_val(first)
    );
    assert_eq!(second.object().0, 2);
    tree.kill(0, 0);
    assert_eq!(tree.root().unwrap().object().0, 2);
    drop(tree);
    allocator.reset(true).unwrap();
}
#[test]
fn paired_overaligned_payloads_use_all_supported_allocator_policies() {
    for allocator in policies() {
        check_aligned_policy(allocator);
    }
}
#[test]
fn zero_sized_payloads_and_incremental_leases_follow_live_pair_ownership() {
    let allocator = BaseAllocator::incremental(1024);
    let mut tree = UBTree::<(), EmptyBounds>::with_allocator(allocator.clone());
    tree.add((), EmptyBounds);
    tree.gemmate_ref(0, &EmptyBounds, &(), &EmptyBounds);
    assert!(allocator.reset(true).is_err());
    tree.kill(0, 1);
    assert!(allocator.reset(true).is_err());
    drop(tree);
    allocator.reset(true).unwrap();
}
#[test]
fn rust_node_layout_difference_is_explicit_despite_real_pairing() {
    type Node = BoundingNode<Payload<'O'>, Payload<'B'>>;
    let native: Vec<usize> = native_row("layout ")
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(native, [24, 8, 48]);
    assert_eq!(std::mem::size_of::<Node>(), 48);
    assert_eq!(std::mem::align_of::<Node>(), 8);
    assert_eq!(std::mem::size_of::<[Node; 2]>(), 96);
}
#[test]
fn complete_pair_ownership_can_cross_threads_and_support_shared_queries() {
    let allocator = BaseAllocator::incremental(1024);
    let mut tree = UBTree::<i32, EmptyBounds>::with_allocator(allocator.clone());
    tree.add(1, EmptyBounds);
    tree.gemmate_ref(0, &EmptyBounds, &2, &EmptyBounds);
    std::thread::scope(|scope| {
        scope.spawn(|| assert_eq!(*tree.child(tree.root().unwrap(), 0).unwrap().object(), 1));
        scope.spawn(|| assert_eq!(*tree.child(tree.root().unwrap(), 1).unwrap().object(), 2));
    });
    std::thread::spawn(move || {
        tree.kill(0, 0);
        assert_eq!(*tree.root().unwrap().object(), 2);
    })
    .join()
    .unwrap();
    allocator.reset(true).unwrap();
}
