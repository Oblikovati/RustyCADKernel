//! Borrowed native order traversal; field projections never reborrow an entire node.
use super::*;
pub(in crate::ncollection) struct OrderedItems<'a, K, V> {
    pub current: Option<*mut MapLinkNode>,
    first: Option<*mut MapLinkNode>,
    pub(super) key: unsafe fn(*mut MapLinkNode) -> *const K,
    pub(super) value: unsafe fn(*mut MapLinkNode) -> *const V,
    next: unsafe fn(*mut MapLinkNode) -> *mut *mut MapLinkNode,
    borrow: PhantomData<(&'a K, &'a V)>,
}
impl<K, V, H, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn items(&self) -> OrderedItems<'_, K, V> {
        let first = self.first.map(|slot| self.node_pointer(slot));
        OrderedItems {
            current: first,
            first,
            key: project_key::<K, V, N>,
            value: project_value::<K, V, N>,
            next: N::next,
            borrow: PhantomData,
        }
    }
}
impl<'a, K, V> OrderedItems<'a, K, V> {
    pub fn current_pair(&self) -> Option<(&'a K, &'a V)> {
        let node = self.current?;
        // SAFETY: the map borrow keeps the current node live; the callbacks project only fields.
        Some(unsafe { (&*(self.key)(node), &*(self.value)(node)) })
    }
    pub fn restart(&mut self) {
        self.current = self.first;
    }
    pub fn advance(&mut self) {
        self.current = self.current.and_then(|node| {
            // SAFETY: current belongs to the borrowed map; reading this link never borrows payloads.
            let next = unsafe { *(self.next)(node) };
            (!next.is_null()).then_some(next)
        });
    }
}
impl<'a, K, V> Iterator for OrderedItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        let pair = self.current_pair()?;
        self.advance();
        Some(pair)
    }
}
impl<K, V> Clone for OrderedItems<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            current: self.current,
            first: self.first,
            key: self.key,
            value: self.value,
            next: self.next,
            borrow: PhantomData,
        }
    }
}
impl<K, V> Default for OrderedItems<'_, K, V> {
    fn default() -> Self {
        Self {
            current: None,
            first: None,
            key: project_key::<K, V, OrderedDataMapNode<K, V>>,
            value: project_value::<K, V, OrderedDataMapNode<K, V>>,
            next: <OrderedDataMapNode<K, V> as StorageNode<K, V>>::next,
            borrow: PhantomData,
        }
    }
}
impl<K, V> PartialEq for OrderedItems<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.current == other.current
    }
}
impl<K, V> Eq for OrderedItems<'_, K, V> {}
// SAFETY: shared map lifetime prevents mutation; payload synchronization controls cross-thread reads.
unsafe impl<K: Sync, V: Sync> Send for OrderedItems<'_, K, V> {}
// SAFETY: only shared payload references and immutable links are observable.
unsafe impl<K: Sync, V: Sync> Sync for OrderedItems<'_, K, V> {}

unsafe fn project_key<K, V, N: StorageNode<K, V>>(node: *mut MapLinkNode) -> *const K {
    // SAFETY: caller supplies the live concrete node selected by this callback.
    unsafe { N::key(node).cast_const() }
}
unsafe fn project_value<K, V, N: StorageNode<K, V>>(node: *mut MapLinkNode) -> *const V {
    // SAFETY: caller supplies the live concrete node selected by this callback.
    unsafe { N::value(node).cast_const() }
}
