//! Native BaseList subclass operations, shared node prefix, and callback lifecycle.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{
    BaseAllocator, BaseListRawIterator, BaseListStorage, DelListNode, ListValueNode,
};
use std::ptr::null_mut;
#[path = "base_list_storage_contracts/fixture.rs"]
mod fixture;
use fixture::{OwnedRawList, delete_i32, oracle_row};

#[test]
fn raw_append_sets_endpoints_and_output_predecessor() {
    let mut list = OwnedRawList::new(&[]);
    let mut cursor = BaseListRawIterator::default();
    let node = list.make(1);
    // SAFETY: fresh disjoint node and exclusive valid empty chain.
    unsafe { list.0.append_at(node, &mut cursor) };
    assert_eq!(cursor.current, node);
    assert!(cursor.previous.is_null());
    assert_eq!(list.0.first(), list.0.first_mut().cast_const());
    assert_eq!(list.0.last(), list.0.last_mut().cast_const());
    oracle_row(&format!(
        "first_append {} {} {} {}",
        list.0.extent(),
        list.0.length(),
        u8::from(cursor.previous.is_null()),
        u8::from(list.0.last() == node)
    ));
}
#[test]
fn insert_before_updates_previous_and_remove_preserves_it() {
    let mut list = OwnedRawList::new(&[1, 2]);
    let mut cursor = BaseListRawIterator::default();
    // SAFETY: current first and resulting second node belong to this exclusive chain.
    unsafe { cursor.init_after(&list.0, list.0.first().cast_mut()) };
    let node = list.make(3);
    // SAFETY: fresh node and current/predecessor match the destination chain.
    unsafe { list.0.insert_before_node(node, &mut cursor).unwrap() };
    oracle_row(&list.row("before"));
    assert_eq!(cursor.previous, node);
    // SAFETY: current is live and delete_i32 matches its type/resource.
    unsafe { list.0.remove(&mut cursor, delete_i32).unwrap() };
    assert!(!cursor.more());
    assert_eq!(cursor.previous, node);
    oracle_row(&format!(
        "remove {} {} {}",
        u8::from(cursor.more()),
        u8::from(cursor.previous == node),
        list.values().last().unwrap()
    ));
}
#[test]
fn reverse_and_clear_follow_native_callback_order() {
    let mut list = OwnedRawList::new(&[1, 3]);
    let mut cursor = BaseListRawIterator::new(&list.0);
    let node = list.make(4);
    // SAFETY: fresh node is inserted into an exclusive valid chain.
    unsafe {
        list.0.insert_after_node(node, &mut cursor).unwrap();
        list.0.reverse()
    };
    oracle_row(&list.row("reverse"));
    let mut removed = Vec::new();
    // SAFETY: every callback receives one of this fixture's live concrete nodes.
    unsafe {
        list.0.clear(|node, allocator| {
            removed.push(*(*node.cast::<ListValueNode<i32>>()).value());
            delete_i32(node, allocator);
        })
    };
    assert_eq!(removed, [3, 4, 1]);
    assert_empty_storage(&list.0);
}
#[test]
fn raw_list_insert_after_transfers_links_without_copying() {
    let mut destination = OwnedRawList::new(&[1, 4]);
    let mut source = OwnedRawList::new(&[2, 3]);
    let node = source.0.first();
    let mut cursor = BaseListRawIterator::new(&destination.0);
    // SAFETY: disjoint owned chains use equal common allocators; cursor names the head.
    unsafe {
        destination
            .0
            .insert_list_after(&mut source.0, &mut cursor)
            .unwrap()
    };
    oracle_row(&destination.row("after_list"));
    assert_empty_donor(&source);
    // SAFETY: predecessor names the live head of the destination chain.
    unsafe { cursor.init_after(&destination.0, cursor.current) };
    assert_eq!(cursor.current.cast_const(), node);
}
#[test]
fn raw_list_insert_before_head_updates_both_chains_and_cursor() {
    let mut destination = OwnedRawList::new(&[1, 2, 3, 4]);
    let mut source = OwnedRawList::new(&[0]);
    let mut cursor = BaseListRawIterator::new(&destination.0);
    let node = source.0.first_mut();
    // SAFETY: disjoint chains and head cursor; their allocators are equal.
    unsafe {
        destination
            .0
            .insert_list_before(&mut source.0, &mut cursor)
            .unwrap()
    };
    oracle_row(&destination.row("before_head_list"));
    assert_eq!(cursor.previous, node);
    assert_eq!(cursor.current.cast_const(), unsafe {
        (*node).next().cast_const()
    });
    assert!(source.0.is_empty());
}
fn insert_middle() -> (OwnedRawList, BaseListRawIterator) {
    let mut destination = OwnedRawList::new(&[1, 4]);
    let mut source = OwnedRawList::new(&[2, 3]);
    let mut cursor = BaseListRawIterator::default();
    // SAFETY: first node is a live predecessor; chains are disjoint and allocator-compatible.
    unsafe {
        cursor.init_after(&destination.0, destination.0.first().cast_mut());
        destination
            .0
            .insert_list_before(&mut source.0, &mut cursor)
            .unwrap();
    }
    oracle_row(&destination.row("middle_before"));
    (destination, cursor)
}
#[test]
fn middle_before_preserves_cursor_for_empty_donor() {
    let (mut destination, mut cursor) = insert_middle();
    let previous = cursor.previous;
    let mut source = OwnedRawList::new(&[]);
    // SAFETY: empty donor and unchanged valid destination cursor.
    let result = unsafe { destination.0.insert_list_before(&mut source.0, &mut cursor) };
    result.unwrap();
    let same = u8::from(cursor.previous == previous);
    oracle_row(&format!("empty_before {same} {}", u8::from(cursor.more())));
}
#[test]
fn insert_after_tail_transfers_nonempty_chain() {
    let (mut destination, mut cursor) = insert_middle();
    let mut tail = OwnedRawList::new(&[5]);
    // SAFETY: current names the tail; donor chain is disjoint with equal allocator.
    let result = unsafe { destination.0.insert_list_after(&mut tail.0, &mut cursor) };
    result.unwrap();
    oracle_row(&destination.row("tail_after"));
}
#[test]
fn append_and_prepend_transfer_whole_chains_with_native_self_no_op() {
    let mut destination = OwnedRawList::new(&[0, 1, 2, 3, 4]);
    let mut tail = OwnedRawList::new(&[5]);
    // SAFETY: both live chains are disjoint and allocator-compatible.
    unsafe { destination.0.append_list(&mut tail.0) };
    destination.0.splice_self();
    oracle_row(&destination.row("append_self_prepend"));
    let mut head = OwnedRawList::new(&[-1]);
    // SAFETY: live disjoint chain and matching allocator.
    unsafe { destination.0.prepend_list(&mut head.0) };
    oracle_row(&destination.row("prepend_list"));
}
#[test]
fn protected_null_predecessor_init_selects_tail() {
    let mut list = OwnedRawList::new(&[1, 2, 3, 4, 5]);
    let mut cursor = BaseListRawIterator::default();
    // SAFETY: null predecessor reads no pointer, native init selects the stored tail.
    unsafe { cursor.init_after(&list.0, null_mut()) };
    assert_eq!(cursor.current.cast_const(), list.0.last());
    assert!(cursor.previous.is_null());
    cursor.initialize(&list.0);
    assert_eq!(cursor, BaseListRawIterator::new(&list.0));
    let mut empty = OwnedRawList::new(&[]);
    // SAFETY: valid disjoint destination and source, common allocator.
    unsafe { empty.0.append_list(&mut list.0) };
    oracle_row(&empty.row("append_into_empty"));
    // SAFETY: valid nonempty typed i32 chain with matching deleter.
    unsafe { empty.0.remove_first(delete_i32).unwrap() };
    oracle_row(&empty.row("remove_first"));
}
#[test]
fn empty_raw_operations_report_native_exceptions() {
    let mut list = OwnedRawList::new(&[]);
    let mut source = OwnedRawList::new(&[]);
    let mut cursor = BaseListRawIterator::default();
    // SAFETY: every branch validates the null position before any link dereference.
    let failures = unsafe {
        [
            list.0.remove_first(delete_i32),
            list.0.remove(&mut cursor, delete_i32),
            list.0.insert_list_before(&mut source.0, &mut cursor),
            list.0.insert_list_after(&mut source.0, &mut cursor),
        ]
    };
    for failure in failures {
        assert_eq!(
            failure.unwrap_err().exception_type(),
            "Standard_NoSuchObject"
        );
    }
    oracle_row("empty_errors 4");
}
#[test]
fn exchange_swaps_allocator_and_node_ownership_together() {
    let arena = BaseAllocator::incremental(1024);
    let mut foreign = OwnedRawList(BaseListStorage::new(Some(arena.clone())));
    let node = foreign.make(7);
    // SAFETY: fresh node matches the foreign storage allocator.
    unsafe { foreign.0.prepend_node(node) };
    let mut list = OwnedRawList::new(&[]);
    list.0.exchange(&mut foreign.0);
    assert_eq!(list.0.allocator(), &arena);
    assert_eq!(list.values(), [7]);
    assert!(foreign.0.is_empty());
    oracle_row(&format!(
        "exchange {} {} {}",
        list.values()[0],
        u8::from(list.0.allocator() == &arena),
        u8::from(foreign.0.is_empty())
    ));
    assert_eq!(foreign.0.allocator_mut(), &mut BaseAllocator::common());
}
#[test]
fn signed_counts_and_empty_query_follow_distinct_native_fields() {
    let storage = BaseListStorage {
        length: usize::MAX,
        ..Default::default()
    };
    assert_eq!(storage.extent(), -1);
    assert_eq!(storage.length(), -1);
    assert_eq!(storage.size(), usize::MAX);
    assert!(storage.is_empty());
    let first = BaseListRawIterator {
        previous: std::ptr::dangling_mut(),
        ..Default::default()
    };
    assert_eq!(first, BaseListRawIterator::default());
}

