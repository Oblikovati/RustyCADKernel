#![allow(unsafe_code)]
//! Native generic iterator behavior plus explicit Rust adaptations of two ill-formed overloads.
use rustycad_tkernel::ncollection::BaseMapBasicIterator;
#[path = "base_map_basic_iterator_contracts/support.rs"]
mod support;
use support::{Node, ProbeMap, with_nodes};
type Read<'a> = BaseMapBasicIterator<'a, Node>;
type Write<'a> = BaseMapBasicIterator<'a, Node, true>;

#[test]
fn all_empty_forms_compare_by_null_node_and_next_is_noop() {
    let mut empty = Read::default();
    let map = ProbeMap::unallocated();
    let unallocated = Read::new(&map);
    let allocated_map = ProbeMap::new([std::ptr::null_mut(); 3]);
    let allocated = Read::new(&allocated_map);
    // SAFETY: a null current node makes advance a no-op.
    unsafe { empty.advance() };
    assert!(empty == unallocated && empty == allocated);
    assert!(!empty.more() && empty.node().is_null());
}

#[test]
fn mapped_writable_and_readonly_dispatch_use_different_native_getters() {
    with_nodes(|nodes, calls| {
        let mut cursor = Write::from_node(nodes[0]);
        // SAFETY: with_nodes owns the node, and each temporary borrow ends before the next.
        unsafe {
            assert_eq!(*cursor.value(), 101);
            *cursor.change_value() = 111;
            assert_eq!(cursor.value_ptr(), cursor.change_value_ptr());
            assert_eq!(*cursor.key(), 1);
        }
        assert_eq!(
            (calls.reads.get(), calls.writes.get(), calls.keys.get()),
            (0, 4, 1)
        );
        let readonly = Read::from_node(nodes[0]);
        // SAFETY: the node remains live, and no value mutation occurs during these shared reads.
        unsafe {
            assert_eq!(*readonly.value(), 111);
            assert!(!readonly.value_ptr().is_null());
        }
        assert_eq!(calls.reads.get(), 2);
    });
}

#[test]
fn bucket_and_raw_node_constructors_have_different_traversal_extent() {
    with_nodes(|nodes, _| {
        let map = ProbeMap::new(nodes);
        let mut cursor = Read::new(&map);
        let mut keys = Vec::new();
        while cursor.more() {
            // SAFETY: all nodes and acyclic links stay live inside with_nodes.
            unsafe {
                keys.push(*cursor.key());
                cursor.advance();
            }
        }
        assert_eq!(keys, [1, 2, 3]);
        let mut raw = Read::from_node(nodes[0]);
        // SAFETY: a->b is the complete raw-node chain, without container bucket state.
        unsafe {
            raw.advance();
            assert_eq!(*raw.key(), 2);
            raw.advance();
        }
        assert!(raw == cursor && !raw.more());
    });
}

#[test]
fn prefix_postfix_copy_and_assign_return_native_identities() {
    with_nodes(|nodes, _| {
        let mut map = ProbeMap::new(nodes);
        let mut cursor = Write::new(&mut map);
        // SAFETY: the callback owns all live nodes for the duration of these steps.
        let previous = unsafe { cursor.advance_post() };
        assert_eq!(previous.node(), nodes[0].cast_const());
        let identity = std::ptr::from_mut(&mut cursor);
        // SAFETY: current b is live and its next step reaches the next bucket c.
        assert_eq!(unsafe { cursor.advance_returning() } as *mut _, identity);
        assert_eq!(cursor.node(), nodes[2].cast_const());
        let mut target = Write::default();
        let target_ptr = std::ptr::from_mut(&mut target);
        assert_eq!(target.assign(&previous) as *mut _, target_ptr);
        assert_eq!(target.assign_self() as *mut _, target_ptr);
        assert!(target == previous && target != cursor);
    });
}

#[test]
fn const_conversion_is_a_documented_extension_of_illformed_native_overloads() {
    with_nodes(|nodes, _| {
        let writable = Write::from_node(nodes[0]);
        let converted = Read::from_mutable(&writable);
        assert!(converted == writable && writable == converted);
        let mut assigned = Read::default();
        let identity = std::ptr::from_mut(&mut assigned);
        assert_eq!(assigned.assign_from_mutable(&writable) as *mut _, identity);
        assert!(assigned == converted && assigned == writable.as_const());
    });
}

#[test]
fn unordered_keyonly_uses_value_for_key_and_never_mutable_getter() {
    with_nodes(|nodes, calls| {
        let cursor = BaseMapBasicIterator::<Node, true, true>::from_node(nodes[0]);
        // SAFETY: key-only borrows remain shared throughout the live-node callback.
        unsafe {
            assert_eq!(*cursor.key(), 101);
            assert_eq!(*cursor.value(), 101);
            assert!(!cursor.value_ptr().is_null());
        }
        assert_eq!(
            (calls.reads.get(), calls.writes.get(), calls.keys.get()),
            (3, 0, 0)
        );
    });
}

#[test]
fn ordered_cursor_follows_order_links_and_keyonly_keeps_node_key() {
    with_nodes(|nodes, calls| {
        let map = ProbeMap::new(nodes);
        let mut cursor = BaseMapBasicIterator::<Node, false, true, true>::new(&map);
        let mut keys = Vec::new();
        while cursor.more() {
            // SAFETY: ordered chain c->a->b is live and acyclic during this callback.
            unsafe {
                keys.push(*cursor.key());
                cursor.advance();
            }
        }
        assert_eq!(keys, [3, 1, 2]);
        assert_eq!((calls.buckets.get(), calls.orders.get()), (0, 3));
    });
}

#[test]
fn same_node_compares_equal_across_independent_bucket_tables() {
    with_nodes(|nodes, _| {
        let left_map = ProbeMap::new(nodes);
        let mut right_map = ProbeMap::new(nodes);
        right_map.storage.bucket_slots_mut()[2] = std::ptr::null_mut();
        right_map.storage.bucket_slots_mut()[0] = nodes[0].cast();
        let left = Read::new(&left_map);
        let right = Read::new(&right_map);
        assert!(left == right && left.is_equal(&right));
    });
}

#[path = "base_map_basic_iterator_contracts/oracle_rows.rs"]
mod oracle_rows;

#[test]
fn native_basic_iterator_rows_match() {
    oracle_rows::compare_all_rows();
}
