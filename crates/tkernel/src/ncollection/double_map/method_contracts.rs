//! Native ownership expressions, signed overloads and pointer-valued lookups.
use super::{BaseAllocator, Debug, DoubleMap, HashPolicy};
use crate::ncollection::buckets_from_int;
use crate::standard::Failure;

impl<A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> DoubleMap<A, B, H1, H2> {
    /// Checks the legacy signed constructor; `DoubleMap::with_int_buckets(p1, p2, -1, arena)`.
    pub fn with_int_buckets(
        first: H1,
        second: H2,
        buckets: i32,
        resource: BaseAllocator,
    ) -> Result<Self, Failure> {
        Ok(Self::with_allocator(
            first,
            second,
            buckets_from_int(buckets)?,
            resource,
        ))
    }
    /// Rejects negative native ReSize requests; `map.resize_int(100)?`.
    pub fn resize_int(&mut self, requested: i32) -> Result<(), Failure> {
        self.resize(buckets_from_int(requested)?);
        Ok(())
    }
    /// Preserves copy/move self-assignment identity; `map.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Moves pairs and exchanges allocator/policy state, leaving source empty; `map.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) -> &mut Self {
        std::mem::swap(&mut self.first_policy, &mut source.first_policy);
        std::mem::swap(&mut self.second_policy, &mut source.second_policy);
        self.clear(true);
        self.exchange_bindings(source);
        self
    }
    /// Moves pairs with explicit native moved-from policy behavior; `DoubleMap::move_from_with(&mut source, std::mem::take, std::mem::take)`.
    pub fn move_from_with(
        source: &mut Self,
        first_move: impl FnOnce(&mut H1) -> H1,
        second_move: impl FnOnce(&mut H2) -> H2,
    ) -> Self {
        let first = first_move(&mut source.first_policy);
        let second = second_move(&mut source.second_policy);
        let mut moved =
            Self::with_allocator(first, second, source.bucket_count, source.resource.clone());
        moved.exchange_bindings(source);
        moved
    }
    /// Returns the native second-key pointer, null when absent; `map.seek1_ptr(&1)`.
    pub fn seek1_ptr(&self, key: &A) -> *const B {
        self.seek1(key).map_or(std::ptr::null(), std::ptr::from_ref)
    }
    /// Returns the native first-key pointer, null when absent; `map.seek2_ptr(&2)`.
    pub fn seek2_ptr(&self, key: &B) -> *const A {
        self.seek2(key).map_or(std::ptr::null(), std::ptr::from_ref)
    }
    fn exchange_bindings(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.slots, &mut other.slots);
        std::mem::swap(&mut self.free, &mut other.free);
        std::mem::swap(&mut self.first_buckets, &mut other.first_buckets);
        std::mem::swap(&mut self.second_buckets, &mut other.second_buckets);
        std::mem::swap(&mut self.count, &mut other.count);
        std::mem::swap(&mut self.bucket_count, &mut other.bucket_count);
        std::mem::swap(&mut self.resource, &mut other.resource);
    }
}

impl<A: Debug, B: Debug, H1: HashPolicy<A> + Default, H2: HashPolicy<B> + Default>
    DoubleMap<A, B, H1, H2>
{
    /// Creates default policies with a checked signed bucket count; `DoubleMap::<i32, i32>::new_int(0)?`.
    pub fn new_int(buckets: i32) -> Result<Self, Failure> {
        Self::with_int_buckets(
            H1::default(),
            H2::default(),
            buckets,
            BaseAllocator::common(),
        )
    }
    /// Moves pairs and resets source policies to defaults; `DoubleMap::move_from(&mut source)`.
    /// Use `move_from_with` when a custom C++ hasher has different moved-from behavior.
    pub fn move_from(source: &mut Self) -> Self {
        Self::move_from_with(source, std::mem::take, std::mem::take)
    }
}
