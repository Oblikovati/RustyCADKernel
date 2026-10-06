//! Row-major arrays with independently bounded rows and columns.
use super::array1::checked_array_extent as extent;
use super::{Array1, ArrayError};

mod buffer_constructors;
mod method_contracts;
mod raw_copy;
mod resize_policy;

/// A rectangular owned array or borrowed view; `Array2::<i32>::with_size(2, 3)`.
#[derive(Clone)]
pub struct Array2<'a, T> {
    elements: Array1<'a, T>,
    row_lower: i32,
    col_lower: i32,
    rows: usize,
    cols: usize,
}

impl<T> Default for Array2<'_, T> {
    fn default() -> Self {
        Self {
            elements: Array1::default(),
            row_lower: 1,
            col_lower: 1,
            rows: 0,
            cols: 0,
        }
    }
}

impl<'a, T: Default> Array2<'a, T> {
    /// Allocates zero-based dimensions; `Array2::<i32>::with_size(2, 3).len() == 6`.
    pub fn with_size(rows: usize, cols: usize) -> Self {
        Self {
            elements: Array1::with_size(
                rows.checked_mul(cols)
                    .expect("array dimensions overflow usize"),
            ),
            row_lower: 0,
            col_lower: 0,
            rows,
            cols,
        }
    }

    /// Allocates inclusive bounds; `Array2::<i32>::with_bounds(1,2,1,3)?`.
    /// As the native constructor, `upper == lower - 1` gives an empty dimension; only a
    /// negative extent (undefined natively) is refused.
    pub fn with_bounds(rl: i32, ru: i32, cl: i32, cu: i32) -> Result<Self, ArrayError> {
        let rows = extent(rl, ru)?;
        let cols = extent(cl, cu)?;
        let mut array = Self::with_size(rows, cols);
        array.rebase(rl, cl);
        Ok(array)
    }
}

impl<'a, T> Array2<'a, T> {
    /// Borrows the required prefix of a row-major buffer; `Array2::from_buffer(&mut [1,2,3],1,2)?`.
    pub fn from_buffer(buffer: &'a mut [T], rows: usize, cols: usize) -> Result<Self, ArrayError> {
        let length = rows
            .checked_mul(cols)
            .expect("array dimensions overflow usize");
        Ok(Self {
            elements: Array1::from_slice_size(buffer, length)?,
            row_lower: 0,
            col_lower: 0,
            rows,
            cols,
        })
    }

