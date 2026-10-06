#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{BaseAllocator, PackedMapNode};
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn allocated_nodes_transfer_without_owning_their_successor() {
    let arena = BaseAllocator::incremental(4096);
    let mut tail = PackedMapNode::<i32>::allocate_with(&arena, PackedMapNode::default);
    let tail_pointer = std::ptr::from_mut(&mut *tail);
    let owner = PackedMapNode::allocate_with(&arena, || PackedMapNode::new(tail_pointer));
    let (pointer, resource) = PackedMapNode::into_raw(owner);
    assert_eq!(resource, arena);
    let mut restored = unsafe { PackedMapNode::from_raw(pointer, resource) };
    assert_eq!(restored.next(), tail_pointer);
    restored.set_next(std::ptr::null_mut());
    drop(restored);
    assert_eq!(tail.nb_values(), 1);
    assert!(arena.reset(false).is_err());
    drop(tail);
    assert!(arena.reset(false).is_ok());
}

#[test]
fn node_allocation_precedes_constructor_and_releases_on_drop() {
    let arena = BaseAllocator::incremental(4096);
    let owner = PackedMapNode::<i32>::allocate_with(&arena, || {
        assert!(arena.reset(false).is_err());
        PackedMapNode::default()
    });
    drop(owner);
    assert!(arena.reset(false).is_ok());
}

#[test]
fn panicking_node_constructor_releases_uninitialized_storage() {
    let arena = BaseAllocator::incremental(4096);
    let caught = catch_unwind(AssertUnwindSafe(|| {
        PackedMapNode::<i32>::allocate_with(&arena, || panic!("rejected node construction"));
    }));
    assert!(caught.is_err());
    assert!(arena.reset(false).is_ok());
}

#[test]
fn node_copy_keeps_independent_bits_and_the_same_unowned_link() {
    let mut tail = PackedMapNode::<i32>::default();
    let pointer = std::ptr::from_mut(&mut tail);
    let original = PackedMapNode::from_parts(32, 1, pointer);
    let mut copied = original;
    assert!(copied.add_value(1));
    assert_eq!(original.bits(), 1);
    assert_eq!(copied.bits(), 3);
    assert_eq!(copied.next(), original.next());
}
