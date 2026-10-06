//! Native indexed node constructors, inherited prefixes, mutable fields and allocator callbacks.
#![allow(unsafe_code)]
// Reuse the established item/copy/Drop trace; its non-indexed convenience constructors are unused here.
#[allow(dead_code)]
#[path = "data_map_node_contracts/fixture.rs"]
mod fixture;
#[path = "indexed_native_node_contracts/lifetime.rs"]
mod lifetime;
use fixture::{NodeItem, NodeTrace};
use rustycad_tkernel::ncollection::{IndexedDataMapNode, IndexedMapNode, MapLinkNode};
fn native_row(prefix: &str, actual: String) {
    let expected = include_str!("../../../tools/oracle/tkernel_indexed_native_nodes.expected");
    let row = expected
        .lines()
        .find(|row| row.starts_with(&format!("{prefix} ")))
        .unwrap();
    assert_eq!(actual, row);
}
fn data_constructor(
    mode: usize,
    trace: &NodeTrace,
    key: &NodeItem,
    value: &NodeItem,
) -> IndexedDataMapNode<NodeItem, NodeItem> {
    let next = std::ptr::null_mut();
    match mode {
        0 => IndexedDataMapNode::from_refs(key, -7, value, next),
        1 => IndexedDataMapNode::from_key_ref(key, -7, trace.item('V'), next),
        2 => IndexedDataMapNode::from_value_ref(trace.item('K'), -7, value, next),
        3 => IndexedDataMapNode::new(trace.item('K'), -7, trace.item('V'), next),
        4 => IndexedDataMapNode::construct_ref(key, -7, next, || trace.item('V')),
        _ => IndexedDataMapNode::construct(trace.item('K'), -7, next, || trace.item('V')),
    }
}
#[test]
fn all_native_indexed_data_constructor_overloads_preserve_copy_and_drop_order() {
    let labels = [
        "borrowed",
        "key_borrowed",
        "value_borrowed",
        "owned",
        "emplace_borrowed",
        "emplace_owned",
    ];
    for (mode, label) in labels.iter().enumerate() {
        let trace = NodeTrace::new();
        let (key, value) = (trace.item('K'), trace.item('V'));
        let node = data_constructor(mode, &trace, &key, &value);
        assert_eq!(node.index_value(), -7);
        drop(node);
        native_row(label, format!("{label} {}", trace.events()));
    }
}
#[test]
fn all_native_indexed_key_constructor_overloads_preserve_copy_and_drop_order() {
    for (mode, label) in ["map_borrowed", "map_owned", "map_emplace"]
        .iter()
        .enumerate()
    {
        let trace = NodeTrace::new();
        let key = trace.item('K');
        let node = match mode {
            0 => IndexedMapNode::from_ref(&key, -7, std::ptr::null_mut()),
            1 => IndexedMapNode::new(trace.item('K'), -7, std::ptr::null_mut()),
            _ => IndexedMapNode::construct(-7, std::ptr::null_mut(), || trace.item('K')),
        };
        assert_eq!(node.index_value(), -7);
        drop(node);
        native_row(label, format!("{label} {}", trace.events()));
    }
}
fn key_prefix_row(node: &mut IndexedMapNode<i32>, next: *mut MapLinkNode) -> String {
    // Flat native fields share the link prefix, not a Rust base spanning derived padding.
    assert_eq!(node.value(), &7);
    let prefix = std::ptr::from_ref(node.link()).cast::<()>() == std::ptr::from_ref(node).cast();
    let same_value =
        std::ptr::from_mut(node.key1()).cast_const() == std::ptr::from_ref(node.value());
    format!(
        "map_fields {} {} {} {}",
        u8::from(prefix),
        u8::from(node.link().next() == next),
        u8::from(same_value),
        node.index_value()
    )
}
#[test]
fn native_indexed_key_fields_alias_actual_prefix_and_keep_signed_index() {
    let mut tail = MapLinkNode::new(std::ptr::null_mut());
    let next = std::ptr::from_mut(&mut tail);
    let mut node = IndexedMapNode::new(7, -7, next);
    let mut row = key_prefix_row(&mut node, next);
    *node.key1() = 8;
    *node.index() = 9;
    *node.change_value() = 10;
    *node.link_mut().next_mut() = std::ptr::null_mut();
    row.push_str(&format!(
        " {} {} {}",
        node.value(),
        node.index_value(),
        u8::from(node.link().next().is_null())
    ));
    native_row("map_fields", row);
}
fn data_prefix_row(node: &IndexedDataMapNode<i32, i32>, next: *mut MapLinkNode) -> String {
    // Flat native fields share the link prefix, not a Rust base spanning derived padding.
    assert_eq!(node.value(), &7);
    let prefix = std::ptr::from_ref(node.link()).cast::<()>() == std::ptr::from_ref(node).cast();
    format!(
        "data_fields {} {} {} {} {}",
        u8::from(prefix),
        u8::from(node.link().next() == next),
        node.key(),
        node.value(),
        node.index_value()
    )
}
#[test]
fn native_indexed_data_fields_mutate_single_actual_node() {
    let mut tail = MapLinkNode::new(std::ptr::null_mut());
    let next = std::ptr::from_mut(&mut tail);
    let mut node = IndexedDataMapNode::new(1, -7, 7, next);
    let mut row = data_prefix_row(&node, next);
    *node.key1() = 2;
    *node.change_value() = 8;
    *node.index() = 9;
    *node.link_mut().next_mut() = std::ptr::null_mut();
    row.push_str(&format!(
        " {} {} {} {}",
        node.key(),
        node.value(),
        node.index_value(),
        u8::from(node.link().next().is_null())
    ));
    native_row("data_fields", row);
}
#[test]
fn allocated_indexed_key_node_mutation_preserves_original_address() {
    let arena = rustycad_tkernel::ncollection::BaseAllocator::common();
    let mut owner = IndexedMapNode::allocate_with(&arena, -7, std::ptr::null_mut(), || 7);
    let address = std::ptr::from_ref(&*owner);
    *owner.key1() = 8;
    *owner.index() = i32::MAX;
    assert_eq!((owner.value(), owner.index_value()), (&8, i32::MAX));
    let (node, mut allocator) = IndexedMapNode::into_raw(owner);
    assert_eq!(node.cast_const().cast(), address);
    unsafe { IndexedMapNode::<i32>::del_node(node, &mut allocator) };
}
#[test]
fn allocated_indexed_data_node_mutation_preserves_original_address() {
    let arena = rustycad_tkernel::ncollection::BaseAllocator::common();
    let mut owner = IndexedDataMapNode::allocate_with(&arena, -7, std::ptr::null_mut(), || 1, || 7);
    let address = std::ptr::from_ref(&*owner);
    *owner.key1() = 2;
    *owner.change_value() = 8;
    *owner.index() = i32::MIN;
    assert_eq!(
        (owner.key(), owner.value(), owner.index_value()),
        (&2, &8, i32::MIN)
    );
    let (node, mut allocator) = IndexedDataMapNode::into_raw(owner);
    assert_eq!(node.cast_const().cast(), address);
    unsafe { IndexedDataMapNode::<i32, i32>::del_node(node, &mut allocator) };
}
