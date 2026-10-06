//! Native forwarding, assignment identities, and signed overloads.
use super::{BaseAllocator, Debug, HashPolicy, Map};
use crate::standard::Failure;

impl<K: Debug, H: HashPolicy<K>> Map<K, H> {
    /// Rejects a negative legacy constructor count; `Map::<i32>::with_hasher_signed(Default::default(), -1)`.
    pub fn with_hasher_signed(hasher: H, buckets: i32) -> Result<Self, Failure> {
        Self::with_allocator_signed(hasher, buckets, BaseAllocator::common())
    }
    /// Uses a node allocator and validates the signed count; `Map::<i32>::with_allocator_signed(Default::default(), 0, arena)`.
    pub fn with_allocator_signed(
        hasher: H,
        buckets: i32,
        allocator: BaseAllocator,
    ) -> Result<Self, Failure> {
        Ok(Self::with_allocator(
            hasher,
            super::super::buckets_from_int(buckets)?,
            allocator,
        ))
    }
    /// Rejects negative resize requests before mutation; `map.resize_signed(-1)`.
    pub fn resize_signed(&mut self, buckets: i32) -> Result<(), Failure> {
        self.resize(super::super::buckets_from_int(buckets)?);
        Ok(())
    }
    /// Preserves the self-assignment branch without aliasing Rust references; `map.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Exchanges storage, allocator and policy like native move assignment; `map.move_from_returning(&mut source)`.
    pub fn move_from_returning(&mut self, source: &mut Self) -> &mut Self {
        self.exchange(source);
        self
    }
    /// Constructs a key after native growth and before lookup; `map.emplace_with(|| 42)`.
    pub fn emplace_with(&mut self, construct: impl FnOnce() -> K) -> bool {
        self.prepare_emplacement();
        self.emplace(construct())
    }
    /// Constructs a replacement and returns its stable slot; `map.emplaced_with(|| 42)`.
    pub fn emplaced_with(&mut self, construct: impl FnOnce() -> K) -> &K {
        self.prepare_emplacement();
        self.emplaced(construct())
    }
    /// Constructs a lookup key even on a hit, retaining the stored key; `map.try_emplace_with(|| 42)`.
    pub fn try_emplace_with(&mut self, construct: impl FnOnce() -> K) -> bool {
        self.prepare_emplacement();
        self.try_emplace(construct())
    }
    /// Constructs a lookup key and returns the existing or inserted key; `map.try_emplaced_with(|| 42)`.
    pub fn try_emplaced_with(&mut self, construct: impl FnOnce() -> K) -> &K {
        self.prepare_emplacement();
        self.try_emplaced(construct())
    }
    pub(super) fn prepare_emplacement(&mut self) {
        // C++ emplaceImpl grows before construction, observable if the constructor fails.
        if self.is_empty() || self.len() > self.bucket_count() {
            self.resize(self.len());
        }
    }
}

impl<K: Debug + Clone, H: HashPolicy<K>> Map<K, H> {
    /// Copies keys while retaining the destination allocator and policy; `map.assign_returning(&source)`.
    pub fn assign_returning(&mut self, source: &Self) -> &mut Self {
        self.clear(false);
        if source.is_empty() {
            return self;
        }
        self.resize(source.len() - 1);
        for key in source {
            self.add_ref(key);
        }
        self
    }
}
