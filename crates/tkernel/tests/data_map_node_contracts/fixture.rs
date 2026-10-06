//! DataMap constructor tests share the same named payload/allocation observer as other native nodes.
#[path = "../fixtures/native_map_node_trace.rs"]
mod trace;
use rustycad_tkernel::ncollection::DataMapNode;
pub use trace::{NodeItem, NodeTrace, node_caught};
impl NodeTrace {
    pub fn borrowed_node(
        &self,
    ) -> rustycad_tkernel::ncollection::DataMapNodeOwner<NodeItem, NodeItem> {
        let key = self.item('K');
        let value = self.item('V');
        // Source cleanup follows this helper; callers reset tracing before observing node Drop.
        DataMapNode::allocate_ref(&self.arena, &key, &value, std::ptr::null_mut())
    }
}
pub fn native_node_row(prefix: &str, actual: String) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_data_map_node_contracts.expected");
    let row = expected
        .lines()
        .find(|row| row.starts_with(prefix))
        .unwrap();
    assert_eq!(actual, row);
}
