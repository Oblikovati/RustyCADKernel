//! Owned `NCollection_Array1` copies with explicit bounds, as the Poly constructors make them.
use rustycad_tkernel::ncollection::Array1;
/// Copies `values` into a new owned array starting at `lower`; `owned_array(&[1, 2], 1).upper() == 2`.
pub(crate) fn owned_array<T: Clone + Default>(values: &[T], lower: i32) -> Array1<'static, T> {
    let mut array = Array1::with_size(values.len());
    array.as_mut_slice().clone_from_slice(values);
    array.update_lower_bound(lower);
    array
}
