use super::fixture::{KeyTrace, TracedKey, caught, native_row};
use rustycad_tkernel::ncollection::{BaseAllocator, MapLinkNode, MapNode};
#[test]
fn rejected_copy_releases_preallocated_native_node() {
    let trace = KeyTrace::new();
    let key = trace.key();
    trace.reject_copy.set(true);
    assert!(caught(|| {
        MapNode::allocate_ref(&trace.arena, &key, std::ptr::null_mut());
    }));
    native_row(
        "copy_failure",
        format!("copy_failure {} {}", trace.events(), u8::from(trace.live())),
    );
}
#[test]
fn factory_panic_frees_the_uninitialized_node() {
    let trace = KeyTrace::new();
    assert!(caught(|| {
        MapNode::<TracedKey>::allocate_with(&trace.arena, std::ptr::null_mut(), || {
            assert!(trace.live());
            panic!("factory rejected")
        });
    }));
    assert!(!trace.live());
    assert!(trace.events().is_empty());
}
#[test]
fn safe_owner_releases_allocation_when_key_drop_panics() {
    let trace = KeyTrace::new();
    let owner = MapNode::allocate_with(&trace.arena, std::ptr::null_mut(), || trace.key());
    trace.reject_drop.set(true);
    assert!(caught(|| drop(owner)));
    assert_eq!(trace.events(), "D,");
    assert!(!trace.live());
}
#[test]
fn raw_deletion_releases_allocation_when_key_drop_panics() {
    let trace = KeyTrace::new();
    let owner = MapNode::allocate_with(&trace.arena, std::ptr::null_mut(), || trace.key());
    let (node, mut arena) = MapNode::into_raw(owner);
    trace.reject_drop.set(true);
    assert!(caught(|| unsafe {
        MapNode::<TracedKey>::del_node(node, &mut arena)
    }));
    assert_eq!(trace.events(), "D,");
    assert!(!trace.live());
}
#[derive(Clone)]
#[repr(align(256))]
struct AlignedKey(i32);
#[test]
fn owner_supports_aligned_keys_mutation_and_unowned_successor() {
    let arena = BaseAllocator::incremental(512);
    let mut tail = MapLinkNode::new(std::ptr::null_mut());
    let mut owner = MapNode::allocate_ref(&arena, &AlignedKey(7), std::ptr::from_mut(&mut tail));
    assert_eq!(std::ptr::from_ref(owner.key()).addr() % 256, 0);
    owner.change_value().0 = 9;
    assert_eq!(owner.key().0, 9);
    drop(owner);
    assert!(tail.next().is_null());
    assert!(arena.reset(false).is_ok());
}
#[test]
fn owner_supports_zero_sized_keys_and_retains_allocator_identity() {
    let arena = BaseAllocator::incremental(512);
    let owner = MapNode::allocate_ref(&arena, &(), std::ptr::null_mut());
    assert_eq!(owner.key(), &());
    let (node, mut returned) = MapNode::into_raw(owner);
    assert_eq!(arena, returned);
    unsafe { MapNode::<()>::del_node(node, &mut returned) };
    assert!(arena.reset(false).is_ok());
}
