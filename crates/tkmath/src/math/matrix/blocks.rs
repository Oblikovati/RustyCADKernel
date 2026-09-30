use super::Matrix;
use crate::math::not_square;
use rustycad_tkernel::standard::Failure;
impl Matrix<'_> {
    /// Adds a same-shaped matrix in place by position; `left.add(&right)?`.
    pub fn add(&mut self, right: &Matrix<'_>) -> Result<(), Failure> {
        self.check_shape(right, "Add() - input matrix has different dimensions")?;
        self.update_pairwise(right, |a, b| a + b);
        Ok(())
    }
    /// Subtracts a same-shaped matrix in place by position; `left.subtract(&right)?`.
    pub fn subtract(&mut self, right: &Matrix<'_>) -> Result<(), Failure> {
        self.check_shape(right, "Subtract() - input matrix has different dimensions")?;
        self.update_pairwise(right, |a, b| a - b);
        Ok(())
    }
    /// Stores `left + right` into this matrix's bounds, the two-operand `Add`;
    /// `result.set_sum(&left,&right)?`.
    pub fn set_sum(&mut self, left: &Matrix<'_>, right: &Matrix<'_>) -> Result<(), Failure> {
        self.check_pair(left, right, "Add() - matrices have incompatible dimensions")?;
        self.store_pairwise(left, right, |a, b| a + b);
        Ok(())
    }
    /// Stores `left - right` into this matrix's bounds, the two-operand `Subtract`;
    /// `result.set_difference(&left,&right)?`.
    pub fn set_difference(&mut self, left: &Matrix<'_>, right: &Matrix<'_>) -> Result<(), Failure> {
        self.check_pair(
            left,
            right,
            "Subtract() - matrices have incompatible dimensions",
        )?;
        self.store_pairwise(left, right, |a, b| a - b);
        Ok(())
    }
    /// Copies `block` into rows `first_row..=last_row` and columns `first_col..=last_col`;
    /// cells outside the bounds raise after earlier cells were written, as natively;
    /// `matrix.set(1,2,1,2,&block)?`.
    pub fn set(
        &mut self,
        first_row: i32,
        last_row: i32,
        first_col: i32,
        last_col: i32,
        block: &Matrix<'_>,
    ) -> Result<(), Failure> {
        if first_row > last_row
            || first_col > last_col
            || last_row - first_row + 1 != block.row_number()
            || last_col - first_col + 1 != block.col_number()
        {
            return Err(dimension("Set() - invalid indices"));
        }
        for row in 0..block.row_number() {
            for col in 0..block.col_number() {
                *self.change_value(first_row + row, first_col + col)? =
                    block[(block.lower_row() + row, block.lower_col() + col)];
            }
        }
        Ok(())
    }
    /// Returns the sign-reversed matrix; `matrix.opposite()`.
    pub fn opposite(&self) -> Self {
        let mut result = self.empty_like();
        for (target, source) in result.cells_mut().iter_mut().zip(self.cells()) {
            *target = -source;
        }
        result
    }
    /// Transposes a square matrix in place, exchanging the row and column lower bounds.
    ///
    /// The native loop runs after rebasing the columns onto the row lower bound but
    /// with the old upper column. When the column bounds start above the row bounds it
    /// raises `Standard_OutOfRange` part-way (bounds already rebased); when they start
    /// below it swaps nothing and only exchanges the bounds. `matrix.transpose()?`.
    pub fn transpose(&mut self) -> Result<(), Failure> {
        if self.row_number() != self.col_number() {
            return Err(not_square(
                "math_Matrix::Transpose() - matrix is not square",
            ));
        }
        let (row, col, upper_col) = (self.lower_row(), self.lower_col(), self.upper_col());
        self.table.set_lower_col(row);
        for i in row..=self.upper_row() {
            for j in i..=upper_col {
                let temp = *self.value(i, j)?;
                *self.change_value(i, j)? = *self.value(j, i)?;
                *self.change_value(j, i)? = temp;
            }
        }
        self.table.set_lower_row(col);
        self.table.set_lower_col(row);
        Ok(())
    }
    fn update_pairwise(&mut self, right: &Matrix<'_>, combine: impl Fn(f64, f64) -> f64) {
        for (target, b) in self.cells_mut().iter_mut().zip(right.cells()) {
            *target = combine(*target, *b);
        }
    }
    fn store_pairwise(
        &mut self,
        left: &Matrix<'_>,
        right: &Matrix<'_>,
        combine: impl Fn(f64, f64) -> f64,
    ) {
        let cells = left.cells().iter().zip(right.cells());
        for (target, (a, b)) in self.cells_mut().iter_mut().zip(cells) {
            *target = combine(*a, *b);
        }
    }
    fn check_shape(&self, right: &Matrix<'_>, message: &str) -> Result<(), Failure> {
        if (self.row_number(), self.col_number()) != (right.row_number(), right.col_number()) {
            return Err(dimension(message));
        }
        Ok(())
    }
    fn check_pair(
        &self,
        left: &Matrix<'_>,
        right: &Matrix<'_>,
        message: &str,
    ) -> Result<(), Failure> {
        let shape = (right.row_number(), right.col_number());
        if (self.row_number(), self.col_number()) != shape
            || (left.row_number(), left.col_number()) != shape
        {
            return Err(dimension(message));
        }
        Ok(())
    }
}
fn dimension(message: &str) -> Failure {
    Failure::with_type(
        "Standard_DimensionError",
        &format!("math_Matrix::{message}"),
        "",
    )
}
