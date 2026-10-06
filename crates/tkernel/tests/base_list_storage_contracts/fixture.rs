//! A typed owner exercises the type-erased native engine without leaking callback nodes.
use rustycad_tkernel::ncollection::{BaseAllocator, BaseListStorage, ListLinkNode, ListValueNode};
use std::ptr::null_mut;
pub(super) struct OwnedRawList(pub BaseListStorage);
impl OwnedRawList {
    pub(super) fn new(values: &[i32]) -> Self {
        let mut list = Self(BaseListStorage::default());
        for value in values {
            let node = list.make(*value);
            // SAFETY: newly allocated disjoint node, exclusive chain.
            unsafe { list.0.append_node(node) };
        }
        list
    }
    pub(super) fn make(&self, value: i32) -> *mut ListLinkNode {
        let owner = ListValueNode::allocate_with(self.0.allocator(), null_mut(), || value);
        ListValueNode::into_raw(owner).0
    }
    pub(super) fn values(&self) -> Vec<i32> {
        let mut values = Vec::new();
        let mut current = self.0.first();
        while !current.is_null() {
            // SAFETY: this fixture owns an acyclic chain of live ListValueNode<i32> allocations.
            unsafe {
                values.push(*(*current.cast::<ListValueNode<i32>>()).value());
                current = (*current).next();
            }
        }
        values
    }
    pub(super) fn row(&self, label: &str) -> String {
        let values: String = self.values().into_iter().map(|n| format!(" {n}")).collect();
        format!("{label} {}{values}", self.0.size())
    }
}
impl Drop for OwnedRawList {
    fn drop(&mut self) {
        // SAFETY: fixture owns all linked nodes and uses their exact generic type/resource.
        unsafe { self.0.clear(delete_i32) };
    }
}
pub(super) fn delete_i32(node: *mut ListLinkNode, allocator: &mut BaseAllocator) {
    // SAFETY: only the fixture's owned concrete i32 nodes reach this callback, exactly once.
    unsafe { ListValueNode::<i32>::del_node(node, allocator) };
}
pub(super) fn oracle_row(row: &str) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_base_list_storage_contracts.expected");
    assert!(
        expected.lines().any(|line| line == row),
        "native row absent: {row}"
    );
}