    /// Returns the element count; `Array2::<i32>::with_size(2,3).len() == 6`.
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    /// Checks an empty rectangle; `Array2::<i32>::default().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
    /// Counts rows; `Array2::<i32>::with_size(2,3).nb_rows() == 2`.
    pub const fn nb_rows(&self) -> usize {
        self.rows
    }
    /// Counts columns; `Array2::<i32>::with_size(2,3).nb_columns() == 3`.
    pub const fn nb_columns(&self) -> usize {
        self.cols
    }
    /// Returns the row origin; `Array2::<i32>::default().lower_row() == 1`.
    pub const fn lower_row(&self) -> i32 {
        self.row_lower
    }
    /// Returns the column origin; `Array2::<i32>::default().lower_col() == 1`.
    pub const fn lower_col(&self) -> i32 {
        self.col_lower
    }
    /// Returns the final row; `Array2::<i32>::with_size(2,3).upper_row() == 1`.
    pub fn upper_row(&self) -> i32 {
        upper(self.row_lower, self.rows)
    }
    /// Returns the final column; `Array2::<i32>::with_size(2,3).upper_col() == 2`.
    pub fn upper_col(&self) -> i32 {
        upper(self.col_lower, self.cols)
    }
    /// Tests storage ownership; `Array2::<i32>::with_size(2,3).is_deletable()`.
    pub fn is_deletable(&self) -> bool {
        self.elements.is_deletable()
    }
    /// Borrows physical row-major storage; `array.as_slice()[0]`.
    pub fn as_slice(&self) -> &[T] {
        self.elements.as_slice()
    }
    /// Mutates physical row-major storage; `array.as_mut_slice()[0] = value`.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.elements.as_mut_slice()
    }
    /// Iterates rows contiguously; `array.iter().count()`.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.elements.iter()
    }
    /// Mutates rows contiguously; `array.iter_mut().for_each(|v| *v = 1)`.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.elements.iter_mut()
    }
    /// Borrows the underlying bounded array; `array.array1().lower()`.
    pub fn array1(&self) -> &Array1<'a, T> {
        &self.elements
    }
    /// Mutates the underlying bounded array; `array.change_array1().init(0)`.
    pub fn change_array1(&mut self) -> &mut Array1<'a, T> {
        &mut self.elements
    }

    /// Reads zero-based row/column offsets; `array.at(0,0)?`.
    pub fn at(&self, row: usize, col: usize) -> Result<&T, ArrayError> {
        self.elements.at(self.offset(row, col)?)
    }
    /// Mutates zero-based row/column offsets; `*array.change_at(0,0)? = value`.
    pub fn change_at(&mut self, row: usize, col: usize) -> Result<&mut T, ArrayError> {
        let offset = self.offset(row, col)?;
        self.elements.change_at(offset)
    }
    /// Reads declared row/column indices; `array.value(array.lower_row(),array.lower_col())?`.
    pub fn value(&self, row: i32, col: i32) -> Result<&T, ArrayError> {
        let (r, c) = self.bounded_offsets(row, col)?;
        self.at(r, c)
    }
    /// Mutates declared row/column indices; `*array.change_value(1,1)? = value`.
    pub fn change_value(&mut self, row: i32, col: i32) -> Result<&mut T, ArrayError> {
        let (r, c) = self.bounded_offsets(row, col)?;
        self.change_at(r, c)
    }
    /// Replaces a cell; `array.set_value(1,1,42)?`.
    pub fn set_value(&mut self, row: i32, col: i32, value: T) -> Result<(), ArrayError> {
        *self.change_value(row, col)? = value;
        Ok(())
    }
    /// Constructs a replacement cell; `array.emplace_value(1,1,|| 42)?`.
    pub fn emplace_value(
        &mut self,
        row: i32,
        col: i32,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, ArrayError> {
        let slot = self.change_value(row, col)?;
        *slot = construct();
        Ok(slot)
    }
    /// Shifts row bounds without moving values; `array.update_lower_row(5)`.
    pub fn update_lower_row(&mut self, lower: i32) {
        self.row_lower = lower;
    }
    /// Shifts column bounds without moving values; `array.update_lower_col(5)`.
    pub fn update_lower_col(&mut self, lower: i32) {
        self.col_lower = lower;
    }
    /// Sets the upper row and derives the lower row; `array.update_upper_row(10)`.
    pub fn update_upper_row(&mut self, upper: i32) {
        self.row_lower = (i64::from(upper) - self.rows as i64 + 1) as i32;
    }
    /// Sets the upper column and derives the lower column; `array.update_upper_col(10)`.
    pub fn update_upper_col(&mut self, upper: i32) {
        self.col_lower = (i64::from(upper) - self.cols as i64 + 1) as i32;
    }
    /// Transfers storage, leaving the source empty; `dest.move_from(&mut source)`.
    pub fn move_from(&mut self, source: &mut Self) -> &mut Self {
        *self = std::mem::take(source);
        self
    }

    fn rebase(&mut self, row: i32, col: i32) {
        self.row_lower = row;
        self.col_lower = col;
        self.elements
            .update_lower_bound((i64::from(col) + i64::from(row) * self.cols as i64) as i32);
    }
    fn offset(&self, row: usize, col: usize) -> Result<usize, ArrayError> {
        check_offset(row, self.rows)?;
        check_offset(col, self.cols)?;
        Ok(row * self.cols + col)
    }
    fn bounded_offsets(&self, row: i32, col: i32) -> Result<(usize, usize), ArrayError> {
        Ok((
            bounded_offset(row, self.lower_row(), self.upper_row())?,
            bounded_offset(col, self.lower_col(), self.upper_col())?,
        ))
    }
}

impl<'a, T: Clone> Array2<'a, T> {
    /// Assigns borrowed data in owned or mutable-buffer storage; `array.set_value_ref(1,1,&value)?`.
    pub fn set_value_ref(&mut self, row: i32, col: i32, value: &T) -> Result<(), ArrayError> {
        self.change_value(row, col)?.clone_from(value);
        Ok(())
    }

