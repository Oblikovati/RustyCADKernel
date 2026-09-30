// Rust translation of FoundationClasses/TKernel/Standard/Standard_MMgrRoot.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The abstract interface of OCCT memory managers.
use super::Failure;
use std::num::NonZeroUsize;

/// The address of a block's user area, the value a native `void*` holds; `storage.get()`.
///
/// A manager hands these out and resolves them back to bytes, because safe Rust cannot
/// dereference an address. `Option<StorageAddress>` is a nullable pointer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StorageAddress(NonZeroUsize);
impl StorageAddress {
    /// Wraps a non-null address; `StorageAddress::new(0x1000).unwrap().get() == 0x1000`.
    pub const fn new(address: usize) -> Option<Self> {
        match NonZeroUsize::new(address) {
            Some(address) => Some(Self(address)),
            None => None,
        }
    }
    /// Returns the numeric address; `storage.get() % 8 == 0`.
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

/// Root of OCCT memory managers (`Standard_MMgrRoot`); `manager.allocate(16)?`.
///
/// `bytes`/`bytes_mut` stand for dereferencing the returned pointer: they borrow the whole
/// usable area of a live block, which is at least the requested size.
pub trait MMgrRoot {
    /// Allocates `size` bytes; `manager.allocate(24)?`.
    fn allocate(&mut self, size: usize) -> Result<StorageAddress, Failure>;
    /// Grows or keeps a block; `None` allocates; `manager.reallocate(Some(storage), 64)?`.
    fn reallocate(
        &mut self,
        storage: Option<StorageAddress>,
        size: usize,
    ) -> Result<StorageAddress, Failure>;
    /// Releases a block; `None` is ignored like a null pointer; `manager.free(Some(storage))?`.
    fn free(&mut self, storage: Option<StorageAddress>) -> Result<(), Failure>;
    /// Releases cached memory, returning non-zero if any was released; `manager.purge(false)`.
    /// The root implementation caches nothing and returns 0.
    fn purge(&mut self, is_destroyed: bool) -> i32 {
        let _ = is_destroyed;
        0
    }
    /// Borrows a live block's usable bytes; `manager.bytes(storage)?[0]`.
    fn bytes(&self, storage: StorageAddress) -> Result<&[u8], Failure>;
    /// Borrows a live block's usable bytes for writing; `manager.bytes_mut(storage)?.fill(0)`.
    fn bytes_mut(&mut self, storage: StorageAddress) -> Result<&mut [u8], Failure>;
}
