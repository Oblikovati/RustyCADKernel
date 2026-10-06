#![allow(unsafe_code)]
use super::{PackedInteger, PackedMap, PackedMapNode, PackedWord, storage::PackedNodeTable};
/// Restartable native cursor and Rust value iterator; `map.iter()`.
#[derive(Clone)]
pub struct PackedMapIter<'a, T: PackedInteger> {
    table: Option<&'a PackedNodeTable<T>>,
    bucket: usize,
    node: Option<&'a PackedMapNode<T>>,
    remaining: u64,
    cached_key: T,
}
// The immutable table borrow prevents link changes or freeing any cursor node.
unsafe impl<T: PackedInteger + Send> Send for PackedMapIter<'_, T> where T::Word: Sync {}
unsafe impl<T: PackedInteger + Sync> Sync for PackedMapIter<'_, T> where T::Word: Sync {}
impl<T: PackedInteger> PackedMap<T> {
    /// Traverse buckets, chains, then ascending bits; `map.iter().collect::<Vec<_>>()`.
    pub fn iter(&self) -> PackedMapIter<'_, T> {
        PackedMapIter::over(Some(&self.buckets))
    }
}
/// The native default `Iterator()`: bound to no map, exhausted, equal to every end.
impl<T: PackedInteger> Default for PackedMapIter<'_, T> {
    fn default() -> Self {
        Self::over(None)
    }
}
impl<'a, T: PackedInteger> PackedMapIter<'a, T> {
    fn over(table: Option<&'a PackedNodeTable<T>>) -> Self {
        let mut iter = Self {
            table,
            bucket: 0,
            node: None,
            remaining: T::Word::from_bits(u64::MAX).bits(),
            cached_key: T::from_bits(0),
        };
        iter.find_first();
        if let Some(node) = iter.node {
            iter.take_key(node);
        }
        iter
    }
    /// Check for a current bit; `iter.more()`.
    pub fn more(&self) -> bool {
        self.node.is_some()
    }
    /// Borrow the cursor's value; `iter.key()`.
    pub fn key(&self) -> Option<&T> {
        self.node.map(|_| &self.cached_key)
    }
    /// Advance once, retaining exhausted state; `iter.advance()`.
    pub fn advance(&mut self) {
        while let Some(node) = self.node {
            self.take_key(node);
            if self.remaining != T::Word::from_bits(u64::MAX).bits() {
                return;
            }
            self.next_node();
        }
    }
    fn take_key(&mut self, node: &PackedMapNode<T>) {
        let mut mask = T::Word::from_bits(self.remaining);
        self.cached_key = PackedMap::<T>::find_next(node, &mut mask);
        self.remaining = mask.bits();
    }
    fn find_first(&mut self) {
        let Some(table) = self.table else {
            return;
        };
        while self.bucket < table.len() {
            self.node = table.head(self.bucket);
            if self.node.is_some() {
                return;
            }
            self.bucket += 1;
        }
    }
    fn next_node(&mut self) {
        self.node = self
            .node
            .and_then(|node| self.table.and_then(|table| table.following(node)));
        if self.node.is_some() {
            return;
        }
        self.bucket += 1;
        self.find_first();
    }
    /// Restart against the same borrowed map; `iter.reset()`.
    pub fn reset(&mut self) {
        *self = Self::over(self.table);
    }
    /// Rebind to another map with the same lifetime; `iter.initialize(&other)`.
    pub fn initialize(&mut self, map: &'a PackedMap<T>) {
        *self = map.iter();
    }
}
impl<T: PackedInteger> Iterator for PackedMapIter<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let value = self.key().copied()?;
        self.advance();
        Some(value)
    }
}
impl<T: PackedInteger> PartialEq for PackedMapIter<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        if !self.more() || !other.more() {
            return self.more() == other.more();
        }
        let table_equal = self
            .table
            .zip(other.table)
            .is_some_and(|(a, b)| std::ptr::eq(a, b));
        let node_equal = self
            .node
            .zip(other.node)
            .is_some_and(|(a, b)| std::ptr::eq(a, b));
        table_equal
            && node_equal
            && self.bucket == other.bucket
            && self.remaining == other.remaining
            && self.cached_key == other.cached_key
    }
}
impl<T: PackedInteger> Eq for PackedMapIter<'_, T> {}
impl<'a, T: PackedInteger> IntoIterator for &'a PackedMap<T> {
    type Item = T;
    type IntoIter = PackedMapIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
mod method_contracts;
