// Rust translation of the NCollection_Array1 allocations and copies of the Convert package at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Array allocation and copy helpers shared by the Convert classes.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// `NCollection_Array1(1, count)`; a negative size is the native allocation failure;
/// `one_based::<f64>(3)?`.
pub(super) fn one_based<T: Default>(count: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, count).map_err(|error| {
        Failure::with_type(
            "Standard_OutOfMemory",
            &format!("Convert: cannot allocate an array 1..{count}: {error}"),
            "",
        )
    })
}

/// Owned copy keeping the bounds (the NCollection_Array1 copy constructor);
/// `owned_copy(&knots).lower() == knots.lower()`.
pub(super) fn owned_copy<T: Copy + Default>(array: &Array1<'_, T>) -> Array1<'static, T> {
    let mut copy: Array1<'static, T> = Array1::with_size(array.len());
    copy.update_lower_bound(array.lower());
    copy.as_mut_slice().copy_from_slice(array.as_slice());
    copy
}
