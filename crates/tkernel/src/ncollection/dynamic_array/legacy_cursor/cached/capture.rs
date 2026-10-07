use super::{DynamicArray, DynamicArrayCachedIterator, DynamicArrayIterator, Failure};
impl<T> DynamicArrayCachedIterator<T, false> {
    /// Captures a shared pointer and native iteration-end snapshot; no Clone bound is required.
    /// `let cursor = DynamicArrayCachedIterator::from_shared(&array, 0)?;`.
    pub fn from_shared(array: &DynamicArray<T>, index: usize) -> Result<Self, Failure> {
        let position = DynamicArrayIterator::new(array, index);
        let (current, block_end) = capture_shared(array, index, position.end)?;
        Ok(Self {
            position,
            current,
            block_end,
        })
    }
}
impl<T> DynamicArrayCachedIterator<T, true> {
    /// Captures writable raw provenance without retaining an exclusive container borrow.
    /// `let cursor = DynamicArrayCachedIterator::from_mutable(&mut array, 0)?;`.
    pub fn from_mutable(array: &mut DynamicArray<T>, index: usize) -> Result<Self, Failure> {
        let position = DynamicArrayIterator::new(array, index);
        let (current, block_end) = capture_mutable(array, index, position.end)?;
        Ok(Self {
            position,
            current,
            block_end,
        })
    }
}
pub(super) fn capture_shared<T>(
    array: &DynamicArray<T>,
    index: usize,
    end: usize,
) -> Result<(*mut T, *mut T), Failure> {
    if index >= end {
        return Ok((std::ptr::null_mut(), std::ptr::null_mut()));
    }
    let (block, offset) = live_coordinates(array, index)?;
    let start = array.blocks[block].as_ptr();
    // Shared pointers are stored uniformly, but only the MUTABLE=true constructor exposes writes.
    Ok((
        start.wrapping_add(offset).cast_mut(),
        start.wrapping_add(array.increment).cast_mut(),
    ))
}
pub(super) fn capture_mutable<T>(
    array: &mut DynamicArray<T>,
    index: usize,
    end: usize,
) -> Result<(*mut T, *mut T), Failure> {
    if index >= end {
        return Ok((std::ptr::null_mut(), std::ptr::null_mut()));
    }
    let (block, offset) = live_coordinates(array, index)?;
    // Do not narrow writable provenance to &mut T: navigation may enter another slot in this block.
    let start = array.blocks[block].as_mut_ptr();
    Ok((
        start.wrapping_add(offset),
        start.wrapping_add(array.increment),
    ))
}
fn live_coordinates<T>(array: &DynamicArray<T>, index: usize) -> Result<(usize, usize), Failure> {
    validate_capture_shape(array, index)?;
    let (block_index, offset) = array.element_coordinates(index);
    let Some(block) = array.blocks.as_slice().get(block_index) else {
        return Err(capture_failure(index, array.increment, 0, 0));
    };
    if offset >= block.len() || array.increment > block.capacity() {
        return Err(capture_failure(
            index,
            array.increment,
            block.len(),
            block.capacity(),
        ));
    }
    Ok((block_index, offset))
}
fn validate_capture_shape<T>(array: &DynamicArray<T>, index: usize) -> Result<(), Failure> {
    let unsupported_zero =
        array.increment == 0 && (index != 0 || array.validate_retained_erasure().is_err());
    if size_of::<T>() == 0 || unsupported_zero {
        return Err(capture_failure(index, array.increment, 0, 0));
    }
    Ok(())
}
fn capture_failure(index: usize, stride: usize, initialized: usize, capacity: usize) -> Failure {
    let reason = format!(
        "cached index {index}, stride {stride}, initialized {initialized}, capacity {capacity}: expected a nonzero-sized live initialized slot and fitting block end (zero stride permits only the registered retained first slot)"
    );
    Failure::with_type("Standard_DomainError", &reason, "")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_stride_cache_records_native_empty_block_extent_for_live_first_slot() {
        let mut array = DynamicArray::with_increment(2);
        array.append(1);
        array.clear(false);
        array.set_increment(usize::MAX);
        array.emplace_value(0_usize, || 7).unwrap();
        let cursor = DynamicArrayCachedIterator::from_shared(&array, 0).unwrap();
        assert_eq!(cursor.current, cursor.block_end);
        assert!(!cursor.current.is_null());
        let end = DynamicArrayCachedIterator::from_shared(&array, 1).unwrap();
        assert!(end.current.is_null());
        assert!(end.block_end.is_null());
    }
}
