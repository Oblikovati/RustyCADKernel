//! Typed protected storage operations; allocations remain owned by the array.
use super::{ArrayError, DynamicArray, emplacement::forget_block_values_after};
use crate::standard::Failure;

#[allow(unsafe_code)]
mod raw_lifetime;
#[allow(unsafe_code)]
mod replacement;

/// Exclusively borrows the array for OCCT's protected block operations.
/// `let mut storage = array.storage_extension(); storage.ensure_block_count(2, size_of::<i32>())?;`
/// Raw pointer access never creates references to uninitialized block capacity.
pub struct DynamicArrayStorageExtension<'a, T> {
    array: &'a mut DynamicArray<T>,
}
impl<T> DynamicArray<T> {
    /// Borrows the protected typed block engine; `array.storage_extension().block_count()`.
    pub fn storage_extension(&mut self) -> DynamicArrayStorageExtension<'_, T> {
        DynamicArrayStorageExtension { array: self }
    }
    pub(super) fn reserve_block_count(&mut self, count: usize) {
        // Native ensureBlockCount reserves its pointer vector before element allocations.
        self.blocks.reserve(count.saturating_sub(self.blocks.len()));
        while self.blocks.len() < count {
            self.add_block();
        }
    }
}
impl<T> DynamicArrayStorageExtension<'_, T> {
    /// Rounds like native unsigned arithmetic, including overflow to zero.
    /// `DynamicArrayStorageExtension::<i32>::round_up_pow2(5) == 8`.
    pub const fn round_up_pow2(value: usize) -> usize {
        let mut rounded = if value == 0 { 0 } else { value - 1 };
        let mut shift = 1;
        while shift < usize::BITS {
            rounded |= rounded >> shift;
            shift *= 2;
        }
        rounded.wrapping_add(1)
    }
    /// Returns native floor-log behavior, including zero for zero.
    /// `DynamicArrayStorageExtension::<i32>::log2_pow2(7) == 2`.
    pub const fn log2_pow2(mut value: usize) -> usize {
        let mut shift = 0;
        while value > 1 {
            value >>= 1;
            shift += 1;
        }
        shift
    }
    /// Counts allocated blocks, including retained empty blocks; `storage.block_count()`.
    pub fn block_count(&self) -> usize {
        self.array.blocks.len()
    }
    /// Counts slots with the current native stride, not physical retained capacity.
    /// `storage.available_size()` changes after decreasing the empty array's increment.
    pub fn available_size(&self) -> usize {
        self.block_count() * self.array.increment
    }
    /// Allocates typed storage through the requested index without constructing elements.
    /// `storage.ensure_storage_for_index(8, size_of::<i32>())?` leaves Size unchanged.
    pub fn ensure_storage_for_index(
        &mut self,
        index: usize,
        item_size: usize,
    ) -> Result<(), Failure> {
        let count = (index / self.array.increment).checked_add(1);
        let count = count.ok_or_else(|| self.storage_size_failure(index))?;
        self.ensure_block_count(count, item_size)
    }
    /// Reserves blocks before constructing any elements; `storage.ensure_block_count(3, size_of::<i32>())?`.
    /// Item size must match T; arbitrary native byte-sized blocks are a separate raw API.
    pub fn ensure_block_count(&mut self, count: usize, item_size: usize) -> Result<(), Failure> {
        self.validate_item_size(item_size)?;
        self.validate_block_count(count)?;
        self.array.reserve_block_count(count);
        Ok(())
    }
    /// Adds exactly one typed block and returns its mutable raw address.
    /// `storage.expand_block(size_of::<i32>())?` does not initialize any T values.
    pub fn expand_block(&mut self, item_size: usize) -> Result<*mut (), Failure> {
        self.validate_item_size(item_size)?;
        let index = self.block_count();
        self.validate_block_count(index + 1)?;
        self.array.add_block();
        self.change_block_address(index)
    }
    /// Returns a const untyped block address; `storage.block_address(0)?`.
    /// Dereferencing uninitialized capacity is not authorized by this accessor.
    pub fn block_address(&self, index: usize) -> Result<*const (), Failure> {
        Ok(self.block_at(index)?.cast())
    }
    /// Returns a mutable untyped block address; `storage.change_block_address(0)?`.
    /// Unsafe caller access must respect allocation lifetime, initialization and aliasing.
    pub fn change_block_address(&mut self, index: usize) -> Result<*mut (), Failure> {
        Ok(self.change_block_at(index)?.cast())
    }
    /// Returns a typed const pointer to the allocated block, including empty capacity.
    /// `storage.block_at(0)?`; a pointer is not proof that any element is initialized.
    pub fn block_at(&self, index: usize) -> Result<*const T, Failure> {
        self.validate_block_index(index)?;
        Ok(self.array.blocks[index].as_ptr())
    }
    /// Returns a typed mutable pointer; `storage.change_block_at(0)?`.
    /// A raw write requires the caller to manage initialization and preserve ownership invariants.
    pub fn change_block_at(&mut self, index: usize) -> Result<*mut T, Failure> {
        self.validate_block_index(index)?;
        Ok(self.array.blocks[index].as_mut_ptr())
    }
    /// Reads an initialized slot; `storage.at(0)?` preserves the checked Rust boundary.
    pub fn at(&self, index: usize) -> Result<&T, ArrayError> {
        self.array.value(index)
    }
    /// Mutates an initialized slot; `*storage.change_at(0)? = 42`.
    pub fn change_at(&mut self, index: usize) -> Result<&mut T, ArrayError> {
        self.array.change_value(index)
    }
    /// Frees blocks without running element destructors, exactly as native clearBlocks.
    /// `storage.clear_blocks()` is distinct from `array.clear(true)` which drops values.
    /// OCCT 3d097a0 uses this after destruction; direct use can intentionally leak element resources.
    pub fn clear_blocks(&mut self) {
        for block in &mut self.array.blocks {
            forget_block_values_after(block, 0);
        }
        self.array.blocks.clear();
        self.array.used = 0;
    }
    fn validate_item_size(&self, item_size: usize) -> Result<(), Failure> {
        let expected = std::mem::size_of::<T>();
        if item_size == expected && expected > 0 {
            return Ok(());
        }
        Err(Failure::with_type(
            "Standard_DimensionMismatch",
            &format!("item size {item_size}: expected nonzero typed size {expected}"),
            "",
        ))
    }
    fn validate_block_count(&self, count: usize) -> Result<(), Failure> {
        if count.checked_mul(self.array.increment).is_some() {
            return Ok(());
        }
        Err(self.storage_size_failure(count))
    }
    fn storage_size_failure(&self, value: usize) -> Failure {
        Failure::with_type(
            "Standard_OutOfRange",
            &format!(
                "storage count/index {value}: expected count * stride {} within usize",
                self.array.increment
            ),
            "",
        )
    }
    fn validate_block_index(&self, index: usize) -> Result<(), Failure> {
        if index < self.block_count() {
            return Ok(());
        }
        Err(Failure::with_type(
            "Standard_OutOfRange",
            &format!(
                "block index {index}: expected allocated range [0, {})",
                self.block_count()
            ),
            "",
        ))
    }
}
