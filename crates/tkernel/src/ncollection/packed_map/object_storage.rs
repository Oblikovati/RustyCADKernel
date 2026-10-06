//! Raw outer-object operations from DEFINE_STANDARD_ALLOC, distinct from map/node lifetime.
use super::{PackedInteger, PackedMap};
use crate::{
    ncollection::{AllocatedBytes, BaseAllocator},
    standard::{Failure, exceptions::out_of_memory},
};
impl<T: PackedInteger> PackedMap<T> {
    /// Reserves zeroed Standard-policy bytes for scalar storage; `PackedMap::<i32>::allocate_object_storage(64)?`.
    pub fn allocate_object_storage(size: usize) -> Result<AllocatedBytes, Failure> {
        // OCCT 3d097a0's macro calls Standard::Allocate, not its uninitialized optimal policy.
        BaseAllocator::common()
            .allocate(size)
            .ok_or_else(|| out_of_memory("Standard_MMgrRaw::Allocate(): malloc failed"))
    }
    /// Releases scalar raw storage, accepting native null; `PackedMap::<i32>::free_object_storage(None)`.
    /// Constructed objects must already have been destroyed before their byte owner is released.
    pub fn free_object_storage(storage: Option<AllocatedBytes>) {
        drop(storage);
    }
    /// Reserves zeroed Standard-policy array bytes, without a compiler cookie; `PackedMap::<i32>::allocate_array_storage(128)?`.
    pub fn allocate_array_storage(size: usize) -> Result<AllocatedBytes, Failure> {
        Self::allocate_object_storage(size)
    }
    /// Releases array raw storage after destroying its elements; `PackedMap::<i32>::free_array_storage(None)`.
    pub fn free_array_storage(storage: Option<AllocatedBytes>) {
        Self::free_object_storage(storage);
    }
    /// Returns caller storage unchanged, without allocation or construction; `PackedMap::<i32>::placement_storage(0, address)`.
    pub fn placement_storage(_size: usize, address: *mut u8) -> *mut u8 {
        address
    }
    /// Preserves caller-owned placement storage after failed construction; `PackedMap::<i32>::release_placement_storage(address, address)`.
    pub fn release_placement_storage(_address: *mut u8, _placement: *mut u8) {}
}
