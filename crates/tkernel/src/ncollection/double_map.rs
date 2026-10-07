//! `NCollection_DoubleMap`: a bijection indexed by two native hash chains over safe slots.
use super::{BaseAllocator, DefaultHashPolicy, HashPolicy, slot_chains::SlotRange};
use crate::standard::Failure;
use std::fmt::Debug;
mod storage;
use storage::DoubleStorage;
/// A bijection indexed independently by both keys; `DoubleMap::<i32,i32>::default().bind(1,42)?`.
pub struct DoubleMap<A, B, H1 = DefaultHashPolicy, H2 = DefaultHashPolicy> {
    storage: DoubleStorage<A, B>,
    first_policy: H1,
    second_policy: H2,
}
/// Shared pairs in first-key bucket order; `map.items()`.
pub struct DoubleMapItems<'a, A, B> {
    entries: SlotRange<'a, A, B>,
}
/// Shared second keys in first-key bucket order; `map.iter()`.
pub struct DoubleMapValues<'a, A, B> {
    pairs: DoubleMapItems<'a, A, B>,
}
impl<A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> DoubleMap<A, B, H1, H2> {
    /// Injects both equality/hash policies and the node resource; `DoubleMap::with_allocator(p1,p2,10,arena)`.
    pub fn with_allocator(
        first_policy: H1,
        second_policy: H2,
        buckets: usize,
        resource: BaseAllocator,
    ) -> Self {
        Self {
            storage: DoubleStorage::new(buckets, resource),
            first_policy,
            second_policy,
        }
    }
    /// Returns pair count; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.storage.len()
    }
    /// Returns OCCT's pair extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.len()
    }
    /// Checks for no pairs; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns the common hash modulus; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.storage.bucket_count()
    }
    /// Borrows the map allocator; `map.allocator()==&arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        self.storage.allocator()
    }
    /// Rehashes both indices, Key1 then Key2 of each pair in first-key order; `map.resize(200)`.
    pub fn resize(&mut self, requested: usize) {
        let (first, second) = (&self.first_policy, &self.second_policy);
        self.storage.resize(requested, |key1, key2, count| {
            (first.hash(key1) % count + 1, second.hash(key2) % count + 1)
        });
    }
    /// Binds new keys or reports MultiplyDefined; `map.bind(1,42)?`.
    pub fn bind(&mut self, first: A, second: B) -> Result<(), Failure> {
        let Some(buckets) = self.prepare_binding(&first, &second) else {
            return Err(borrowed_binding::duplicate_pair(&first, &second));
        };
        self.storage.insert(buckets, || first, || second);
        Ok(())
    }
    /// Binds only when neither key exists; `map.try_bind(1,42)`.
    pub fn try_bind(&mut self, first: A, second: B) -> bool {
        let Some(buckets) = self.prepare_binding(&first, &second) else {
            return false;
        };
        self.storage.insert(buckets, || first, || second);
        true
    }
    /// Moves constructed keys into a new pair only when both are absent; `map.try_emplace(1,42)`.
    pub fn try_emplace(&mut self, first: A, second: B) -> bool {
        self.try_bind(first, second)
    }
    /// Tests first-key membership; `map.is_bound1(&1)`.
    pub fn is_bound1(&self, key: &A) -> bool {
        self.lookup1(key).is_some()
    }
    /// Tests second-key membership; `map.is_bound2(&42)`.
    pub fn is_bound2(&self, key: &B) -> bool {
        self.lookup2(key).is_some()
    }
    /// Finds a second key through the first, reporting NoSuchObject if absent; `map.find1(&1)?`.
    pub fn find1(&self, key: &A) -> Result<&B, Failure> {
        self.seek1(key).ok_or_else(|| missing_pair(key))
    }
    /// Finds a first key through the second; `map.find2(&42)?`.
    pub fn find2(&self, key: &B) -> Result<&A, Failure> {
        self.seek2(key).ok_or_else(|| missing_pair(key))
    }
    /// Optionally borrows the second key; `map.seek1(&1)`.
    pub fn seek1(&self, key: &A) -> Option<&B> {
        Some(self.storage.slot(self.lookup1(key)?).value())
    }
    /// Optionally borrows the first key; `map.seek2(&42)`.
    pub fn seek2(&self, key: &B) -> Option<&A> {
        Some(self.storage.slot(self.lookup2(key)?).key())
    }
    /// Removes both indices through the first key; `map.unbind1(&1)`.
    pub fn unbind1(&mut self, key: &A) -> bool {
        let Some(slot) = self.lookup1(key) else {
            return false;
        };
        // Native UnBind1 hashes Key2 to reach the second chain. The slot already records that
        // bucket, so the call keeps only the native hasher sequence; a key changed through
        // interior mutability still unlinks safely (native behaviour is undefined there).
        let _ = self.second_policy.hash(self.storage.slot(slot).value());
        drop(self.storage.remove(slot));
        true
    }
    /// Removes both indices through the second key; `map.unbind2(&42)`.
    pub fn unbind2(&mut self, key: &B) -> bool {
        let Some(slot) = self.lookup2(key) else {
            return false;
        };
        // Native UnBind2 hashes Key1 to reach the first chain (see `unbind1`).
        let _ = self.first_policy.hash(self.storage.slot(slot).key());
        drop(self.storage.remove(slot));
        true
    }
    /// Drops pairs and optionally bucket storage; `map.clear(true)`.
    pub fn clear(&mut self, release: bool) {
        self.storage.clear(release);
    }
    /// Swaps pairs, policies and allocators; `map.exchange(&mut other)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.first_policy, &mut other.first_policy);
        std::mem::swap(&mut self.second_policy, &mut other.second_policy);
        self.storage.exchange(&mut other.storage);
    }
    fn lookup1(&self, key: &A) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        self.match_first(self.first_policy.hash(key) % self.bucket_count() + 1, key)
    }
    fn lookup2(&self, key: &B) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        self.match_second(self.second_policy.hash(key) % self.bucket_count() + 1, key)
    }
    fn prepare_binding(&mut self, first: &A, second: &B) -> Option<(usize, usize)> {
        if self.storage.resizable() {
            self.resize(self.len());
        }
        let one = self.first_policy.hash(first) % self.bucket_count() + 1;
        if self.match_first(one, first).is_some() {
            return None;
        }
        let two = self.second_policy.hash(second) % self.bucket_count() + 1;
        if self.match_second(two, second).is_some() {
            return None;
        }
        Some((one, two))
    }
    fn match_first(&self, bucket: usize, key: &A) -> Option<usize> {
        self.storage
            .find1(bucket, |stored| self.first_policy.equal(stored, key))
    }
    fn match_second(&self, bucket: usize, key: &B) -> Option<usize> {
        self.storage
            .find2(bucket, |stored| self.second_policy.equal(stored, key))
    }
}
impl<A: Debug, B: Debug, H1: HashPolicy<A> + Default, H2: HashPolicy<B> + Default>
    DoubleMap<A, B, H1, H2>
{
    /// Creates default policies with the requested bucket count; `DoubleMap::<i32,i32>::new(100)`.
    pub fn new(buckets: usize) -> Self {
        Self::with_allocator(
            H1::default(),
            H2::default(),
            buckets,
            BaseAllocator::common(),
        )
    }
}
impl<A: Debug, B: Debug, H1: HashPolicy<A> + Default, H2: HashPolicy<B> + Default> Default
    for DoubleMap<A, B, H1, H2>
{
    fn default() -> Self {
        Self::new(1)
    }
}
fn missing_pair<K: Debug>(key: &K) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("DoubleMap key {key:?}; expected a bound key"),
        "",
    )
}
mod copying;
mod cursor;
mod iteration;
mod queries;
pub use cursor::DoubleMapIterator;

mod borrowed_binding;
mod method_contracts;
