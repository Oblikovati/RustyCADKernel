//! Exclusive data-map traversal projects disjoint fields without snapshot allocations.
use super::*;
pub(in crate::ncollection) struct OrderedItemsMut<'a, K, V> {
    inner: OrderedItems<'a, K, V>,
    exclusive: PhantomData<&'a mut V>,
}
impl<K, V, H> OrderedTable<K, V, H> {
    pub fn items_mut(&mut self) -> OrderedItemsMut<'_, K, V> {
        OrderedItemsMut {
            inner: self.items(),
            exclusive: PhantomData,
        }
    }
}
impl<'a, K, V> Iterator for OrderedItemsMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.inner.current?;
        self.inner.advance();
        // SAFETY: exclusive map borrow, distinct data-node fields, and acyclic links ensure
        // each value is yielded once. No full-node reference overlaps previously yielded values.
        Some(unsafe {
            (
                &*(self.inner.key)(node),
                &mut *(self.inner.value)(node).cast_mut(),
            )
        })
    }
}
// SAFETY: keys remain shared and each exclusive value can transfer once with the iterator.
unsafe impl<K: Sync, V: Send> Send for OrderedItemsMut<'_, K, V> {}
// SAFETY: shared access to the iterator cannot advance or expose exclusive values.
unsafe impl<K: Sync, V: Sync> Sync for OrderedItemsMut<'_, K, V> {}
