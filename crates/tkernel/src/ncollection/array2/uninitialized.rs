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

#[allow(unsafe_code)]
impl<'a, T> Array2<'a, MaybeUninit<T>> {
    /// Transfers exclusive initialized backing while retaining all rectangle metadata.
    /// Shared backing returns the original matrix unchanged in `Err`.
    /// ```
    /// use rustycad_tkernel::ncollection::Array2;
    /// let mut slots = Array2::<String>::with_uninit_size(1, 1);
    /// slots.as_mut_slice()[0].write("ready".into());
    /// let values = unsafe { slots.try_assume_init_exclusive() }.unwrap_or_else(|_| panic!("shared"));
    /// assert_eq!(values.as_slice(), ["ready"]);
    /// ```
    ///
    /// # Safety
    /// Every actual base-array slot must contain a valid initialized `T` for owned
    /// or exclusively borrowed backing, even if its length differs from rows times
    /// columns. Spare capacity and unborrowed tails are excluded. Owned payloads drop
    /// with the result; borrowed payloads remain the donor's responsibility. Retained
    /// pointers must obey the resulting borrow. Shared slots are neither read nor
    /// converted, so rejected shared backing may remain uninitialized.
    pub unsafe fn try_assume_init_exclusive(self) -> Result<Array2<'a, T>, Self> {
        let rectangle = (self.row_lower, self.col_lower, self.rows, self.cols);
        // SAFETY: the caller's obligation covers every actual inherited storage slot.
        unsafe { self.elements.try_assume_init_exclusive() }
            .map(|elements| initialization_rectangle(elements, rectangle))
            .map_err(|elements| initialization_rectangle(elements, rectangle))
    }
}

fn initialization_rectangle<T>(
    elements: Array1<'_, T>,
    rectangle: (i32, i32, usize, usize),
) -> Array2<'_, T> {
    Array2 {
        elements,
        row_lower: rectangle.0,
        col_lower: rectangle.1,
        rows: rectangle.2,
        cols: rectangle.3,
    }
}
