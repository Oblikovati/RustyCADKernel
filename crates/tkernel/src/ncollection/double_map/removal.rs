//! Native first unlink is observable during the opposite hash; unwind restores it before escaping.
use super::{Debug, DoubleMap, HashPolicy, MapLinkNode};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
impl<A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> DoubleMap<A, B, H1, H2> {
    pub(super) fn remove_native_pair(
        &mut self,
        bucket: usize,
        node: *mut MapLinkNode,
        secondary: bool,
    ) -> bool {
        let token = self
            .storage
            .unlink(bucket, node, secondary)
            .expect("matched DoubleMap node must remain linked");
        self.unlink_partner(token, node, secondary);
        self.storage.destroy_unlinked(node);
        true
    }
    fn unlink_partner(
        &mut self,
        token: super::storage::UnlinkedDoubleNode,
        node: *mut MapLinkNode,
        secondary: bool,
    ) {
        let opposite = catch_unwind(AssertUnwindSafe(|| self.opposite_bucket(node, secondary)));
        let opposite = match opposite {
            Ok(value) => value,
            Err(panic) => {
                self.storage.restore(token);
                resume_unwind(panic)
            }
        };
        self.require_partner(token, node, secondary, opposite);
    }
    fn require_partner(
        &mut self,
        token: super::storage::UnlinkedDoubleNode,
        node: *mut MapLinkNode,
        secondary: bool,
        opposite: usize,
    ) {
        if self.storage.unlink(opposite, node, !secondary).is_none() {
            self.storage.restore(token);
            panic!(
                "DoubleMap node {node:p} missing opposite bucket {opposite}; expected both indices to contain the same node"
            );
        }
    }
    fn opposite_bucket(&self, node: *mut MapLinkNode, secondary: bool) -> usize {
        let node = unsafe { self.storage.node(node) };
        if secondary {
            self.first_policy.hash(node.key1()) % self.bucket_count() + 1
        } else {
            self.second_policy.hash(node.key2()) % self.bucket_count() + 1
        }
    }
}
