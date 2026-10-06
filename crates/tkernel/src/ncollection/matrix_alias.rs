//! Native Map overloads borrow the whole matrix interface without copying its storage.
use super::{Mat3, Mat4};

macro_rules! matrix_alias_contract {
    ($matrix:ident, $count:literal) => {
        impl<T> $matrix<T> {
            /// Borrows a flat array as a complete matrix; `Mat3::map_ref(&values).determinant()`.
            pub fn map_ref(elements: &[T; $count]) -> &Self {
                // SAFETY: repr(transparent) over an array of equal contiguous columns
                // guarantees identical size, alignment and valid T representations.
                unsafe { &*std::ptr::from_ref(elements).cast::<Self>() }
            }

            /// Borrows a writable array as a complete matrix; `Mat4::map_ref_mut(&mut values).transpose()`.
            pub fn map_ref_mut(elements: &mut [T; $count]) -> &mut Self {
                // SAFETY: the transparent layout equals the flat array; the exclusive
                // input borrow is retained for the entire lifetime of the result.
                unsafe { &mut *std::ptr::from_mut(elements).cast::<Self>() }
            }

            /// Returns GetData's column-major pointer; `matrix.data_ptr()`.
            pub fn data_ptr(&self) -> *const T {
                self.as_slice().as_ptr()
            }

            /// Returns ChangeData's column-major pointer; `matrix.data_mut_ptr()`.
            pub fn data_mut_ptr(&mut self) -> *mut T {
                self.as_mut_slice().as_mut_ptr()
            }
        }
    };
}

matrix_alias_contract!(Mat3, 9);
matrix_alias_contract!(Mat4, 16);
