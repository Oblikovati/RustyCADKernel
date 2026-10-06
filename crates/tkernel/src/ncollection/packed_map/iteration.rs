use super::{PackedBlock, PackedInteger, PackedMap, PackedWord};
/// Restartable native cursor and Rust value iterator; `map.iter()`.
#[derive(Clone)]
pub struct PackedMapIter<'a, T: PackedInteger> {
    // The map's bucket chains; empty for a default-constructed native iterator.
    buckets: &'a [Vec<PackedBlock<T::Word>>],
    bucket: usize,
    node: usize,
    remaining: u64,
    current: Option<T>,
    cached_key: T,
}
impl<T: PackedInteger> PackedMap<T> {
    /// Traverse buckets, chains, then ascending bits; `map.iter().collect::<Vec<_>>()`.
    pub fn iter(&self) -> PackedMapIter<'_, T> {
        PackedMapIter::over(&self.buckets)
    }
}
/// The native default `Iterator()`: bound to no map, exhausted, equal to every end.
impl<T: PackedInteger> Default for PackedMapIter<'_, T> {
    fn default() -> Self {
        Self::over(&[])
    }
}
impl<'a, T: PackedInteger> PackedMapIter<'a, T> {
    fn over(buckets: &'a [Vec<PackedBlock<T::Word>>]) -> Self {
        let mut iter = Self {
            buckets,
            bucket: 0,
            node: 0,
            remaining: 0,
            current: None,
            cached_key: T::from_bits(0),
        };
        iter.advance();
        iter
    }
    /// Check for a current bit; `iter.more()`.
    pub fn more(&self) -> bool {
        self.current.is_some()
    }
    /// Borrow the cursor's value; `iter.key()`.
    pub fn key(&self) -> Option<&T> {
        self.current.as_ref()
    }
    /// Advance once, retaining exhausted state; `iter.advance()`.
    pub fn advance(&mut self) {
        self.cache_exhausted_block();
        while self.remaining == 0 {
            if self.bucket >= self.buckets.len() {
                self.current = None;
                return;
            }
            if self.node >= self.buckets[self.bucket].len() {
                self.bucket += 1;
                self.node = 0;
                continue;
            }
            self.remaining = self.buckets[self.bucket][self.node].bits.bits();
            self.node += 1;
        }
        let block = &self.buckets[self.bucket][self.node - 1];
        self.current = Some(T::from_bits(
            (block.index.bits() << PackedMap::<T>::SHIFT)
                | u64::from(self.remaining.trailing_zeros()),
        ));
        self.cached_key = self.current.expect("selected bit has a current key");
        self.remaining &= self.remaining - 1;
    }
    fn cache_exhausted_block(&mut self) {
        if self.remaining == 0 && self.current.is_some() {
            // Native findNext returns the block base when its remaining-bit mask is empty.
            let block = &self.buckets[self.bucket][self.node - 1];
            self.cached_key = T::from_bits(block.index.bits() << PackedMap::<T>::SHIFT);
        }
    }
    /// Restart against the same borrowed map; `iter.reset()`.
    pub fn reset(&mut self) {
        *self = Self::over(self.buckets);
    }
    /// Rebind to another map with the same lifetime; `iter.initialize(&other)`.
    pub fn initialize(&mut self, map: &'a PackedMap<T>) {
        *self = map.iter();
    }
}
impl<T: PackedInteger> Iterator for PackedMapIter<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let value: T = self.current?;
        self.advance();
        Some(value)
    }
}
impl<T: PackedInteger> PartialEq for PackedMapIter<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        if !self.more() || !other.more() {
            return self.more() == other.more();
        }
        std::ptr::eq(self.buckets, other.buckets)
            && (self.bucket, self.node, self.remaining, self.current)
                == (other.bucket, other.node, other.remaining, other.current)
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
