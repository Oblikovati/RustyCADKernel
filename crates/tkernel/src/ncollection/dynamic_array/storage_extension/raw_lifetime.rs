//! Detached raw allocation and explicitly unsafe initialized-prefix publication.
use super::{DynamicArray, DynamicArrayStorageExtension, Failure};

impl<T> DynamicArrayStorageExtension<'_, T> {
    /// Allocates an uninitialized detached block through the native raw allocator policy.
    /// `let pointer = storage.allocate_block(size_of::<i32>())?;`
    /// The block is not registered in the array; retain its allocator until raw deallocation.
    pub fn allocate_block(&self, item_size: usize) -> Result<*mut u8, Failure> {
        let bytes = self
            .array
            .increment
            .checked_mul(item_size)
            .ok_or_else(|| self.storage_size_failure(item_size))?;
        let allocator = self.array.allocator.clone().unwrap_or_default();
        let reason = format!("raw block bytes {bytes}: expected successful allocation");
        allocator
            .allocate_optimal_raw(bytes)
            .map(|pointer| pointer.as_ptr())
            .ok_or_else(|| Failure::with_type("Standard_OutOfMemory", &reason, ""))
    }
    /// Frees a detached raw block without destroying elements or changing array size.
    /// `unsafe { storage.deallocate_block(pointer)? };`
    /// # Safety
    /// Non-null pointers must be live allocations of the current compatible raw allocator.
    /// Never pass an AllocVec-owned `block_at` address; its allocation layout differs.
    /// No live values/references may be accessed afterward. Null follows the allocator policy.
    pub unsafe fn deallocate_block(&self, pointer: *mut u8) -> Result<(), Failure> {
        let allocator = self.array.allocator.clone().unwrap_or_default();
        // SAFETY: the caller supplies this allocator's live raw-policy allocation or null.
        unsafe { allocator.free_raw_nullable(pointer) }
    }
    /// Constructs a value in caller-owned raw storage without publishing it in the array.
    /// `unsafe { DynamicArrayStorageExtension::<i32>::construct_at(pointer, || 42) };`
    /// # Safety
    /// The pointer must be aligned, writable for T, exclusively accessible, and uninitialized.
    /// Caller owns cleanup until initialized slots are published; no active reference may alias.
    pub unsafe fn construct_at(pointer: *mut T, construct: impl FnOnce() -> T) {
        let value = construct();
        // SAFETY: the caller provides exclusive aligned storage with no owned live value.
        unsafe { pointer.write(value) };
    }
    /// Ends one value's lifetime without freeing storage or changing initialized lengths.
    /// `unsafe { DynamicArrayStorageExtension::<i32>::destroy_at(pointer) };`
    /// # Safety
    /// Pointer must identify one exclusively accessible live T removed from automatic ownership.
    /// Do not read, destroy or republish it again without reconstruction. Drop must not unwind
    /// when matching native destroyAt's noexcept contract.
    pub unsafe fn destroy_at(pointer: *mut T) {
        // SAFETY: caller guarantees a live value whose destructor will not run automatically.
        unsafe { pointer.drop_in_place() };
    }
    /// Publishes exactly the initialized prefix without constructing, moving or dropping values.
    /// `unsafe { storage.set_used_size(3)? };`
    /// # Safety
    /// Every slot in [0,new_used) must contain a valid fully initialized T uniquely owned by this
    /// array. Newly exposed slots must already have been constructed. No conflicting references
    /// or duplicate owners may exist. Removed slots lose automatic Drop ownership; caller must
    /// destroy or validly republish them. Never publish a slot after destruction without rebuilding it.
    /// On a validation error, no initialization preconditions apply and metadata is unchanged.
    pub unsafe fn set_used_size(&mut self, new_used: usize) -> Result<(), Failure> {
        self.validate_initialized_prefix(new_used)?;
        for (index, block) in self.array.blocks.iter_mut().enumerate() {
            let length = initialized_block_length(index, self.array.increment, new_used);
            // SAFETY: validation checks capacity; the caller promises initialized unique values.
            unsafe { block.set_len(length) };
        }
        self.array.used = new_used;
        Ok(())
    }
    /// Copies native parameters into a destination with no allocated blocks.
    /// `destination.storage_extension().copy_parameters(&source, true)?;`
    /// Retained blocks are rejected because allocator rebinding needs a separate ownership model.
    /// Existing detached pointers still require their original allocator after an allocator change.
    /// The unsafe `copy_parameters_retained` adapter supplies the separate retained ownership policy.
    pub fn copy_parameters(
        &mut self,
        source: &DynamicArray<T>,
        copy_allocator: bool,
    ) -> Result<(), Failure> {
        self.require_unallocated_parameters()?;
        self.apply_parameter_metadata(source, copy_allocator);
        Ok(())
    }
    fn require_unallocated_parameters(&self) -> Result<(), Failure> {
        if self.block_count() == 0 {
            return Ok(());
        }
        let reason = format!(
            "destination block count {}: expected zero allocated blocks",
            self.block_count()
        );
        Err(Failure::with_type("Standard_DomainError", &reason, ""))
    }
    fn validate_initialized_prefix(&self, new_used: usize) -> Result<(), Failure> {
        let capacity = self.available_size();
        let unavailable = self.array.blocks.iter().enumerate().any(|(index, block)| {
            initialized_block_length(index, self.array.increment, new_used) > block.capacity()
        });
        if new_used <= capacity && !unavailable {
            return Ok(());
        }
        let reason = format!(
            "used size {new_used}: expected initialized physical blocks within logical capacity {capacity}"
        );
        Err(Failure::with_type("Standard_OutOfRange", &reason, ""))
    }
}
fn initialized_block_length(index: usize, increment: usize, used: usize) -> usize {
    used.saturating_sub(index * increment).min(increment)
}
