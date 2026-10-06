//! Real key-only MapNode and canonical DataMap generic iterator adapter contracts.
#![allow(unsafe_code)]
#[path = "map_node_bridge_contracts/fixture.rs"]
mod fixture;
#[path = "map_node_bridge_contracts/iterator.rs"]
mod iterator;
#[path = "map_node_bridge_contracts/lifetime.rs"]
mod lifetime;
use fixture::{KeyTrace, native_row};
use rustycad_tkernel::ncollection::{ListValueNode, MapLinkNode, MapNode};
#[test]
fn borrowed_constructor_copies_then_drops_one_inherited_key() {
    let trace = KeyTrace::new();
    let key = trace.key();
    drop(MapNode::from_ref(&key, std::ptr::null_mut()));
    native_row("borrowed", format!("borrowed {}", trace.events()));
}
#[test]
fn owned_constructor_transfers_one_inherited_key() {
    let trace = KeyTrace::new();
    drop(MapNode::new(trace.key(), std::ptr::null_mut()));
    native_row("owned", format!("owned {}", trace.events()));
}
#[test]
fn in_place_constructor_constructs_the_native_value() {
    let trace = KeyTrace::new();
    drop(MapNode::construct(std::ptr::null_mut(), || trace.key()));
    native_row("in_place", format!("in_place {}", trace.events()));
}
#[test]
fn node_has_real_list_value_prefix_and_shared_key_value_identity() {
    let mut tail = MapLinkNode::new(std::ptr::null_mut());
    let mut node = MapNode::new(7, std::ptr::from_mut(&mut tail));
    let base = std::ptr::from_ref(&node).cast::<ListValueNode<i32>>();
    assert_eq!(
        std::ptr::from_ref(node.link()).cast::<()>(),
        std::ptr::from_ref(&node).cast::<()>()
    );
    let same = std::ptr::from_ref(node.key()) == std::ptr::from_ref(node.value());
    assert_eq!(unsafe { (*base).value() }, &7);
    assert_eq!(node.link().next(), std::ptr::from_mut(&mut tail));
    *node.change_value() = 9;
    *node.link_mut().next_mut() = std::ptr::null_mut();
    assert_prefix_row(&node, same);
}
fn assert_prefix_row(node: &MapNode<i32>, same: bool) {
    native_row(
        "prefix",
        format!(
            "prefix 1 1 {} {} {} {}",
            u8::from(same),
            u8::from(same),
            node.key(),
            u8::from(node.link().next().is_null())
        ),
    );
}
#[test]
fn allocation_precedes_key_copy_and_raw_deletion_releases_node() {
    let trace = KeyTrace::new();
    let key = trace.key();
    let owner = MapNode::allocate_ref(&trace.arena, &key, std::ptr::null_mut());
    assert_allocation_row(&trace);
    trace.events.borrow_mut().clear();
    let (pointer, mut arena) = MapNode::into_raw(owner);
    unsafe { MapNode::<fixture::TracedKey>::del_node(pointer, &mut arena) };
    native_row(
        "delete",
        format!("delete {} {}", trace.events(), u8::from(trace.live())),
    );
}
fn assert_allocation_row(trace: &KeyTrace) {
    native_row(
        "allocate",
        format!(
            "allocate {} {} {}",
            trace.events(),
            u8::from(trace.allocated.get()),
            u8::from(trace.live())
        ),
    );
}
