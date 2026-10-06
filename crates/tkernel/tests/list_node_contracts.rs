//! Non-owning native ListNode links and correctly paired TListNode destruction.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{BaseAllocator, ListLinkNode, ListValueNode};
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::null_mut;
use std::rc::Rc;

fn assert_live(arena: &BaseAllocator, count: usize) {
    let failure = arena.reset(false).unwrap_err();
    assert!(failure.message().starts_with(&format!("{count} live")));
}
#[test]
fn native_link_and_value_accessors_preserve_pointer_identity() {
    let mut tail = ListLinkNode::new(null_mut());
    let tail_pointer = std::ptr::from_mut(&mut tail);
    let mut node = ListValueNode::new(7, tail_pointer);
    assert_eq!(node.link().next(), tail_pointer);
    assert_eq!(node.value(), &7);
    let value_pointer = std::ptr::from_ref(node.value());
    assert_eq!(
        std::ptr::from_mut(node.change_value()).cast_const(),
        value_pointer
    );
    *node.change_value() = 8;
    *node.link_mut().next_mut() = null_mut();
    assert!(node.link().next().is_null());
    assert!(tail.next().is_null());
    assert_eq!(node.value(), &8);
}
#[test]
fn borrowed_and_in_place_stack_constructors_build_values() {
    let text = String::from("native node");
    let copy = ListValueNode::from_ref(&text, null_mut());
    let constructed = ListValueNode::construct(null_mut(), || String::from("created"));
    assert_eq!(copy.value(), &text);
    assert_ne!(copy.value().as_ptr(), text.as_ptr());
    assert_eq!(constructed.value(), "created");
}
#[derive(Clone)]
struct DropCount(Rc<Cell<usize>>);
impl Drop for DropCount {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
#[test]
fn raw_deleter_reclaims_exactly_one_value_and_preserves_successor() {
    let arena = BaseAllocator::incremental(1024);
    let drops = Rc::new(Cell::new(0));
    let mut tail = ListLinkNode::new(null_mut());
    let owner = ListValueNode::allocate_with(&arena, &mut tail, || {
        assert_live(&arena, 1);
        DropCount(drops.clone())
    });
    let (pointer, mut resource) = ListValueNode::into_raw(owner);
    assert_eq!(resource, arena);
    // SAFETY: original node type, pointer and allocator; no references remain.
    unsafe { ListValueNode::<DropCount>::del_node(pointer, &mut resource) };
    assert_eq!(drops.get(), 1);
    assert!(tail.next().is_null());
    assert!(arena.reset(false).is_ok());
}
struct ObserveClone(BaseAllocator);
impl Clone for ObserveClone {
    fn clone(&self) -> Self {
        assert_live(&self.0, 1);
        Self(self.0.clone())
    }
}
#[test]
fn allocated_borrowed_constructor_copies_after_allocation() {
    let arena = BaseAllocator::incremental(1024);
    let source = ObserveClone(arena.clone());
    let owner = ListValueNode::allocate_ref(&arena, &source, null_mut());
    assert!(owner.link().next().is_null());
    let (pointer, mut allocator) = ListValueNode::into_raw(owner);
    // SAFETY: original node type and allocator are transferred once.
    unsafe { ListValueNode::<ObserveClone>::del_node(pointer, &mut allocator) };
    assert!(arena.reset(false).is_ok());
}
#[test]
fn panicking_placement_constructor_frees_uninitialized_storage() {
    let arena = BaseAllocator::incremental(1024);
    let result = catch_unwind(AssertUnwindSafe(|| {
        ListValueNode::<i32>::allocate_with(&arena, null_mut(), || {
            assert_live(&arena, 1);
            panic!("injected node construction failure")
        });
    }));
    assert!(result.is_err());
    assert!(arena.reset(false).is_ok());
}
#[repr(align(128))]
struct AlignedValue(DropCount);
#[test]
fn raw_deleter_retains_full_typed_alignment_and_resource_layout() {
    let arena = BaseAllocator::common();
    let drops = Rc::new(Cell::new(0));
    let owner = ListValueNode::allocate_with(&arena, null_mut(), || {
        AlignedValue(DropCount(drops.clone()))
    });
    assert_eq!(std::ptr::from_ref(owner.value()) as usize % 128, 0);
    assert_eq!(owner.value().0.0.get(), 0);
    let (pointer, mut resource) = ListValueNode::into_raw(owner);
    // SAFETY: reconstruct the original over-aligned concrete type with its resource.
    unsafe { ListValueNode::<AlignedValue>::del_node(pointer, &mut resource) };
    assert_eq!(drops.get(), 1);
}
#[test]
fn zero_sized_value_still_owns_a_real_link_node_allocation() {
    let arena = BaseAllocator::incremental(1024);
    let owner = ListValueNode::allocate_with(&arena, null_mut(), || ());
    assert_live(&arena, 1);
    let (pointer, mut resource) = ListValueNode::into_raw(owner);
    assert!(!pointer.is_null());
    // SAFETY: matching zero-sized item type; the prefix gives this node a nonzero size.
    unsafe { ListValueNode::<()>::del_node(pointer, &mut resource) };
    assert!(arena.reset(false).is_ok());
}
