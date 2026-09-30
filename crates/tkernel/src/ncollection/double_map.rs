//! Two hash indices share each stable pair, preserving OCCT's first-key traversal.
use super::hash_policy::next_map_prime;
use super::{AllocatedBox, BaseAllocator, DefaultHashPolicy, HashPolicy};
use crate::standard::Failure;
use std::fmt::Debug;
struct Pair<A, B> {
    first: A,
    second: B,
    _identity: u8,
}
type Slots<A, B> = Vec<Option<AllocatedBox<Pair<A, B>>>>;
/// A bijection indexed independently by both keys; `DoubleMap::<i32,i32>::default().bind(1,42)?`.
pub struct DoubleMap<A, B, H1 = DefaultHashPolicy, H2 = DefaultHashPolicy> {
    slots: Slots<A, B>,
    free: Vec<usize>,
    first_buckets: Vec<Vec<usize>>,
    second_buckets: Vec<Vec<usize>>,
    count: usize,
    bucket_count: usize,
    resource: BaseAllocator,
    first_policy: H1,
    second_policy: H2,
}
/// Shared pairs in first-key bucket order; `map.items()`.
pub struct DoubleMapItems<'a, A, B> {
    slots: &'a [Option<AllocatedBox<Pair<A, B>>>],
    buckets: &'a [Vec<usize>],
    bucket: usize,
    index: usize,
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
            slots: Vec::new(),
            free: Vec::new(),
            first_buckets: Vec::new(),
            second_buckets: Vec::new(),
            count: 0,
            // The native constructors keep a zero bucket count until the first insertion resizes.
            bucket_count: buckets,
            resource,
            first_policy,
            second_policy,
        }
    }
    /// Returns pair count; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.count
    }
    /// Returns OCCT's pair extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.count
    }
    /// Checks for no pairs; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    /// Returns the common hash modulus; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.bucket_count
    }
    /// Borrows the node allocator; `map.allocator()==&arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        &self.resource
    }
    /// Rehashes both indices without relocating pairs; `map.resize(200)`.
    pub fn resize(&mut self, requested: usize) {
        let count = next_map_prime(requested).max(self.bucket_count);
        if count == self.bucket_count && !self.first_buckets.is_empty() {
            return;
        }
        let mut first: Vec<Vec<usize>> = (0..=count).map(|_| Vec::new()).collect();
        let mut second: Vec<Vec<usize>> = (0..=count).map(|_| Vec::new()).collect();
        for &slot in self.first_buckets.iter().flatten() {
            let pair = self.slots[slot]
                .as_ref()
                .expect("bucket points to a live pair");
            first[self.first_policy.hash(&pair.first) % count + 1].insert(0, slot);
            second[self.second_policy.hash(&pair.second) % count + 1].insert(0, slot);
        }
        self.first_buckets = first;
        self.second_buckets = second;
        self.bucket_count = count;
    }
    /// Binds new keys or reports MultiplyDefined; `map.bind(1,42)?`.
    pub fn bind(&mut self, first: A, second: B) -> Result<(), Failure> {
        self.prepare_insert();
        if self.is_bound1(&first) || self.is_bound2(&second) {
            return Err(Failure::with_type(
                "Standard_MultiplyDefined",
                &format!("DoubleMap pair ({first:?}, {second:?}); expected two unbound keys"),
                "",
            ));
        }
        self.insert_pair(first, second);
        Ok(())
    }
    /// Binds only when neither key exists; `map.try_bind(1,42)`.
    pub fn try_bind(&mut self, first: A, second: B) -> bool {
        self.prepare_insert();
        if self.is_bound1(&first) || self.is_bound2(&second) {
            return false;
        }
        self.insert_pair(first, second);
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
        Some(&self.slots[self.lookup1(key)?].as_ref()?.second)
    }
    /// Optionally borrows the first key; `map.seek2(&42)`.
    pub fn seek2(&self, key: &B) -> Option<&A> {
        Some(&self.slots[self.lookup2(key)?].as_ref()?.first)
    }
    /// Removes both indices through the first key; `map.unbind1(&1)`.
    pub fn unbind1(&mut self, key: &A) -> bool {
        let Some(slot) = self.lookup1(key) else {
            return false;
        };
        self.remove_slot(slot);
        true
    }
    /// Removes both indices through the second key; `map.unbind2(&42)`.
    pub fn unbind2(&mut self, key: &B) -> bool {
        let Some(slot) = self.lookup2(key) else {
            return false;
        };
        self.remove_slot(slot);
        true
    }
    /// Drops pairs and optionally bucket storage; `map.clear(true)`.
    pub fn clear(&mut self, release: bool) {
        self.slots.clear();
        self.free.clear();
        self.count = 0;
        if release {
            self.first_buckets.clear();
            self.second_buckets.clear();
            return;
        }
        for bucket in self
            .first_buckets
            .iter_mut()
            .chain(&mut self.second_buckets)
        {
            bucket.clear();
        }
    }
    /// Swaps pairs, policies and allocators; `map.exchange(&mut other)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    fn lookup1(&self, key: &A) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        let hash = self.first_policy.hash(key) % self.bucket_count + 1;
        self.first_buckets[hash].iter().copied().find(|&slot| {
            self.first_policy
                .equal(&self.slots[slot].as_ref().unwrap().first, key)
        })
    }
    fn lookup2(&self, key: &B) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        let hash = self.second_policy.hash(key) % self.bucket_count + 1;
        self.second_buckets[hash].iter().copied().find(|&slot| {
            self.second_policy
                .equal(&self.slots[slot].as_ref().unwrap().second, key)
        })
    }
    fn prepare_insert(&mut self) {
        if self.is_empty() || self.count > self.bucket_count {
            self.resize(self.count);
        }
    }
    fn insert_pair(&mut self, first: A, second: B) {
        let h1 = self.first_policy.hash(&first) % self.bucket_count + 1;
        let h2 = self.second_policy.hash(&second) % self.bucket_count + 1;
        let pair = self.resource.allocate_value(Pair {
            first,
            second,
            _identity: 0,
        });
        let slot = self.free.pop().unwrap_or_else(|| {
            self.slots.push(None);
            self.slots.len() - 1
        });
        self.slots[slot] = Some(pair);
        self.first_buckets[h1].insert(0, slot);
        self.second_buckets[h2].insert(0, slot);
        self.count += 1;
    }
    fn remove_slot(&mut self, slot: usize) {
        let pair = self.slots[slot]
            .as_ref()
            .expect("lookup returned live pair");
        let h1 = self.first_policy.hash(&pair.first) % self.bucket_count + 1;
        let h2 = self.second_policy.hash(&pair.second) % self.bucket_count + 1;
        self.first_buckets[h1].retain(|&entry| entry != slot);
        self.second_buckets[h2].retain(|&entry| entry != slot);
        self.slots[slot] = None;
        self.free.push(slot);
        self.count -= 1;
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
