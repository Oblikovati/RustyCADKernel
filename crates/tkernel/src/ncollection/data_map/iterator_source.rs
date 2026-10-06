//! The generic native iterator borrows the actual canonical DataMap bucket table.
use super::DataMap;
use crate::ncollection::{BasicMapIteratorSource, DataMapNode, MapLinkNode};
impl<K, V, H> BasicMapIteratorSource for DataMap<K, V, H> {
    type Node = DataMapNode<K, V>;
    fn basic_bucket_slots(&self) -> Option<&[*mut MapLinkNode]> {
        if self.storage.base.buckets().is_null() {
            return None;
        }
        Some(self.storage.base.bucket_slots())
    }
    fn basic_first_node(&self) -> *mut Self::Node {
        // DataMap is unordered; native ordered traversal is not a DataMap specialization.
        std::ptr::null_mut()
    }
}
