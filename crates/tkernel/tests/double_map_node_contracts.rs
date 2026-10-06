//! Public DoubleMapNode constructor and real prefix contracts against pinned OCCT.
#![allow(unsafe_code)]
#[path = "double_map_node_contracts/engine.rs"]
mod engine;
#[path = "double_map_node_contracts/fixture.rs"]
mod fixture;
#[path = "double_map_node_contracts/layout.rs"]
mod layout;
#[path = "double_map_node_contracts/lifetime.rs"]
mod lifetime;
use fixture::{NodeTrace, native_node_row};
use rustycad_tkernel::ncollection::{DoubleMapNode, MapLinkNode};
#[test]
fn native_borrowed_node_constructs_value_then_key_and_drops_key_then_value() {
    let trace = NodeTrace::new();
    let key = trace.item('K');
    let value = trace.item('V');
    drop(DoubleMapNode::from_refs(
        &key,
        &value,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    ));
    native_node_row("borrowed", format!("borrowed {}", trace.events()));
}
#[test]
fn native_key_borrowed_node_copies_only_key() {
    let trace = NodeTrace::new();
    let key = trace.item('K');
    drop(DoubleMapNode::from_key1_ref(
        &key,
        trace.item('V'),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    ));
    native_node_row("key_borrowed", format!("key_borrowed {}", trace.events()));
}
#[test]
fn native_value_borrowed_node_copies_only_value() {
    let trace = NodeTrace::new();
    let value = trace.item('V');
    drop(DoubleMapNode::from_key2_ref(
        trace.item('K'),
        &value,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    ));
    native_node_row(
        "value_borrowed",
        format!("value_borrowed {}", trace.events()),
    );
}
#[test]
fn native_owned_node_performs_no_copies() {
    let trace = NodeTrace::new();
    drop(DoubleMapNode::new(
        trace.item('K'),
        trace.item('V'),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    ));
    native_node_row("owned", format!("owned {}", trace.events()));
}
#[test]
fn native_emplacement_borrowed_key() {
    let trace = NodeTrace::new();
    let key = trace.item('K');
    drop(DoubleMapNode::construct_ref(
        &key,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        || trace.item('V'),
    ));
    native_node_row(
        "emplace_borrowed",
        format!("emplace_borrowed {}", trace.events()),
    );
}
#[test]
fn native_emplacement_owned_key() {
    let trace = NodeTrace::new();
    drop(DoubleMapNode::construct(
        trace.item('K'),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        || trace.item('V'),
    ));
    native_node_row("emplace_owned", format!("emplace_owned {}", trace.events()));
}
#[test]
fn native_real_value_and_link_prefix() {
    let mut tail = MapLinkNode::new(std::ptr::null_mut());
    let next = std::ptr::from_mut(&mut tail);
    let mut node = DoubleMapNode::new(1, 7, next, next);
    let prefix = std::ptr::from_ref(node.link());
    let same_prefix = prefix.cast::<()>() == std::ptr::from_ref(&node).cast();
    let mut row = prefix_start(&node, next, same_prefix);
    *node.change_value() = 8;
    *node.link_mut().next_mut() = std::ptr::null_mut();
    row.push_str(&format!(
        " {} {}",
        node.value(),
        u8::from(node.link().next().is_null())
    ));
    native_node_row("prefix", row);
}

fn prefix_start(
    node: &DoubleMapNode<i32, i32>,
    next: *mut MapLinkNode,
    same_prefix: bool,
) -> String {
    format!(
        "prefix {} {} {} {}",
        u8::from(same_prefix),
        u8::from(node.link().next() == next),
        node.key1(),
        node.value()
    )
}
