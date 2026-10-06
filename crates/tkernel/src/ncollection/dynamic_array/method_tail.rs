//! Native signed overloads, borrowed construction and explicit self-alias adapters.
use super::DynamicArray;
use crate::ncollection::{ArrayError, BaseAllocator, OccAllocator};
use std::fmt::Debug;
impl<T> DynamicArray<T> {
    /// Reports native size_t Size, without truncating to Length; `array.size() == array.len()`.
    pub const fn size(&self) -> usize {
        self.len()
    }
    /// Clamps the native signed increment below one; `DynamicArray::<i32>::with_increment_i32(-4)`.
    pub fn with_increment_i32(increment: i32) -> Self {
        Self::with_increment(increment.max(1) as usize)
    }
    /// Uses the signed constructor with an explicit resource; `DynamicArray::<i32>::with_allocator_i32(-1, None)`.
    pub fn with_allocator_i32(increment: i32, allocator: Option<BaseAllocator>) -> Self {
        Self::with_allocator(increment.max(1) as usize, allocator)
    }
    /// Uses the signed typed-allocator constructor; `DynamicArray::with_occ_allocator_i32(2, &typed)`.
    pub fn with_occ_allocator_i32(increment: i32, allocator: &OccAllocator<T>) -> Self {
        Self::with_occ_allocator(increment.max(1) as usize, allocator)
    }
    /// Clamps the signed SetIncrement overload before the empty-array check; `array.set_increment_i32(-2)`.
    pub fn set_increment_i32(&mut self, increment: i32) {
        self.set_increment(increment.max(1) as usize);
    }
    /// Preserves all storage on native Assign(self, policy); `array.assign_self(true)`.
    pub fn assign_self(&mut self, _own_allocator: bool) -> &mut Self {
        self
    }
    /// Preserves all storage on native copy self-assignment; `array.assign_operator_self()`.
    pub fn assign_operator_self(&mut self) -> &mut Self {
        self
    }
    /// Preserves all storage on native move self-assignment; `array.move_self()`.
    pub fn move_self(&mut self) -> &mut Self {
        self
    }
}
impl<T: Clone> DynamicArray<T> {
    /// Reserves the element block before invoking the borrowed copy; `array.append_ref(&value)`.
    pub fn append_ref(&mut self, value: &T) -> &mut T {
        self.append_ref_with_constructor(value, T::clone)
    }
}
impl<T: Clone + Default> DynamicArray<T> {
    /// Reserves all blocks and constructs gaps before copying the requested value; `array.set_value_ref(4, &value)?`.
    pub fn set_value_ref<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        index: I,
        value: &T,
    ) -> Result<&mut T, ArrayError> {
        self.emplace_value(index, || value.clone())
    }
}
