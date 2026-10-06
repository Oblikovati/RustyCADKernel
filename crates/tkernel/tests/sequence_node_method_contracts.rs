//! Native SeqNode and Sequence<T>::Node methods with typed allocator provenance.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{BaseAllocator, SeqNode, Sequence, SequenceNode};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn seq_node_default_setters_and_copy_preserve_non_owning_links() {
    let mut node = SeqNode::new();
    let mut next = SeqNode::default();
    let mut previous = SeqNode::default();
    assert!(node.next().is_null() && node.previous().is_null());
    node.set_next(&mut next);
    node.set_previous(&mut previous);
    let copied = node;
    let mut assigned = SeqNode::default();
    assigned.clone_from(&node);
    assert_eq!(copied.next(), &raw mut next);
    assert_eq!(assigned.previous(), &raw mut previous);
    node.set_next(std::ptr::null_mut());
    node.set_previous(std::ptr::null_mut());
    assert!(node.next().is_null() && node.previous().is_null());
    assert_eq!(copied.next(), &raw mut next);
}

#[test]
fn seq_node_raw_allocation_reclaims_exact_prefix_owner() {
    let arena = BaseAllocator::incremental(4096);
    let owner = SeqNode::allocate(&arena);
    assert!(owner.next().is_null() && owner.previous().is_null());
    let (pointer, mut resource) = SeqNode::into_raw(owner);
    assert!(arena.reset(false).is_err());
    assert_eq!(resource, arena);
    // SAFETY: this is the untouched raw owner returned above, with its original allocator.
    unsafe { SeqNode::delete_raw(pointer, &mut resource) };
    assert!(arena.reset(false).is_ok());
}

#[test]
fn sequence_node_constructors_value_access_and_implicit_copy_keep_native_links() {
    let mut node = SequenceNode::with(|| "xxx".to_owned());
    node.change_value().push('y');
    let mut neighbor = SeqNode::default();
    node.change_base().set_next(&mut neighbor);
    let copied = node.clone();
    let mut assigned = SequenceNode::new("old".to_owned());
    assigned.clone_from(&node);
    assert_eq!(node.value(), "xxxy");
    assert_eq!(copied.value(), "xxxy");
    assert_eq!(assigned.value(), "xxxy");
    assert_eq!(copied.base().next(), &raw mut neighbor);
    assert_eq!(assigned.base().next(), &raw mut neighbor);
    let borrowed = SequenceNode::from_ref(node.value());
    assert!(borrowed.base().next().is_null());
    assert_eq!(&*borrowed, "xxxy");
    *assigned = "changed".to_owned();
    assert_eq!(assigned.value(), "changed");
}

#[repr(align(64))]
struct DropProbe {
    count: Rc<Cell<usize>>,
}
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.count.set(self.count.get() + 1);
    }
}
#[test]
fn sequence_node_raw_deleter_drops_once_and_preserves_alignment_and_neighbors() {
    let arena = BaseAllocator::incremental(4096);
    let count = Rc::new(Cell::new(0));
    let mut neighbor = SeqNode::new();
    let mut owner = SequenceNode::allocate(
        &arena,
        DropProbe {
            count: count.clone(),
        },
    );
    owner.change_base().set_next(&mut neighbor);
    let (pointer, mut resource) = SequenceNode::into_raw(owner);
    assert_eq!(pointer as usize % 64, 0);
    assert!(arena.reset(false).is_err());
    // SAFETY: exact DropProbe type, original resource, and unique raw owner are retained here.
    unsafe { Sequence::<DropProbe>::del_node(pointer, &mut resource) };
    assert_eq!(count.get(), 1);
    assert!(arena.reset(false).is_ok());
    assert!(neighbor.next().is_null());
}

#[test]
fn sequence_node_safe_delete_releases_value_and_storage() {
    let arena = BaseAllocator::incremental(4096);
    let count = Rc::new(Cell::new(0));
    let owner = SequenceNode::allocate_with(&arena, || {
        assert!(arena.reset(false).is_err());
        DropProbe {
            count: count.clone(),
        }
    });
    SequenceNode::delete(owner);
    assert_eq!(count.get(), 1);
    assert!(arena.reset(false).is_ok());
}

