//! Bounded matrix storage used by the vector algebra dependency closure.
use super::DoubleTab;
use rustycad_tkernel::ncollection::ArrayError;
/// Row-major numeric matrix retaining declared bounds; `Matrix::with_bounds(1,2,1,3)?`.
#[derive(Clone)]
pub struct Matrix<'a> {
    table: DoubleTab<'a>,
}
impl<'a> Matrix<'a> {
    /// Allocates zero-based dimensions; `Matrix::with_size(2,3)`.
    pub fn with_size(rows: usize, columns: usize) -> Self {
        Self {
            table: DoubleTab::with_size(rows, columns),
        }
    }
    /// Allocates zero-based dimensions set to one value; `Matrix::with_size_filled(2,3,0.0)`.
    pub fn with_size_filled(rows: usize, columns: usize, value: f64) -> Self {
        let mut result = Self::with_size(rows, columns);
        result.table.init(value);
        result
    }
    /// Borrows external row-major cells, the `void* Tab` constructor;
    /// `Matrix::from_buffer(&mut cells,1,2,1,3)?`.
    pub fn from_buffer(
        buffer: &'a mut [f64],
        rl: i32,
        ru: i32,
        cl: i32,
        cu: i32,
    ) -> Result<Self, ArrayError> {
        Ok(Self {
            table: DoubleTab::from_buffer(buffer, rl, ru, cl, cu)?,
        })
    }
    /// Counts rows as `size_t`; `matrix.row_size()`.
    pub fn row_size(&self) -> usize {
        self.table.row_size()
    }
    /// Counts columns as `size_t`; `matrix.col_size()`.
    pub fn col_size(&self) -> usize {
        self.table.col_size()
    }
}
impl Matrix<'_> {
    /// Allocates inclusive matrix bounds; `Matrix::with_bounds(1,2,1,3)?`.
    pub fn with_bounds(rl: i32, ru: i32, cl: i32, cu: i32) -> Result<Self, ArrayError> {
        Ok(Self {
            table: DoubleTab::with_bounds(rl, ru, cl, cu)?,
        })
    }
    /// Allocates and initializes every matrix entry; `Matrix::filled(1,2,1,3,0.0)?`.
    pub fn filled(rl: i32, ru: i32, cl: i32, cu: i32, value: f64) -> Result<Self, ArrayError> {
        let mut result = Self::with_bounds(rl, ru, cl, cu)?;
        result.table.init(value);
        Ok(result)
    }
    /// Returns row count; `matrix.row_number()`.
    pub fn row_number(&self) -> i32 {
        self.table.nb_rows()
    }
    /// Returns column count; `matrix.col_number()`.
    pub fn col_number(&self) -> i32 {
        self.table.nb_columns()
    }
    /// Returns the first declared row; `matrix.lower_row()`.
    pub fn lower_row(&self) -> i32 {
        self.table.lower_row()
    }
    /// Returns the first declared column; `matrix.lower_col()`.
    pub fn lower_col(&self) -> i32 {
        self.table.lower_col()
    }
    /// Returns the final declared row; `matrix.upper_row()`.
    pub fn upper_row(&self) -> i32 {
        self.table.upper_row()
    }
    /// Returns the final declared column; `matrix.upper_col()`.
    pub fn upper_col(&self) -> i32 {
        self.table.upper_col()
    }
    /// Mutates a zero-based cell independently of declared bounds; `*matrix.change_at(0,0)? = 2.0`.
    pub fn change_at(&mut self, row: usize, col: usize) -> Result<&mut f64, ArrayError> {
        self.table.change_at(row, col)
    }
    /// Reads a zero-based cell; `matrix.at(0,0)?`.
    pub fn at(&self, row: usize, col: usize) -> Result<&f64, ArrayError> {
        self.table.at(row, col)
    }
    /// Reads the row-major cells, as native code does through `&Matrix(LowerRow, LowerCol)`;
    /// `matrix.cells()[0]`.
    pub(crate) fn cells(&self) -> &[f64] {
        self.table.cells()
    }
    /// Mutates the row-major cells; `matrix.cells_mut()[0] = 1.0`.
    pub(crate) fn cells_mut(&mut self) -> &mut [f64] {
        self.table.cells_mut()
    }
}
impl std::ops::Index<(i32, i32)> for Matrix<'_> {
    type Output = f64;
    fn index(&self, index: (i32, i32)) -> &f64 {
        &self.table[index]
    }
}
impl std::ops::IndexMut<(i32, i32)> for Matrix<'_> {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut f64 {
        &mut self.table[index]
    }
}

mod algebra;
mod blocks;
mod inverse;
mod operators;
mod products;
mod rows;
mod storage;
