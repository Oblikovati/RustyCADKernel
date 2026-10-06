//! The existing BaseMap generic iterator reads this node's actual inherited links and objects.
use super::DataMapNode;
use crate::ncollection::{BasicMapNode, BasicMapNodeMut};
impl<K, V> BasicMapNode for DataMapNode<K, V> {
    type Key = K;
    type Value = V;
    fn bucket_next(&self) -> *mut Self {
        self.link().next().cast()
    }
    fn order_next(&self) -> *mut Self {
        // DataMap is unordered; only the bucket chain participates in native iteration.
        std::ptr::null_mut()
    }
    fn map_key(&self) -> &K {
        self.key()
    }
    fn map_value(&self) -> &V {
        self.value()
    }
}
impl<K, V> BasicMapNodeMut for DataMapNode<K, V> {
    fn map_value_mut(&mut self) -> &mut V {
        self.change_value()
    }
}
