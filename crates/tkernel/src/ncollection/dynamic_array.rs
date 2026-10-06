//! Segmented storage whose existing element addresses survive append.
use super::{ArrayError, BaseAllocator, LinearVector};
use allocator_api2::vec::Vec as AllocVec;
use std::fmt::Debug;

/// Stable-block storage with zero-based indexing; `DynamicArray::<i32>::with_increment(256)`.
pub struct DynamicArray<T> {
    blocks: LinearVector<AllocVec<T, BaseAllocator>>,
    allocator: Option<BaseAllocator>,
    increment: usize,
    used: usize,
    // False proves canonical registered prefixes; uncertain states retain the exact scan.
    needs_tail_scan: bool,
}
/// A detached index that survives appends; `array.begin_cursor().value(&array)?`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DynamicCursor {
    index: usize,
}
impl<T> Default for DynamicArray<T> {
    fn default() -> Self {
        Self::with_increment(256)
    }
}
mod block_table;
mod borrowed;
mod copy_date;
mod initialized_tail;
pub use copy_date::DynamicArrayCopyFailure;
mod emplacement;
mod insertion;
mod legacy_cursor;
mod method_tail;
mod storage_extension;
pub use legacy_cursor::DynamicArrayIterator;
pub use storage_extension::DynamicArrayStorageExtension;
impl<T> DynamicArray<T> {
    /// Selects initial block size, rounded up to a power of two; `DynamicArray::<i32>::with_increment(5)`.
    pub fn with_increment(increment: usize) -> Self {
        Self::with_allocator(increment, None)
    }
    /// Selects the allocator that owns every element block; `DynamicArray::<i32>::with_allocator(256,Some(alloc))`.
    pub fn with_allocator(increment: usize, allocator: Option<BaseAllocator>) -> Self {
        Self {
            blocks: LinearVector::default(),
            allocator,
            increment: increment.max(1).next_power_of_two(),
            used: 0,
            needs_tail_scan: false,
        }
    }
    /// Keeps the resource of a typed allocator, as the native `allocator_type` constructor
    /// stores `theAllocator.Allocator()`; `DynamicArray::with_occ_allocator(256, &OccAllocator::default())`.
    pub fn with_occ_allocator(increment: usize, allocator: &super::OccAllocator<T>) -> Self {
        Self::with_allocator(increment, allocator.allocator().cloned())
    }
    /// Returns allocator identity, preserving the null/default distinction; `array.allocator()`.
    pub fn allocator(&self) -> Option<&BaseAllocator> {
        self.allocator.as_ref()
    }
    /// Returns the number of logical elements; `DynamicArray::<i32>::default().len() == 0`.
    pub const fn len(&self) -> usize {
        self.used
    }
    /// Returns OCCT's legacy signed count; `DynamicArray::<i32>::default().length() == 0_i32`.
    pub const fn length(&self) -> i32 {
        self.used as i32
    }
    /// Checks the logical range; `DynamicArray::<i32>::default().is_empty()`.
    pub const fn is_empty(&self) -> bool {
        self.used == 0
    }
    /// Returns zero, the indexing origin; `array.lower() == 0`.
    pub const fn lower(&self) -> i32 {
        0
    }
    /// Returns the inclusive last index; `DynamicArray::<i32>::default().upper() == -1`.
    pub const fn upper(&self) -> i32 {
        (self.used as i32).wrapping_sub(1)
    }
    /// Changes block size while empty; `array.set_increment(512)`.
    pub fn set_increment(&mut self, increment: usize) {
        if !self.is_empty() {
            return;
        }
        // OCCT keeps allocated blocks when changing the stride of an empty array.
        self.increment = increment.max(1).next_power_of_two();
    }
    /// Appends without moving existing elements; `array.append(42)`.
    pub fn append(&mut self, value: T) -> &mut T {
        self.emplace_append(|| value)
    }
    /// Reserves storage before constructing the appended value; `array.emplace_append(||42)`.
    pub fn emplace_append(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        let index = self.used;
        let block = index / self.increment;
        let offset = index % self.increment;
        if block == self.blocks.len() {
            self.add_block();
        }
        let value = construct();
        if offset < self.blocks[block].len() {
            self.blocks[block][offset] = value;
        } else {
            self.blocks[block].push(value);
        }
        self.used += 1;
        &mut self.blocks[block][offset]
    }
    fn ensure_storage_for_index(&mut self, index: usize) {
        self.reserve_block_count(index / self.increment + 1);
    }
    /// Reads an initialized slot with either i32 or usize indexing; `array.value(0_usize)?`.
    pub fn value<I: TryInto<usize> + Copy + Debug>(&self, index: I) -> Result<&T, ArrayError> {
        let index = checked_index(index)?;
        self.blocks
            .as_slice()
            .get(index / self.increment)
            .and_then(|block| block.get(index % self.increment))
            .ok_or_else(|| self.index_error(index))
    }
    /// Mutates an initialized slot; `*array.change_value(0)? = 42`.
    pub fn change_value<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
    ) -> Result<&mut T, ArrayError> {
        let index = checked_index(index)?;
        let error = self.index_error(index);
        self.blocks
            .as_mut_slice()
            .get_mut(index / self.increment)
            .and_then(|block| block.get_mut(index % self.increment))
            .ok_or(error)
    }
    /// Reads the first value; `array.first()?`.
    pub fn first(&self) -> Result<&T, ArrayError> {
        self.value(0)
    }
    /// Reads the last logical value; `array.last()?`.
    pub fn last(&self) -> Result<&T, ArrayError> {
        self.value(self.used.wrapping_sub(1))
    }
    /// Mutates the first value; `*array.change_first()? = 42`.
    pub fn change_first(&mut self) -> Result<&mut T, ArrayError> {
        self.change_value(0)
    }
    /// Mutates the last logical value; `*array.change_last()? = 42`.
    pub fn change_last(&mut self) -> Result<&mut T, ArrayError> {
        self.change_value(self.used.wrapping_sub(1))
    }
    /// Removes the last logical value, retaining its block; `array.erase_last()`.
    pub fn erase_last(&mut self) {
        if self.used == 0 {
            return;
        }
        let previous_scan = self.needs_tail_scan;
        self.needs_tail_scan = true;
        self.used -= 1;
        self.blocks[self.used / self.increment].truncate(self.used % self.increment);
        self.needs_tail_scan = previous_scan;
    }
    /// Drops logical elements, optionally releasing blocks; `array.clear(false)`.
    pub fn clear(&mut self, release: bool) {
        self.needs_tail_scan = true;
        // Native Clear destroys every logical element before clearBlocks frees any block.
        for block in &mut self.blocks {
            block.clear();
        }
        if release {
            self.blocks.clear(true);
        }
        self.used = 0;
        self.needs_tail_scan = false;
    }
    /// Transfers blocks and empties the source; `dest.move_from(&mut source)`.
    pub fn move_from(&mut self, source: &mut Self) {
        // Named native Assign(rvalue) destroys the receiver before touching the source.
        self.clear(true);
        let empty = Self::with_increment(source.increment);
        *self = std::mem::replace(source, empty);
    }
    /// Iterates the logical sequence; `array.iter().sum::<i32>()`.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator {
        (0..self.used).map(|index| &self[index])
    }
    /// Mutably iterates only logical elements; `array.iter_mut().for_each(|v| *v *= 2)`.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.blocks
            .iter_mut()
            .flat_map(|block| block.iter_mut())
            .take(self.used)
    }
    /// Returns a detached first position; `array.begin_cursor()`.
    pub const fn begin_cursor(&self) -> DynamicCursor {
        DynamicCursor { index: 0 }
    }
    /// Returns a snapshot of the current end; `array.end_cursor()`.
    pub const fn end_cursor(&self) -> DynamicCursor {
        DynamicCursor { index: self.used }
    }
    /// Reverses values while retaining block storage; `array.reverse()`.
    pub fn reverse(&mut self) {
        for index in 0..self.used / 2 {
            self.swap(index, self.used - 1 - index);
        }
    }
    fn add_block(&mut self) {
        self.blocks.append(AllocVec::with_capacity_in(
            self.increment,
            self.allocator.clone().unwrap_or_default(),
        ));
    }
    fn index_error(&self, index: usize) -> ArrayError {
        ArrayError::Index {
            index: index as i64,
            lower: 0,
            upper: self.used as i64 - 1,
        }
    }
    fn swap(&mut self, first: usize, second: usize) {
        let (first, second) = (first.min(second), first.max(second));
        let (block1, block2) = (first / self.increment, second / self.increment);
        if block1 == block2 {
            self.blocks[block1].swap(first % self.increment, second % self.increment);
            return;
        }
        let (left, right) = self.blocks.as_mut_slice().split_at_mut(block2);
        std::mem::swap(
            &mut left[block1][first % self.increment],
            &mut right[0][second % self.increment],
        );
    }
}
impl<T: Default> DynamicArray<T> {
    /// Appends a default value; `*array.appended() = 42`.
    pub fn appended(&mut self) -> &mut T {
        self.emplace_append(T::default)
    }
    /// Overwrites or extends, default-constructing gaps; `array.set_value(5,42)?`.
    pub fn set_value<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        value: T,
    ) -> Result<&mut T, ArrayError> {
        self.emplace_value(index, || value)
    }
    fn allocated_slot(&mut self, index: usize) -> &mut T {
        self.allocated_slot_with(index, T::default)
    }
}
impl<T: Clone> Clone for DynamicArray<T> {
    fn clone(&self) -> Self {
        let mut copy = Self::with_allocator(self.increment, self.allocator.clone());
        copy.copy_blocks_from(self);
        copy
    }
}
impl<T: Clone> DynamicArray<T> {
    /// Copies contents and optionally keeps destination allocator; `dest.assign(&source,true)`.
    pub fn assign(&mut self, source: &Self, own_allocator: bool) {
        self.clear(true);
        self.increment = source.increment;
        if !own_allocator {
            self.allocator = source.allocator.clone();
        }
        self.copy_blocks_from(source);
    }

