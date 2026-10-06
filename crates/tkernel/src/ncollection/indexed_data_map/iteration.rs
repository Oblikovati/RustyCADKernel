use super::{
    Debug, HashPolicy, IndexedDataItems, IndexedDataMap, IndexedDataMapNode, IndexedDataValues,
    IndexedDataValuesMut, IndexedNode, MapLinkNode,
};
impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Borrows values in dense index order; `map.iter().min()`.
    pub fn iter(&self) -> IndexedDataValues<'_, V> {
        IndexedDataValues {
            nodes: self.storage.indices(),
            project: value_shared::<K, V>,
            payload: std::marker::PhantomData,
            position: 0,
        }
    }
    /// Returns the exhausted value iterator; `map.iter()!=map.end()`.
    pub fn end(&self) -> IndexedDataValues<'_, V> {
        IndexedDataValues {
            nodes: self.storage.indices(),
            project: value_shared::<K, V>,
            payload: std::marker::PhantomData,
            position: self.len(),
        }
    }
    /// Borrows all values exclusively; `map.iter_mut().for_each(|value|*value+=1)`.
    pub fn iter_mut(&mut self) -> IndexedDataValuesMut<'_, V> {
        IndexedDataValuesMut {
            nodes: self.storage.indices().iter(),
            project: value_mut::<K, V>,
            payload: std::marker::PhantomData,
        }
    }
    /// Borrows key/value pairs in index order; `for (key,value) in map.items() {}`.
    pub fn items(&self) -> IndexedDataItems<'_, K, V> {
        IndexedDataItems {
            keys: self.key_iter(0),
            values: self.iter(),
        }
    }
    /// Returns the exhausted pair iterator; `map.items()!=map.items_end()`.
    pub fn items_end(&self) -> IndexedDataItems<'_, K, V> {
        IndexedDataItems {
            keys: self.key_iter(self.len()),
            values: self.end(),
        }
    }
    /// Borrows values exclusively while keeping keys immutable; `for (_,value) in map.items_mut() {}`.
    pub fn items_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        IndexedPairsMut {
            nodes: self.storage.indices().iter(),
            payload: std::marker::PhantomData,
        }
    }
    /// Borrows keys and values paired with one-based indices; `for (key,value,index) in map.indexed_items() {}`.
    pub fn indexed_items(&self) -> impl Iterator<Item = (&K, &V, usize)> {
        self.items()
            .enumerate()
            .map(|(index, (key, value))| (key, value, index + 1))
    }
}
impl<'a, V> Iterator for IndexedDataValues<'a, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.nodes.get(self.position)?;
        self.position += 1;
        // SAFETY: the view keeps its owning map borrowed; projection touches only V.
        Some(unsafe { &*(self.project)(*node) })
    }
}
impl<V> Clone for IndexedDataValues<'_, V> {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes,
            project: self.project,
            payload: std::marker::PhantomData,
            position: self.position,
        }
    }
}
impl<V> PartialEq for IndexedDataValues<'_, V> {
    fn eq(&self, other: &Self) -> bool {
        (self.position >= self.nodes.len() && other.position >= other.nodes.len())
            || (self.nodes.as_ptr() == other.nodes.as_ptr() && self.position == other.position)
    }
}
impl<V> Eq for IndexedDataValues<'_, V> {}
impl<'a, V> Iterator for IndexedDataValuesMut<'a, V> {
    type Item = &'a mut V;
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: exclusive source borrow and unique native index slots yield each V once.
        self.nodes
            .next()
            .map(|&node| unsafe { &mut *(self.project)(node) })
    }
}
impl<'a, K, V> Iterator for IndexedDataItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next().zip(self.values.next())
    }
}
impl<K, V> Clone for IndexedDataItems<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            values: self.values.clone(),
        }
    }
}
impl<K, V> PartialEq for IndexedDataItems<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys && self.values == other.values
    }
}
impl<K, V> Eq for IndexedDataItems<'_, K, V> {}
impl<'a, K: Debug, V, H: HashPolicy<K>> IntoIterator for &'a IndexedDataMap<K, V, H> {
    type Item = &'a V;
    type IntoIter = IndexedDataValues<'a, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, K: Debug, V, H: HashPolicy<K>> IntoIterator for &'a mut IndexedDataMap<K, V, H> {
    type Item = &'a mut V;
    type IntoIter = IndexedDataValuesMut<'a, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    fn key_iter(&self, position: usize) -> IndexedDataValues<'_, K> {
        IndexedDataValues {
            nodes: self.storage.indices(),
            position,
            project: key_shared::<K, V>,
            payload: std::marker::PhantomData,
        }
    }
}
unsafe fn key_shared<K, V>(node: *mut MapLinkNode) -> *const K {
    unsafe { IndexedDataMapNode::<K, V>::key_ptr(node.cast()).cast_const() }
}
unsafe fn value_shared<K, V>(node: *mut MapLinkNode) -> *const V {
    unsafe { IndexedDataMapNode::<K, V>::value_ptr(node.cast()).cast_const() }
}
unsafe fn value_mut<K, V>(node: *mut MapLinkNode) -> *mut V {
    unsafe { IndexedDataMapNode::<K, V>::value_ptr(node.cast()) }
}
// Caller retains an exclusive owning map borrow and projects every index only once.
// Field-only pointers avoid retagging a whole node while an earlier &mut V remains alive.
pub(super) unsafe fn pair_mut<'a, K, V>(node: *mut MapLinkNode) -> (&'a K, &'a mut V) {
    unsafe { (&*key_shared::<K, V>(node), &mut *value_mut::<K, V>(node)) }
}
// SAFETY: erased projections read only V; the owning map remains borrowed and structurally fixed.
unsafe impl<V: Sync> Send for IndexedDataValues<'_, V> {}
unsafe impl<V: Sync> Sync for IndexedDataValues<'_, V> {}
// SAFETY: exclusive view owns disjoint V borrows; K is never accessed through this erased view.
unsafe impl<V: Send> Send for IndexedDataValuesMut<'_, V> {}
unsafe impl<V: Sync> Sync for IndexedDataValuesMut<'_, V> {}

struct IndexedPairsMut<'a, K, V> {
    nodes: std::slice::Iter<'a, *mut MapLinkNode>,
    payload: std::marker::PhantomData<(&'a K, &'a mut V)>,
}
impl<'a, K, V> Iterator for IndexedPairsMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: source exclusive borrow and each unique index yield disjoint item references.
        self.nodes
            .next()
            .map(|&node| unsafe { pair_mut::<K, V>(node) })
    }
}
// SAFETY: keys remain shared and items exclusive; each field projection avoids the other payload.
unsafe impl<K: Sync, V: Send> Send for IndexedPairsMut<'_, K, V> {}
unsafe impl<K: Sync, V: Sync> Sync for IndexedPairsMut<'_, K, V> {}
