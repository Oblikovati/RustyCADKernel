//! A separate value guard reproduces base-subobject cleanup even when the key panics in Drop.
use super::{DataMapNode, ListValueNode, ManuallyDrop};
struct DataMapValueCleanup<'a, V>(&'a mut ManuallyDrop<ListValueNode<V>>);
impl<V> Drop for DataMapValueCleanup<'_, V> {
    fn drop(&mut self) {
        // SAFETY: the guard exclusively owns cleanup of the still-initialized base value node.
        unsafe { ManuallyDrop::drop(self.0) };
    }
}
impl<K, V> Drop for DataMapNode<K, V> {
    fn drop(&mut self) {
        let _value_cleanup = DataMapValueCleanup(&mut self.value_node);
        // SAFETY: only this Drop owns key destruction; the value guard always runs afterwards.
        unsafe { ManuallyDrop::drop(&mut self.key) };
    }
}
