//! Matrix metadata is independent of the initialization state of its flat storage.
use super::{Array2, ArrayError, extent};
use crate::ncollection::Array1;
use std::mem::MaybeUninit;

impl<'a, T> Array2<'a, T> {
    /// Allocates a rectangle without constructing its elements.
    /// `let slots = Array2::<i32>::with_uninit_size(2, 3); assert_eq!(slots.len(), 6);`.
    pub fn with_uninit_size(rows: usize, cols: usize) -> Array2<'a, MaybeUninit<T>> {
        let length = rows.checked_mul(cols).unwrap_or_else(|| {
            panic!("array dimensions {rows} x {cols}: expected a product fitting usize")
        });
        Array2 {
            elements: Array1::<T>::with_uninit_size(length),
            row_lower: 0,
            col_lower: 0,
            rows,
            cols,
        }
    }

    /// Allocates inclusive matrix bounds with explicitly uninitialized elements.
    /// `let slots = Array2::<i32>::with_uninit_bounds(-1, 0, 3, 5)?;`.
    pub fn with_uninit_bounds(
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
    ) -> Result<Array2<'a, MaybeUninit<T>>, ArrayError> {
        let mut array = Self::with_uninit_size(extent(rl, ru)?, extent(cl, cu)?);
        array.rebase(rl, cl);
        Ok(array)
    }
}

#[allow(unsafe_code)]
impl<'a, T: Copy> Array2<'a, MaybeUninit<T>> {
    /// Changes element initialization state without moving storage or rectangle bounds.
    /// After writing every flat slot, use `let values = unsafe { slots.assume_init() };`.
    ///
    /// # Safety
    /// Every slot in `self.array1().as_slice()` must contain a valid initialized `T`.
    /// The actual flat length may differ from the rectangle after base-array replacement
    /// or resize; every actual slot, including those outside the rectangle, is required.
    /// Retained pointers must obey the resulting borrows; spare capacity is not exposed.
    pub unsafe fn assume_init(self) -> Array2<'a, T> {
        Array2 {
            // SAFETY: the caller's obligation covers the complete inherited storage.
            elements: unsafe { self.elements.assume_init() },
            row_lower: self.row_lower,
            col_lower: self.col_lower,
            rows: self.rows,
            cols: self.cols,
        }
    }
}
