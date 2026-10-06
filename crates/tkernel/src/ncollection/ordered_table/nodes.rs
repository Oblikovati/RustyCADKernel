//! Lifetime-bound safe views over the actual native nodes and insertion-order links.
use super::*;
/// Shared ordered node view; `map.first_node().unwrap().key()`.
pub struct OrderedNodeRef<'a, K, V> {
    inner: OrderedItems<'a, K, V>,
}
/// Exclusive ordered-data node view; `*map.first_node_mut().unwrap().change_value()=7`.
pub struct OrderedNodeMut<'a, K, V> {
    inner: OrderedItems<'a, K, V>,
    exclusive: PhantomData<&'a mut V>,
}
impl<K, V, H, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn first_node(&self) -> Option<OrderedNodeRef<'_, K, V>> {
        let inner = self.items();
        inner.current?;
        Some(OrderedNodeRef { inner })
    }
}
impl<K, V, H> OrderedTable<K, V, H> {
    pub fn first_node_mut(&mut self) -> Option<OrderedNodeMut<'_, K, V>> {
        let inner = self.items();
        inner.current?;
        Some(OrderedNodeMut {
            inner,
            exclusive: PhantomData,
        })
    }
}
impl<'a, K, V> OrderedNodeRef<'a, K, V> {
    /// Borrows the actual stored key; `node.key()`.
    pub fn key(&self) -> &'a K {
        self.inner.current_pair().unwrap().0
    }
    /// Borrows the item (unit for the retained set-view adaptation); `node.value()`.
    pub fn value(&self) -> &'a V {
        self.inner.current_pair().unwrap().1
    }
    /// Follows the actual insertion-order link; `node.order_next()`.
    pub fn order_next(&self) -> Option<Self> {
        let mut inner = self.inner.clone();
        inner.advance();
        inner.current?;
        Some(Self { inner })
    }
}
impl<K, V> OrderedNodeMut<'_, K, V> {
    /// Borrows the immutable key; `node.key()`.
    pub fn key(&self) -> &K {
        // SAFETY: this view keeps the node live and projects only its key.
        unsafe { &*(self.inner.key)(self.inner.current.unwrap()) }
    }
    /// Borrows the current value; `node.value()`.
    pub fn value(&self) -> &V {
        // SAFETY: the view's shared borrow excludes its exclusive accessor.
        unsafe { &*(self.inner.value)(self.inner.current.unwrap()) }
    }
    /// Edits only the current value; `*node.change_value()=7`.
    pub fn change_value(&mut self) -> &mut V {
        // SAFETY: only data maps create exclusive views; the exclusive view borrow prevents aliasing.
        unsafe { &mut *(self.inner.value)(self.inner.current.unwrap()).cast_mut() }
    }
    /// Consumes this view and follows the native order link; `node.order_next()`.
    pub fn order_next(mut self) -> Option<Self> {
        self.inner.advance();
        self.inner.current?;
        Some(self)
    }
}
// SAFETY: exclusive values transfer with the unique view, while keys stay shared.
unsafe impl<K: Sync, V: Send> Send for OrderedNodeMut<'_, K, V> {}
// SAFETY: shared view access never permits value mutation.
unsafe impl<K: Sync, V: Sync> Sync for OrderedNodeMut<'_, K, V> {}