    /// Views the leading `rows * cols` values of a read-only buffer with inclusive
    /// bounds, as the native `NCollection_Array2(const_reference, ...)` constructor
    /// does. The first mutable access detaches a private copy (see
    /// [`Array1::from_shared`]); `Array2::from_shared(&[1.0; 4], 1, 2, 1, 2)?`.
    pub fn from_shared(
        buffer: &'a [T],
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
    ) -> Result<Self, ArrayError> {
        let rows = extent(rl, ru)?;
        let cols = extent(cl, cu)?;
        let length = rows
            .checked_mul(cols)
            .expect("array dimensions overflow usize");
        if buffer.len() < length {
            return Err(ArrayError::DimensionMismatch {
                expected: length,
                actual: buffer.len(),
            });
        }
        let mut array = Self {
            elements: Array1::from_shared(&buffer[..length], 0),
            row_lower: 0,
            col_lower: 0,
            rows,
            cols,
        };
        array.rebase(rl, cl);
        Ok(array)
    }
}

impl<T: Clone> Array2<'_, T> {
    /// Fills the rectangle; `array.init(42)`.
    pub fn init(&mut self, value: T) {
        // The native loop assigns from one borrowed value rather than cloning a temporary per slot.
        self.init_ref(&value);
    }
    /// Assigns from a borrowed value in row-major order; `array.init_ref(&fill_value)`.
    pub fn init_ref(&mut self, value: &T) {
        // Keep the borrowed contract so custom elements can assign without copy construction.
        self.elements.init_ref(value);
    }
    /// Assigns bounds and contents, reusing equal-size storage; `dest.assign(&source)`.
    pub fn assign(&mut self, source: &Array2<'_, T>) -> &mut Self {
        self.elements.assign(&source.elements);
        self.adopt_rectangle(source);
        self
    }
    /// Copies matching dimensions without rebasing; `dest.copy_values(&source)?`.
    pub fn copy_values(&mut self, source: &Array2<'_, T>) -> Result<(), ArrayError> {
        self.check_rectangle(source)?;
        self.elements.copy_values(&source.elements)
    }
}

// Constructors accept an empty `[lower, lower - 1]` range (native RangeSize 0); only
// Resize raises Standard_RangeError for `upper < lower`, which `dimension` mirrors.
fn dimension(lower: i32, upper: i32) -> Result<usize, ArrayError> {
    if upper < lower {
        return Err(ArrayError::Bounds { lower, upper });
    }
    Ok((i64::from(upper) - i64::from(lower) + 1) as usize)
}
fn upper(lower: i32, size: usize) -> i32 {
    (i64::from(lower) + size as i64 - 1) as i32
}
fn check_offset(index: usize, size: usize) -> Result<(), ArrayError> {
    if index < size {
        return Ok(());
    }
    Err(ArrayError::Index {
        index: index as i64,
        lower: 0,
        upper: size as i64 - 1,
    })
}
fn bounded_offset(index: i32, lower: i32, upper: i32) -> Result<usize, ArrayError> {
    if index < lower || index > upper {
        return Err(ArrayError::Index {
            index: index.into(),
            lower: lower.into(),
            upper: upper.into(),
        });
    }
    Ok((i64::from(index) - i64::from(lower)) as usize)
}

impl<T> std::ops::Index<(i32, i32)> for Array2<'_, T> {
    type Output = T;
    fn index(&self, (row, col): (i32, i32)) -> &T {
        self.value(row, col)
            .unwrap_or_else(|error| panic!("{error}"))
    }
}
impl<T> std::ops::IndexMut<(i32, i32)> for Array2<'_, T> {
    fn index_mut(&mut self, (row, col): (i32, i32)) -> &mut T {
        self.change_value(row, col)
            .unwrap_or_else(|error| panic!("{error}"))
    }
}
impl<'b, T> IntoIterator for &'b Array2<'_, T> {
    type Item = &'b T;
    type IntoIter = std::slice::Iter<'b, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'b, T> IntoIterator for &'b mut Array2<'_, T> {
    type Item = &'b mut T;
    type IntoIter = std::slice::IterMut<'b, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

include!("array2_resize.rs");
include!("array2_layout.rs");
