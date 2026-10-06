//! The facade exposes the actual native two-table graph to existing generic readonly iterators.
use super::{DoubleMap, DoubleMapNode, DoubleMapObserver, MapLinkNode};
use crate::ncollection::BasicMapIteratorSource;
impl<A, B, H1, H2> DoubleMap<A, B, H1, H2> {
    /// Returns the native primary table identity; `map.buckets()`.
    pub fn buckets(&self) -> *const *mut MapLinkNode {
        self.storage.base().buckets()
    }
    /// Returns the native secondary table identity; `map.second_buckets()`.
    pub fn second_buckets(&self) -> *const *mut MapLinkNode {
        self.storage.base().second_buckets()
    }
    /// Captures an independent metadata observer; `assert_eq!(map.observer().snapshot().size,0)`.
    pub fn observer(&self) -> DoubleMapObserver {
        self.storage.observer()
    }
}
impl<A, B, H1, H2> BasicMapIteratorSource for DoubleMap<A, B, H1, H2> {
    type Node = DoubleMapNode<A, B>;
    fn basic_bucket_slots(&self) -> Option<&[*mut MapLinkNode]> {
        (!self.storage.base().buckets().is_null()).then(|| self.storage.base().bucket_slots())
    }
    fn basic_first_node(&self) -> *mut Self::Node {
        std::ptr::null_mut()
    }
}
