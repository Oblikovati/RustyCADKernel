// Rust translation of FoundationClasses/TKMath/BVH/BVH_Types.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH::Array<T, N>`: the typical operations on BVH arrays of vectors.
//!
//! Indices are `int` upstream and convert to `size_t`, so a negative index is a
//! huge one; `NCollection_LinearVector::Value` then raises `Standard_OutOfRange`.
use rustycad_tkernel::ncollection::LinearVector;
use rustycad_tkernel::standard::Failure;

fn out_of_range<E>(array: &LinearVector<E>, operation: &str, index: i32) -> Failure {
    Failure::out_of_range(&format!(
        "NCollection_LinearVector::{operation}: index {index}, expected 0..{}",
        array.len()
    ))
}

/// Borrows the element at `index`; `array::value(&tree_buffer, 0)?`.
pub fn value<E>(array: &LinearVector<E>, index: i32) -> Result<&E, Failure> {
    usize::try_from(index)
        .ok()
        .and_then(|offset| array.as_slice().get(offset))
        .ok_or_else(|| out_of_range(array, "Value", index))
}

/// Mutably borrows the element at `index`; `*array::change_value(&mut buffer, 0)? = v`.
pub fn change_value<E>(array: &mut LinearVector<E>, index: i32) -> Result<&mut E, Failure> {
    let failure = out_of_range(array, "ChangeValue", index);
    usize::try_from(index)
        .ok()
        .and_then(|offset| array.as_mut_slice().get_mut(offset))
        .ok_or(failure)
}

/// Adds the new element at the end; `array::append(&mut vertices, Vec3::new(0.0, 0.0, 0.0))`.
pub fn append<E>(array: &mut LinearVector<E>, element: E) {
    array.append(element);
}

/// Returns the number of elements as `int`; `array::size(&vertices) == 3`.
pub fn size<E>(array: &LinearVector<E>) -> i32 {
    array.len() as i32
}

/// Removes all elements, keeping capacity (`Clear()` default); `array::clear(&mut vertices)`.
pub fn clear<E>(array: &mut LinearVector<E>) {
    array.clear(false);
}

/// Requests capacity for at least `count` elements; `array::reserve(&mut vertices, 8)?`.
///
/// A negative count converts to a huge `size_t` upstream and the allocation fails.
pub fn reserve<E>(array: &mut LinearVector<E>, count: i32) -> Result<(), Failure> {
    let capacity = usize::try_from(count).map_err(|_| {
        Failure::with_type(
            "Standard_OutOfMemory",
            &format!("NCollection_LinearVector::Reserve: capacity {count}, expected >= 0"),
            "",
        )
    })?;
    array.reserve(capacity);
    Ok(())
}
