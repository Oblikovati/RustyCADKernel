#[path = "../fixtures/native_map_node_trace.rs"]
mod trace;
use rustycad_tkernel::ncollection::{DoubleMapNode, DoubleMapNodeOwner};
pub use trace::{NodeItem, NodeTrace, node_caught};
pub fn borrowed_node(trace: &NodeTrace) -> DoubleMapNodeOwner<NodeItem, NodeItem> {
    let key = trace.item('K');
    let value = trace.item('V');
    DoubleMapNode::allocate_ref(
        &trace.arena,
        &key,
        &value,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    )
}
pub fn native_node_row(prefix: &str, actual: String) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_double_map_node_contracts.expected");
    let row = expected
        .lines()
        .find(|row| row.starts_with(prefix))
        .unwrap();
    assert_eq!(actual, row);
}
