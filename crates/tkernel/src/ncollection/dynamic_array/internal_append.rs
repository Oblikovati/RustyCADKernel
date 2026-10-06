//! Index-based internal sources preserve native self-append while respecting Rust exclusive access.
use super::{ArrayError, DynamicArray, checked_index};
use std::fmt::Debug;

impl<T> DynamicArray<T> {
    /// Appends a copy of an existing logical item after allocating its destination block.
    /// `array.append_from_index_with_constructor(0, |source| *source)?`.
    /// This represents native Append(array[index]); no Clone or Default bound is required.
    pub fn append_from_index_with_constructor<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        source_index: I,
        copy_construct: impl FnOnce(&T) -> T,
    ) -> Result<&mut T, ArrayError> {
        let source_index = self.logical_source_index(source_index)?;
        Ok(self.append_constructed_with(|array| {
            let source =
                &array.blocks[source_index / array.increment][source_index % array.increment];
            copy_construct(source)
        }))
    }

    /// Move-constructs an appended item from an existing logical donor that remains in the array.
    /// `array.append_moved_from_index_with_constructor(0, std::mem::take)?`.
    /// This represents native Append(std::move(array[index])). A constructor panic retains
    /// the old logical size and any donor mutation; its block allocation is retained for retry.
    pub fn append_moved_from_index_with_constructor<I: TryInto<usize> + Copy + Debug>(
        &mut self,
        source_index: I,
        move_construct: impl FnOnce(&mut T) -> T,
    ) -> Result<&mut T, ArrayError> {
        let source_index = self.logical_source_index(source_index)?;
        Ok(self.append_constructed_with(|array| {
            let source =
                &mut array.blocks[source_index / array.increment][source_index % array.increment];
            move_construct(source)
        }))
    }

    pub(super) fn logical_source_index<I: TryInto<usize> + Copy + Debug>(
        &self,
        source_index: I,
    ) -> Result<usize, ArrayError> {
        let source_index = checked_index(source_index)?;
        if source_index >= self.used {
            return Err(self.index_error(source_index));
        }
        // Safe Rust can retain holes after a caught destructor panic; reject an uninitialized source.
        self.value(source_index)?;
        Ok(source_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn internal_sources_survive_full_descriptor_table_growth() {
        let mut array = DynamicArray::with_increment(2);
        for value in 0..4 {
            array.append(value);
        }
        assert_eq!(array.blocks.len(), array.blocks.capacity());
        let addresses: Vec<*const i32> = array.blocks.iter().map(|block| block.as_ptr()).collect();
        array
            .append_moved_from_index_with_constructor(3, std::mem::take)
            .unwrap();
        assert_eq!(array.blocks.len(), 3);
        assert_eq!(array.blocks.capacity(), 4);
        for (index, address) in addresses.into_iter().enumerate() {
            assert_eq!(array.blocks[index].as_ptr(), address);
        }
        assert_eq!(array.iter().copied().collect::<Vec<_>>(), [0, 1, 2, 0, 3]);
    }
}
