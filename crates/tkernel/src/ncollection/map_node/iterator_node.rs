//! KEY_ONLY BasicIterator reads the real ListValueNode<K> value rather than a unit placeholder.
use super::MapNode;
use crate::ncollection::BasicMapNode;
impl<K> BasicMapNode for MapNode<K> {
    type Key = K;
    type Value = K;
    fn bucket_next(&self) -> *mut Self {
        self.link().next().cast()
    }
    fn order_next(&self) -> *mut Self {
        std::ptr::null_mut()
    }
    fn map_key(&self) -> &K {
        self.key()
    }
    fn map_value(&self) -> &K {
        self.value()
    }
}
// Deliberately omit BasicMapNodeMut: even native mutable Map iterators expose only const keys.
