//! Flat native payloads retain key-before-value cleanup, including a panicking key destructor.
use super::DoubleMapNode;
use crate::ncollection::map_payload::drop_key_then_value;
impl<K, V> Drop for DoubleMapNode<K, V> {
    fn drop(&mut self) {
        // SAFETY: both distinct fields remain initialized until their single owned cleanup.
        unsafe { drop_key_then_value(&mut self.key, &mut self.value) };
    }
}
