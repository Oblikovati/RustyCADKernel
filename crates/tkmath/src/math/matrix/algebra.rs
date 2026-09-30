use super::Matrix;
use rustycad_tkernel::standard::Failure;
impl Matrix<'_> {
    /// Scales every cell in place; `matrix.multiply(2.0)`.
    pub fn multiply(&mut self, scalar: f64) {
        for row in self.lower_row()..=self.upper_row() {
            for col in self.lower_col()..=self.upper_col() {
                self[(row, col)] *= scalar;
            }
        }
    }
    /// Produces native cell-times-scalar values; `matrix.multiplied(2.0)`.
    pub fn multiplied(&self, scalar: f64) -> Self {
        let mut result = self.empty_like();
        for row in self.lower_row()..=self.upper_row() {
            for col in self.lower_col()..=self.upper_col() {
                result[(row, col)] = self[(row, col)] * scalar;
            }
        }
        result
    }
    /// Divides cells directly after the native epsilon check; `matrix.divide(2.0)?`.
    pub fn divide(&mut self, scalar: f64) -> Result<(), Failure> {
        checked_divisor(scalar, "Divide")?;
        for row in self.lower_row()..=self.upper_row() {
            for col in self.lower_col()..=self.upper_col() {
                self[(row, col)] /= scalar;
            }
        }
        Ok(())
    }
    /// Multiplies by a reciprocal after checking epsilon; `matrix.divided(2.0)?`.
    pub fn divided(&self, scalar: f64) -> Result<Self, Failure> {
        checked_divisor(scalar, "Divided")?;
        Ok(self.multiplied(1.0 / scalar))
    }
    /// Adds matching dimensions by position; `left.added(&right)?`.
    pub fn added(&self, right: &Self) -> Result<Self, Failure> {
        self.pairwise(right, "Added", |left, right| left + right)
    }
    /// Subtracts matching dimensions by position; `left.subtracted(&right)?`.
    pub fn subtracted(&self, right: &Self) -> Result<Self, Failure> {
        self.pairwise(right, "Subtracted", |left, right| left - right)
    }
    fn pairwise(
        &self,
        right: &Self,
        operation: &str,
        combine: impl Fn(f64, f64) -> f64,
    ) -> Result<Self, Failure> {
        if (self.row_number(), self.col_number()) != (right.row_number(), right.col_number()) {
            return Err(Failure::with_type(
                "Standard_DimensionError",
                &format!("math_Matrix::{operation}() - input matrix has different dimensions"),
                "",
            ));
        }
        let mut result = self.empty_like();
        for row in 0..self.row_number() {
            self.pairwise_row(right, &mut result, row, &combine);
        }
        Ok(result)
    }
    fn pairwise_row(
        &self,
        right: &Self,
        result: &mut Self,
        row: i32,
        combine: &impl Fn(f64, f64) -> f64,
    ) {
        for col in 0..self.col_number() {
            result[(row + self.lower_row(), col + self.lower_col())] = combine(
                *self.at(row as usize, col as usize).unwrap(),
                *right.at(row as usize, col as usize).unwrap(),
            );
        }
    }
    /// Copies cells into exchanged row/column bounds; `matrix.transposed()`.
    pub fn transposed(&self) -> Self {
        let mut result = Self::with_bounds(
            self.lower_col(),
            self.upper_col(),
            self.lower_row(),
            self.upper_row(),
        )
        .unwrap();
        for row in self.lower_row()..=self.upper_row() {
            for col in self.lower_col()..=self.upper_col() {
                result[(col, row)] = self[(row, col)];
            }
        }
        result
    }
}
fn checked_divisor(scalar: f64, operation: &str) -> Result<(), Failure> {
    if scalar.abs() <= f64::EPSILON {
        return Err(Failure::with_type(
            "Standard_DivideByZero",
            &format!("math_Matrix::{operation}() - zero divisor"),
            "",
        ));
    }
    Ok(())
}
impl std::ops::MulAssign<f64> for Matrix<'_> {
    fn mul_assign(&mut self, scalar: f64) {
        self.multiply(scalar);
    }
}
