#![allow(unsafe_code)]
//! Generic native iterator contracts, including readonly-only and offset custom nodes.
#[path = "base_list_basic_iterator_contracts/fixture.rs"]
mod fixture;
use fixture::{NativeList, Readonly, Writable, oracle};
use rustycad_tkernel::ncollection::{
    BaseListBasicIterator, BaseListRawIterator, BaseListStorage, BasicListNode, ListLinkNode,
    ListValueNode,
};
use std::ptr::null_mut;

#[test]
fn custom_list_dispatch_matches_native_const_and_mutable_value() {
    let mut list = NativeList::new();
    let mut cursor = Writable::new(&mut list.base);
    oracle(&format!(
        "initial {} {}",
        cursor.more() as u8,
        cursor.raw.previous.is_null() as u8
    ));
    unsafe {
        *cursor.value() = 8;
        let _ = cursor.dereference();
        let pointer = cursor.arrow();
        *cursor.change_value() += 2;
        let readonly = Readonly::from_mutable(&cursor);
        let _ = readonly.value();
        let _ = readonly.dereference();
        let const_pointer = readonly.arrow();
        let node = &*list.nodes[0];
        oracle(&format!(
            "dispatch {} {} {} {}",
            node.item,
            node.reads.get(),
            node.writes,
            std::ptr::eq(pointer, const_pointer) as u8
        ));
    }
}

#[test]
fn generic_list_movement_retains_native_predecessor_at_end() {
    let mut list = NativeList::new();
    let mut cursor = Writable::new(&mut list.base);
    unsafe {
        let old = cursor.postfix_increment();
        let second = cursor.raw.current == list.link(1) && cursor.raw.previous == list.link(0);
        cursor.next();
        let previous = cursor.raw.previous;
        let address = std::ptr::from_mut(&mut cursor);
        assert!(std::ptr::eq(cursor.prefix_increment(), address));
        oracle(&format!(
            "movement {} {} {} {}",
            (old.raw.current == list.link(0)) as u8,
            second as u8,
            !cursor.more() as u8,
            (previous == cursor.raw.previous) as u8
        ));
    }
}

#[test]
fn list_const_conversion_and_assignment_copy_both_native_links() {
    let mut list = NativeList::new();
    let mut cursor = Writable::new(&mut list.base);
    unsafe { cursor.next() };
    let converted = Readonly::from_mutable(&cursor);
    let mut assigned = Readonly::default();
    let destination = std::ptr::from_mut(&mut assigned);
    assert!(std::ptr::eq(assigned.assign_mutable(&cursor), destination));
    oracle(&format!(
        "conversion {} {}",
        (converted.raw.current == list.link(1) && converted.raw.previous == list.link(0)) as u8,
        (assigned == cursor) as u8
    ));
    let mut copy = Readonly::default();
    copy.assign(&assigned);
    assert!(copy == converted);
    copy.assign_self();
    assert!(copy == converted);
}

#[test]
fn generic_list_equality_ignores_previous_across_const_specializations() {
    let mut list = NativeList::new();
    let mut cursor = Writable::new(&mut list.base);
    let old = cursor;
    unsafe { cursor.next() };
    let assigned = Readonly::from_mutable(&cursor);
    let mut differing = cursor;
    differing.raw.previous = null_mut();
    oracle(&format!(
        "equality {} {} {}",
        (differing == cursor) as u8,
        (cursor != old) as u8,
        assigned.is_equal(&cursor) as u8
    ));
}

#[test]
fn list_initialization_aliases_reset_head_and_preserve_view_identity() {
    let mut list = NativeList::new();
    let mut cursor = Writable::default();
    cursor.init(&mut list.base);
    unsafe { cursor.next() };
    cursor.initialize(&mut list.base);
    let mut readonly = Readonly::default();
    readonly.init(&list.base);
    let mut assigned = Readonly::new(&list.base);
    assigned.initialize(&list.base);
    let address = std::ptr::from_mut(&mut cursor);
    let const_identity = std::ptr::eq(cursor.iterator(), address);
    let mut_identity = std::ptr::eq(cursor.change_iterator(), address);
    oracle(&format!(
        "reset {} {} {} {}",
        (cursor.raw.current == list.link(0)) as u8,
        (readonly == assigned) as u8,
        const_identity as u8,
        mut_identity as u8
    ));
}

#[test]
fn custom_list_node_downcast_adjusts_nonzero_embedded_offset() {
    let list = NativeList::new();
    let node = list.nodes[0];
    unsafe { (*node).item = 10 };
    let cursor = Readonly::new(&list.base);
    let adjusted = !std::ptr::eq(list.nodes[0].cast::<()>(), list.link(0).cast::<()>());
    oracle(&format!("offset {} {}", adjusted as u8, unsafe {
        cursor.value()
    }));
}

#[repr(C)]
struct ReadOnlyNode {
    link: ListLinkNode,
    item: i32,
}
// SAFETY: the embedded link is the first repr(C) field.
unsafe impl BasicListNode for ReadOnlyNode {
    type Item = i32;
    unsafe fn from_list_link(link: *mut ListLinkNode) -> *mut Self {
        link.cast()
    }
    fn list_value(&self) -> &i32 {
        &self.item
    }
}
#[test]
fn readonly_node_needs_no_mutable_value_member() {
    let mut node = ReadOnlyNode {
        link: ListLinkNode::new(null_mut()),
        item: 42,
    };
    let cursor = BaseListBasicIterator::<ReadOnlyNode>::from_raw(BaseListRawIterator {
        current: std::ptr::from_mut(&mut node).cast(),
        previous: null_mut(),
    });
    unsafe {
        oracle(&format!(
            "readonly_only {} {} {}",
            cursor.value(),
            cursor.dereference(),
            *cursor.arrow()
        ))
    };
}

#[test]
fn default_list_cursor_advances_without_dereferencing_null() {
    let mut empty = Writable::default();
    let before = unsafe { empty.postfix_increment() };
    unsafe { empty.next() };
    oracle(&format!(
        "empty {} {} {}",
        !empty.more() as u8,
        (before == empty) as u8,
        empty.raw.previous.is_null() as u8
    ));
}

#[test]
fn built_in_list_value_node_uses_native_projection_and_mutation() {
    let mut node = ListValueNode::new(String::from("OCCT"), null_mut());
    let mut base = BaseListStorage {
        first: std::ptr::from_mut(&mut node).cast(),
        ..BaseListStorage::default()
    };
    let mut cursor = BaseListBasicIterator::<ListValueNode<String>, true>::new(&mut base);
    unsafe { cursor.change_value().push_str(" Rust") };
    let readonly = BaseListBasicIterator::<ListValueNode<String>>::from_mutable(&cursor);
    assert_eq!(unsafe { readonly.value() }, "OCCT Rust");
}
