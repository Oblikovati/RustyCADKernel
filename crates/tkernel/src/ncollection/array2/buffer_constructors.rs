use super::{Array2, ArrayError, extent};

impl<'a, T> Array2<'a, T> {
    /// Borrows a bounded row-major prefix, including the native flat Array1 bounds.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Array2;
    /// let mut values = [3, 4, 5, 6, 99];
    /// let mut array = Array2::from_buffer_bounds(&mut values, -1, 0, 3, 4).unwrap();
    /// array[(-1, 3)] = 7;
    /// assert_eq!(array.array1().lower(), 1);
    /// assert_eq!(array.as_slice(), [7, 4, 5, 6]);
    /// ```
    pub fn from_buffer_bounds(
        buffer: &'a mut [T],
        row_lower: i32,
        row_upper: i32,
        col_lower: i32,
        col_upper: i32,
    ) -> Result<Self, ArrayError> {
        let rows = extent(row_lower, row_upper)?;
        let cols = extent(col_lower, col_upper)?;
        let mut array = Self::from_buffer(buffer, rows, cols)?;
        array.rebase(row_lower, col_lower);
        Ok(array)
    }
}
