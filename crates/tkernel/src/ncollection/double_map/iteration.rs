//! Shared ranges follow the first-key chains in native bucket order.
use super::{Debug, DoubleMap, DoubleMapItems, DoubleMapValues, HashPolicy};
use crate::ncollection::{
    HashBucketTable,
    slot_chains::{ChainSlot, SlotRange},
};
impl<A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> DoubleMap<A, B, H1, H2> {
    /// Borrows both keys in first-key bucket order; `for (first,second) in map.items() {}`.
    pub fn items(&self) -> DoubleMapItems<'_, A, B> {
        DoubleMapItems {
            entries: SlotRange::new(self.storage.view()),
        }
    }
    /// Borrows second keys in first-key bucket order; `map.iter().count()`.
    pub fn iter(&self) -> DoubleMapValues<'_, A, B> {
        DoubleMapValues {
            pairs: self.items(),
        }
    }
    /// Returns an exhausted value iterator; `map.iter()!=map.end()`.
    pub fn end(&self) -> DoubleMapValues<'_, A, B> {
        DoubleMapValues {
            pairs: DoubleMapItems::default(),
        }
    }
    /// Native `Buckets()`: the first (Key1) table, yielding `(key1, key2)` pairs per chain, or
    /// `None` where native returns null. The second table is a protected member (plan D1 B6).
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DoubleMap;
    /// let mut map = DoubleMap::<i32, i32>::default();
    /// assert!(map.buckets().is_none());
    /// map.bind(1, 5).unwrap();
    /// let table = map.buckets().unwrap();
    /// assert_eq!(table.len(), 102);
    /// assert_eq!(table.chain(2)?.collect::<Vec<_>>(), [(&1, &5)]);
    /// # Ok::<(), rustycad_tkernel::standard::Failure>(())
    /// ```
    pub fn buckets(&self) -> Option<HashBucketTable<'_, A, B>> {
        self.storage.buckets()
    }
}

impl<'a, A, B> DoubleMapItems<'a, A, B> {
    pub(super) fn current(&self) -> Option<&'a ChainSlot<A, B>> {
        self.entries.current()
    }
    pub(super) fn reset(&mut self) {
        self.entries.reset();
    }
}
impl<'a, A, B> Iterator for DoubleMapItems<'a, A, B> {
    type Item = (&'a A, &'a B);
    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.entries.next()?;
        Some((entry.key(), entry.value()))
    }
}
impl<A, B> Clone for DoubleMapItems<'_, A, B> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}
impl<A, B> Default for DoubleMapItems<'_, A, B> {
    fn default() -> Self {
        Self {
            entries: SlotRange::default(),
        }
    }
}
impl<A, B> PartialEq for DoubleMapItems<'_, A, B> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}
impl<A, B> Eq for DoubleMapItems<'_, A, B> {}
impl<'a, A, B> Iterator for DoubleMapValues<'a, A, B> {
    type Item = &'a B;
    fn next(&mut self) -> Option<Self::Item> {
        self.pairs.next().map(|(_, second)| second)
    }
}
impl<A, B> Clone for DoubleMapValues<'_, A, B> {
    fn clone(&self) -> Self {
        Self {
            pairs: self.pairs.clone(),
        }
    }
}
impl<A, B> PartialEq for DoubleMapValues<'_, A, B> {
    fn eq(&self, other: &Self) -> bool {
        self.pairs == other.pairs
    }
}
impl<A, B> Eq for DoubleMapValues<'_, A, B> {}
impl<'a, A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> IntoIterator
    for &'a DoubleMap<A, B, H1, H2>
{
    type Item = &'a B;
    type IntoIter = DoubleMapValues<'a, A, B>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