struct CopyProbe(BaseAllocator);
impl Clone for CopyProbe {
    fn clone(&self) -> Self {
        assert!(
            self.0.reset(false).is_err(),
            "expected allocated node before clone"
        );
        Self(self.0.clone())
    }
}
#[test]
fn sequence_node_borrowed_allocation_precedes_copy() {
    let arena = BaseAllocator::incremental(4096);
    let owner = SequenceNode::allocate_ref(&arena, &CopyProbe(arena.clone()));
    assert!(owner.base().previous().is_null());
    SequenceNode::delete(owner);
    assert!(arena.reset(false).is_ok());
}

#[test]
fn sequence_node_failed_constructor_frees_uninitialized_allocation() {
    let arena = BaseAllocator::incremental(4096);
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        SequenceNode::<i32>::allocate_with(&arena, || {
            assert!(arena.reset(false).is_err());
            panic!("injected constructor failure")
        });
    }));
    assert!(failed.is_err());
    assert!(arena.reset(false).is_ok());
}

fn assert_native_links(sequence: &Sequence<i32>) {
    for index in 1..=sequence.size() {
        let node = sequence.node(index).unwrap();
        let previous = sequence
            .node(index.wrapping_sub(1))
            .ok()
            .map_or(std::ptr::null_mut(), |n| {
                std::ptr::from_ref(n.base()).cast_mut()
            });
        let next = sequence
            .node(index + 1)
            .ok()
            .map_or(std::ptr::null_mut(), |n| {
                std::ptr::from_ref(n.base()).cast_mut()
            });
        assert_eq!(node.base().previous(), previous);
        assert_eq!(node.base().next(), next);
        assert_eq!(node.value(), sequence.value(index).unwrap());
    }
    assert!(sequence.node(0).is_err());
    assert!(sequence.node(sequence.size() + 1).is_err());
}

#[test]
fn sequence_existing_insert_and_remove_engine_updates_public_node_prefixes() {
    let mut sequence: Sequence<_> = [1, 2, 3].into_iter().collect();
    assert_native_links(&sequence);
    sequence.prepend(0);
    sequence.insert_after(2, 8).unwrap();
    assert_native_links(&sequence);
    sequence.remove(1).unwrap();
    assert_native_links(&sequence);
    sequence.remove_range(2, 3).unwrap();
    assert_native_links(&sequence);
    let mut cursor = sequence.cursor();
    sequence.remove_cursor(&mut cursor).unwrap();
    assert_native_links(&sequence);
    sequence.remove_cursor(&mut cursor).unwrap();
    assert!(sequence.is_empty());
}

#[test]
fn sequence_reverse_exchange_and_split_keep_link_identities() {
    let mut sequence: Sequence<_> = [1, 2, 3, 4, 5].into_iter().collect();
    sequence.reverse();
    assert_native_links(&sequence);
    sequence.exchange(1, 5).unwrap();
    assert_native_links(&sequence);
    sequence.exchange(2, 3).unwrap();
    assert_native_links(&sequence);
    let mut tail = Sequence::default();
    sequence.split(3, &mut tail).unwrap();
    assert_native_links(&sequence);
    assert_native_links(&tail);
    tail.split(1, &mut sequence).unwrap();
    assert_native_links(&sequence);
    assert!(tail.is_empty());
}

#[test]
fn sequence_splice_updates_only_link_boundaries_without_relocating_nodes() {
    let mut source: Sequence<_> = [2, 3, 4].into_iter().collect();
    let middle = std::ptr::from_ref(source.node(2).unwrap());
    let mut target: Sequence<_> = [1, 5].into_iter().collect();
    target.insert_sequence_after(1, &mut source).unwrap();
    assert!(source.is_empty());
    assert_native_links(&target);
    assert_eq!(std::ptr::from_ref(target.node(3).unwrap()), middle);
    source.append_sequence(&mut target);
    assert_native_links(&source);
    target.prepend_sequence(&mut source);
    assert_native_links(&target);
}

#[test]
fn sequence_node_links_do_not_remove_send_sync_for_owned_values() {
    fn requires_send_sync<T: Send + Sync>() {}
    requires_send_sync::<SeqNode>();
    requires_send_sync::<SequenceNode<i32>>();
    requires_send_sync::<Sequence<i32>>();
}
