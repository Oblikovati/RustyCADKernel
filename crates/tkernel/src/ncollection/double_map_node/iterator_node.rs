//! The existing BaseMap generic iterator reads this node's actual inherited links and objects.
use super::DoubleMapNode;
use crate::ncollection::BasicMapNode;
impl<K, V> BasicMapNode for DoubleMapNode<K, V> {
    type Key = K;
    type Value = V;
    fn bucket_next(&self) -> *mut Self {
        self.link().next().cast()
    }
    fn order_next(&self) -> *mut Self {
        // DoubleMap is unordered; only the bucket chain participates in native iteration.
        std::ptr::null_mut()
    }
    fn map_key(&self) -> &K {
        self.key1()
    }
    fn map_value(&self) -> &V {
        self.value()
    }
}
// Native DoubleMap iterators expose only const Key2 even for a mutable container.
