// The static layout helpers and the int-typed size queries of NCollection_Array2.
// Native `int` arithmetic is undefined on overflow; Rust wraps, which is what the
// native build computes on every supported target.

impl<T> Array2<'_, T> {
    /// Element count of a `rows x cols` rectangle; `Array2::<i32>::array_size(2, 3) == 6`.
    pub const fn array_size(rows: usize, cols: usize) -> usize {
        rows.wrapping_mul(cols)
    }

    /// Element count of inclusive bounds, as [`Array1::range_size`];
    /// `Array2::<i32>::range_size(1, 3) == 3`.
    pub const fn range_size(lower: i32, upper: i32) -> usize {
        Array1::<T>::range_size(lower, upper)
    }

    /// Upper bound of `size` elements from `lower`; `Array2::<i32>::upper_for_size(1, 3) == 3`.
    pub const fn upper_for_size(lower: i32, size: usize) -> i32 {
        lower.wrapping_add(size as i32).wrapping_sub(1)
    }

    /// Flat lower bound of the underlying Array1, `ColLower + RowLower * NbCols`;
    /// `Array2::<i32>::begin_position(1, 2, 1, 3) == 4`.
    pub const fn begin_position(row_lower: i32, _row_upper: i32, col_lower: i32, col_upper: i32) -> i32 {
        col_lower.wrapping_add(row_lower.wrapping_mul(col_upper.wrapping_sub(col_lower).wrapping_add(1)))
    }

    /// Flat upper bound of the underlying Array1; `Array2::<i32>::last_position(1, 2, 1, 3) == 9`.
    pub const fn last_position(row_lower: i32, row_upper: i32, col_lower: i32, col_upper: i32) -> i32 {
        let cols = col_upper.wrapping_sub(col_lower).wrapping_add(1);
        let rows = row_upper.wrapping_sub(row_lower).wrapping_add(1);
        rows.wrapping_mul(cols)
            .wrapping_add(col_lower)
            .wrapping_add(row_lower.wrapping_mul(cols))
            .wrapping_sub(1)
    }

    /// OCCT's signed element count `NbRows() * NbColumns()`; `Array2::<i32>::with_size(2, 3).length() == 6`.
    pub fn length(&self) -> i32 {
        (self.rows as i32).wrapping_mul(self.cols as i32)
    }

    /// Row count as `size_t`; `Array2::<i32>::with_size(2, 3).row_size() == 2`.
    pub const fn row_size(&self) -> usize {
        self.rows
    }

    /// Column count as `size_t`; `Array2::<i32>::with_size(2, 3).col_size() == 3`.
    pub const fn col_size(&self) -> usize {
        self.cols
    }

    /// Length of one row, i.e. the column count; `Array2::<i32>::with_size(2, 3).row_length() == 3`.
    pub fn row_length(&self) -> i32 {
        self.cols as i32
    }

    /// Length of one column, i.e. the row count; `Array2::<i32>::with_size(2, 3).col_length() == 2`.
    pub fn col_length(&self) -> i32 {
        self.rows as i32
    }

    /// Reads a flat zero-based offset of the row-major storage; `array.at_offset(0)?`.
    pub fn at_offset(&self, offset: usize) -> Result<&T, ArrayError> {
        self.elements.at(offset)
    }

    /// Mutates a flat zero-based offset; `*array.change_at_offset(0)? = 7`.
    pub fn change_at_offset(&mut self, offset: usize) -> Result<&mut T, ArrayError> {
        self.elements.change_at(offset)
    }

    /// Borrows the first stored element; `array.first()?`.
    pub fn first(&self) -> Result<&T, ArrayError> {
        self.elements.first()
    }

    /// Mutates the first stored element; `*array.change_first()? = 7`.
    pub fn change_first(&mut self) -> Result<&mut T, ArrayError> {
        self.elements.change_first()
    }

    /// Borrows the last stored element; `array.last()?`.
    pub fn last(&self) -> Result<&T, ArrayError> {
        self.elements.last()
    }

    /// Mutates the last stored element; `*array.change_last()? = 7`.
    pub fn change_last(&mut self) -> Result<&mut T, ArrayError> {
        self.elements.change_last()
    }
}