#[test]
fn singleton_removal_clears_both_endpoints_with_native_deleter_type() {
    let mut list = OwnedRawList::new(&[7]);
    let mut cursor = BaseListRawIterator::new(&list.0);
    let deleter: DelListNode = ListValueNode::<i32>::del_node;
    // SAFETY: node has the deleter's exact type and matching allocator; cursor names the head.
    unsafe {
        list.0
            .remove(&mut cursor, |node, allocator| deleter(node, allocator))
            .unwrap()
    };
    assert!(!cursor.more());
    assert!(cursor.previous.is_null());
    assert!(list.0.first().is_null() && list.0.last().is_null());
    assert_eq!(list.0.size(), 0);
}
#[test]
fn empty_destination_prepend_and_empty_source_splices_preserve_endpoints() {
    let mut destination = OwnedRawList::new(&[]);
    let mut source = OwnedRawList::new(&[1]);
    let node = source.0.first();
    // SAFETY: disjoint valid chains use matching common allocators.
    unsafe {
        destination.0.prepend_list(&mut source.0);
        destination.0.append_list(&mut source.0);
        destination.0.prepend_list(&mut source.0);
        destination.0.reverse()
    };
    assert_eq!(destination.0.first(), node);
    assert_eq!(destination.0.last(), node);
    assert_eq!(destination.0.size(), 1);
}

fn assert_empty_storage(storage: &BaseListStorage) {
    assert!(storage.is_empty());
    assert_eq!(storage.size(), 0);
    assert!(storage.first().is_null() && storage.last().is_null());
}
fn assert_empty_donor(source: &OwnedRawList) {
    let empty = u8::from(source.0.is_empty());
    let tail_null = u8::from(source.0.last().is_null());
    oracle_row(&format!(
        "donor_empty {empty} {} {tail_null}",
        source.0.size()
    ));
}
