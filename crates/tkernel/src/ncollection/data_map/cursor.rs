//! `NCollection_DataMap::Iterator`: More/Next/Value over the actual native bucket topology.
use super::super::base_map::exhausted_iterator;
use super::{DataMap, DataMapItems, Debug, HashPolicy, storage::DataMapStorage};
use crate::{
    ncollection::{DataMapNode, owned_bucket_chains::BucketChainPosition},
    standard::Failure,
};
/// Read-only OCCT cursor; `let mut cursor=map.cursor(); while cursor.more() { cursor.next(); }`.
pub struct DataMapIterator<'a, K, V> {
    items: DataMapItems<'a, K, V>,
}
/// Exclusive OCCT cursor; `*map.cursor_mut().change_value()?=7`.
pub struct DataMapIteratorMut<'a, K, V> {
    source: Option<&'a mut DataMapStorage<K, V>>,
    position: BucketChainPosition,
}
// SAFETY: the cursor retains the exclusive graph borrow; each value follows K,V ownership.
unsafe impl<K: Send, V: Send> Send for DataMapIteratorMut<'_, K, V> {}
unsafe impl<K: Sync, V: Sync> Sync for DataMapIteratorMut<'_, K, V> {}
impl<K: Debug, V, H: HashPolicy<K>> DataMap<K, V, H> {
    /// `Iterator(const NCollection_DataMap&)`: a cursor on the first binding in bucket order.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DataMap;
    /// let mut map = DataMap::<i32, i32>::default();
    /// map.bind(1, 10);
    /// let it = map.cursor();
    /// assert_eq!((it.key().unwrap(), it.value().unwrap()), (&1, &10));
    /// ```
    pub fn cursor(&self) -> DataMapIterator<'_, K, V> {
        DataMapIterator {
            items: self.items(),
        }
    }
    /// `Iterator(NCollection_DataMap&)`: a cursor that may also change items.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DataMap;
    /// let mut map = DataMap::<i32, i32>::default();
    /// map.bind(1, 10);
    /// *map.cursor_mut().change_value().unwrap() = 5;
    /// assert_eq!(map.seek(&1), Some(&5));
    /// ```
    pub fn cursor_mut(&mut self) -> DataMapIteratorMut<'_, K, V> {
        DataMapIteratorMut {
            position: BucketChainPosition::new(&self.storage.base),
            source: Some(&mut self.storage),
        }
    }
}
impl<'a, K, V> DataMapIterator<'a, K, V> {
    /// Reports whether the native cursor has an item; `cursor.more()`.
    pub fn more(&self) -> bool {
        self.items.current().is_some()
    }
    /// Advances in bucket/chain order; `cursor.next()`.
    pub fn next(&mut self) {
        self.items.next();
    }
    /// Borrows native Value or reports exhaustion; `cursor.value()?`.
    pub fn value(&self) -> Result<&'a V, Failure> {
        Ok(self.entry("NCollection_DataMap::Iterator::Value")?.value())
    }
    /// Borrows native Key or reports exhaustion; `cursor.key()?`.
    pub fn key(&self) -> Result<&'a K, Failure> {
        Ok(self.entry("NCollection_DataMap::Iterator::Key")?.key())
    }
    /// Rebinds the native const-map initialization overload; `cursor.initialize(&map)`.
    pub fn initialize<H: HashPolicy<K>>(&mut self, map: &'a DataMap<K, V, H>)
    where
        K: Debug,
    {
        *self = map.cursor();
    }
    /// Restarts on the bound graph (default remains empty); `cursor.reset()`.
    pub fn reset(&mut self) {
        self.items.reset();
    }
    /// Compares native node identity, including exhausted cursors; `cursor.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.items == other.items
    }
    fn entry(&self, method: &str) -> Result<&'a DataMapNode<K, V>, Failure> {
        self.items
            .current()
            .ok_or_else(|| exhausted_iterator(method))
    }
}
impl<K, V> Default for DataMapIterator<'_, K, V> {
    fn default() -> Self {
        Self {
            items: DataMapItems::default(),
        }
    }
}
impl<K, V> Clone for DataMapIterator<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
        }
    }
}
impl<K, V> DataMapIterator<'_, K, V> {
    /// Advances and returns this cursor, as native prefix increment; `cursor.advance().value()`.
    pub fn advance(&mut self) -> &mut Self {
        self.next();
        self
    }
    /// Returns the previous cursor and advances, as native postfix increment; `cursor.advance_post().value()`.
    pub fn advance_post(&mut self) -> Self {
        let previous = self.clone();
        self.next();
        previous
    }
}
impl<'a, K, V> DataMapIteratorMut<'a, K, V> {
    /// Reports native More; `cursor.more()`.
    pub fn more(&self) -> bool {
        !self.position.node.is_null()
    }
    /// Advances native Next; `cursor.next()`.
    pub fn next(&mut self) {
        if let Some(source) = self.source.as_ref() {
            self.position.advance(&source.base);
        }
    }
    /// Borrows native Value without permitting changes; `cursor.value()?`.
    pub fn value(&self) -> Result<&V, Failure> {
        Ok(self.entry("NCollection_DataMap::Iterator::Value")?.value())
    }
    /// Borrows native ChangeValue exclusively; `*cursor.change_value()?=8`.
    pub fn change_value(&mut self) -> Result<&mut V, Failure> {
        if !self.more() {
            return Err(exhausted_iterator(
                "NCollection_DataMap::Iterator::ChangeValue",
            ));
        }
        let source = self.source.as_mut().unwrap();
        Ok(unsafe { source.node_mut(self.position.node).change_value() })
    }
    /// Borrows native Key; `cursor.key()?`.
    pub fn key(&self) -> Result<&K, Failure> {
        Ok(self.entry("NCollection_DataMap::Iterator::Key")?.key())
    }
    /// Rebinds native mutable-map Initialize; `cursor.initialize(&mut map)`.
    pub fn initialize<H: HashPolicy<K>>(&mut self, map: &'a mut DataMap<K, V, H>)
    where
        K: Debug,
    {
        *self = map.cursor_mut();
    }
    /// Restarts on this map's first node; `cursor.reset()`.
    pub fn reset(&mut self) {
        self.position = self
            .source
            .as_ref()
            .map_or_else(BucketChainPosition::default, |source| {
                BucketChainPosition::new(&source.base)
            });
    }
    fn entry(&self, method: &str) -> Result<&DataMapNode<K, V>, Failure> {
        if !self.more() {
            return Err(exhausted_iterator(method));
        }
        Ok(unsafe { self.source.as_ref().unwrap().node(self.position.node) })
    }
}
impl<K, V> Default for DataMapIteratorMut<'_, K, V> {
    fn default() -> Self {
        Self {
            source: None,
            position: BucketChainPosition::default(),
        }
    }
}
impl<K, V> DataMapIteratorMut<'_, K, V> {
    /// Compares native node identity, with all exhausted cursors equal; `first.is_equal(&second)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.position.node == other.position.node
    }
    /// Advances and returns the writable cursor; `cursor.advance().change_value()`.
    pub fn advance(&mut self) -> &mut Self {
        self.next();
        self
    }
    /// Returns a read-only previous position while advancing; `cursor.advance_post().value()`.
    /// The snapshot borrows this cursor to prevent aliased writable item references.
    pub fn advance_post(&mut self) -> DataMapIterator<'_, K, V> {
        let position = self.position;
        self.next();
        DataMapIterator {
            items: DataMapItems::from_position(self.source.as_deref(), position),
        }
    }
}
