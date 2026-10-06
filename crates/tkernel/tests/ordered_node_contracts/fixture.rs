#[path = "../fixtures/native_map_node_trace.rs"]
mod trace;
use rustycad_tkernel::ncollection::{OrderedDataMapNode, OrderedDataMapNodeOwner};
pub use trace::{NodeItem, NodeTrace, node_caught};
pub fn borrowed_node(trace: &NodeTrace) -> OrderedDataMapNodeOwner<NodeItem, NodeItem> {
    let key = trace.item('K');
    let value = trace.item('V');
    OrderedDataMapNode::allocate_ref(&trace.arena, &key, &value, std::ptr::null_mut())
}
pub fn native_node_row(prefix: &str, actual: String) {
    let expected = include_str!("../../../../tools/oracle/tkernel_ordered_node_contracts.expected");
    let row = expected
        .lines()
        .find(|row| row.starts_with(prefix))
        .unwrap();
    assert_eq!(actual, row);
}
