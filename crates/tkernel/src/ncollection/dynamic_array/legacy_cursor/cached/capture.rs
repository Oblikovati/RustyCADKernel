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
    if array.increment == 0 || size_of::<T>() == 0 {
        return Err(capture_failure(index, 0, 0, 0));
    }
    let block_index = index / array.increment;
    let offset = index % array.increment;
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
fn capture_failure(index: usize, stride: usize, initialized: usize, capacity: usize) -> Failure {
    let reason = format!(
        "cached index {index}, stride {stride}, initialized {initialized}, capacity {capacity}: expected a nonzero-sized live initialized slot and fitting nonzero-stride block end"
    );
    Failure::with_type("Standard_DomainError", &reason, "")
}