    /// Copies values and returns the destination for native `Assign`-style chaining; `dest.assign_returning(&source,false)`.
    pub fn assign_returning(&mut self, source: &Self, own_allocator: bool) -> &mut Self {
        self.assign(source, own_allocator);
        self
    }

    /// Uses native `Assign`'s default allocator policy and returns this array; `dest.assign_default_returning(&source)`.
    pub fn assign_default_returning(&mut self, source: &Self) -> &mut Self {
        self.assign_returning(source, true)
    }

    /// Copies values and adopts the source allocator, matching copy `operator=`; `dest.assign_operator_returning(&source)`.
    pub fn assign_operator_returning(&mut self, source: &Self) -> &mut Self {
        self.assign_returning(source, false)
    }

    fn copy_blocks_from(&mut self, source: &Self) {
        self.copy_date_using(source, |value| Ok(value.clone()))
            .unwrap_or_else(|error| {
                panic!("DynamicArray copy storage: expected valid source: {error:?}")
            });
    }
}
impl<T> DynamicArray<T> {
    /// Transfers blocks and returns the destination for native move-assignment chaining; `dest.move_from_returning(&mut source)`.
    pub fn move_from_returning(&mut self, source: &mut Self) -> &mut Self {
        self.move_from(source);
        self
    }
}
impl<T: Ord> DynamicArray<T> {
    /// Sorts the logical sequence without replacing its blocks; `array.sort()`.
    pub fn sort(&mut self) {
        for index in 1..self.used {
            let mut position = index;
            while position > 0 && self[position] < self[position - 1] {
                self.swap(position, position - 1);
                position -= 1;
            }
        }
    }
}
impl<T> std::ops::Index<usize> for DynamicArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.value(index).unwrap_or_else(|error| panic!("{error}"))
    }
}
impl<T: Default> std::ops::IndexMut<usize> for DynamicArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.allocated_slot(index)
    }
}
impl DynamicCursor {
    /// Reads this position from an array after any appends; `cursor.value(&array)?`.
    pub fn value<T>(self, array: &DynamicArray<T>) -> Result<&T, ArrayError> {
        array.value(self.index)
    }
}
impl std::ops::Add<usize> for DynamicCursor {
    type Output = Self;
    fn add(self, offset: usize) -> Self {
        Self {
            index: self.index + offset,
        }
    }
}
impl std::ops::Sub<usize> for DynamicCursor {
    type Output = Self;
    fn sub(self, offset: usize) -> Self {
        Self {
            index: self.index - offset,
        }
    }
}
fn checked_index<I: TryInto<usize> + Copy + Debug>(index: I) -> Result<usize, ArrayError> {
    index.try_into().map_err(|_| ArrayError::Index {
        index: format!("{index:?}").parse().unwrap_or(i64::MAX),
        lower: 0,
        upper: i64::MAX,
    })
}
