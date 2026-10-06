//! Legacy signed bucket counts retain native range validation before mutation.
use super::{BaseAllocator, DataMap, Debug, HashPolicy};
use crate::standard::Failure;

impl<K: Debug, V, H: HashPolicy<K>> DataMap<K, V, H> {
    /// Validates the legacy signed constructor count; `DataMap::<i32,i32>::with_hasher_signed(Default::default(), -1)`.
    pub fn with_hasher_signed(hasher: H, bucket_count: i32) -> Result<Self, Failure> {
        Self::with_allocator_signed(hasher, bucket_count, BaseAllocator::common())
    }
    /// Selects an allocator after validating a signed bucket count; `DataMap::<i32,i32>::with_allocator_signed(Default::default(), 0, arena)`.
    pub fn with_allocator_signed(
        hasher: H,
        bucket_count: i32,
        resource: BaseAllocator,
    ) -> Result<Self, Failure> {
        Ok(Self::with_allocator(
            hasher,
            super::super::buckets_from_int(bucket_count)?,
            resource,
        ))
    }
    /// Rejects a negative legacy resize without modifying bindings; `map.resize_signed(-1)`.
    pub fn resize_signed(&mut self, bucket_count: i32) -> Result<(), Failure> {
        self.resize(super::super::buckets_from_int(bucket_count)?);
        Ok(())
    }
}
