//! Two actual native hash chains share stable DoubleMapNode key pairs.
#![allow(unsafe_code)]
use super::{BaseAllocator, DefaultHashPolicy, DoubleMapNode, HashPolicy, MapLinkNode};
use crate::standard::Failure;
use std::fmt::Debug;
mod storage;
use storage::DoubleStorage;
pub use storage::{DoubleMapObservation, DoubleMapObserver};
/// A bijection indexed independently by both keys; `DoubleMap::<i32,i32>::default().bind(1,42)?`.
pub struct DoubleMap<A, B, H1 = DefaultHashPolicy, H2 = DefaultHashPolicy> {
    storage: DoubleStorage<A, B>,
    first_policy: H1,
    second_policy: H2,
}
/// Shared pairs in first-key bucket order; `map.items()`.
pub struct DoubleMapItems<'a, A, B> {
    source: Option<&'a DoubleStorage<A, B>>,
    position: super::owned_bucket_chains::BucketChainPosition,
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
    /// Borrows the node allocator; `map.allocator()==&arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        self.storage.allocator()
    }
    /// Rehashes both indices without relocating pairs; `map.resize(200)`.
    pub fn resize(&mut self, requested: usize) {
        let (first, second) = (&self.first_policy, &self.second_policy);
        self.storage.resize(requested, |node, count| {
            (
                first.hash(node.key1()) % count + 1,
                second.hash(node.key2()) % count + 1,
            )
        });
    }
    /// Binds new keys or reports MultiplyDefined; `map.bind(1,42)?`.
    pub fn bind(&mut self, first: A, second: B) -> Result<(), Failure> {
        let Some((one, two)) = self.prepare_binding(&first, &second) else {
            return Err(borrowed_binding::duplicate_pair(&first, &second));
        };
        self.storage.insert(one, two, || first, || second);
        Ok(())
    }
    /// Binds only when neither key exists; `map.try_bind(1,42)`.
    pub fn try_bind(&mut self, first: A, second: B) -> bool {
        let Some((one, two)) = self.prepare_binding(&first, &second) else {
            return false;
        };
        self.storage.insert(one, two, || first, || second);
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
        Some(unsafe { self.storage.node(self.lookup1(key)?) }.key2())
    }
    /// Optionally borrows the first key; `map.seek2(&42)`.
    pub fn seek2(&self, key: &B) -> Option<&A> {
        Some(unsafe { self.storage.node(self.lookup2(key)?) }.key1())
    }
    /// Removes both indices through the first key; `map.unbind1(&1)`.
    pub fn unbind1(&mut self, key: &A) -> bool {
        if self.is_empty() {
            return false;
        }
        let bucket = self.first_policy.hash(key) % self.bucket_count() + 1;
        let Some(node) = self.storage.find(bucket, false, |node| {
            self.first_policy.equal(node.key1(), key)
        }) else {
            return false;
        };
        self.remove_native_pair(bucket, node, false)
    }
    /// Removes both indices through the second key; `map.unbind2(&42)`.
    pub fn unbind2(&mut self, key: &B) -> bool {
        if self.is_empty() {
            return false;
        }
        let bucket = self.second_policy.hash(key) % self.bucket_count() + 1;
        let Some(node) = self.storage.find(bucket, true, |node| {
            self.second_policy.equal(node.key2(), key)
        }) else {
            return false;
        };
        self.remove_native_pair(bucket, node, true)
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
    fn lookup1(&self, key: &A) -> Option<*mut MapLinkNode> {
        if self.is_empty() {
            return None;
        }
        let bucket = self.first_policy.hash(key) % self.bucket_count() + 1;
        self.storage.find(bucket, false, |node| {
            self.first_policy.equal(node.key1(), key)
        })
    }
    fn lookup2(&self, key: &B) -> Option<*mut MapLinkNode> {
        if self.is_empty() {
            return None;
        }
        let bucket = self.second_policy.hash(key) % self.bucket_count() + 1;
        self.storage.find(bucket, true, |node| {
            self.second_policy.equal(node.key2(), key)
        })
    }
    fn prepare_binding(&mut self, first: &A, second: &B) -> Option<(usize, usize)> {
        if self.storage.base().resizable() {
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
    fn match_first(&self, bucket: usize, key: &A) -> Option<*mut MapLinkNode> {
        self.storage.find(bucket, false, |node| {
            self.first_policy.equal(node.key1(), key)
        })
    }
    fn match_second(&self, bucket: usize, key: &B) -> Option<*mut MapLinkNode> {
        self.storage.find(bucket, true, |node| {
            self.second_policy.equal(node.key2(), key)
        })
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

mod base_bridge;
mod removal;
