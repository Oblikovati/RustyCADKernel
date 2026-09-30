//! math_Matrix operators, `Dump` and the free `operator<<` / `operator*(double, Matrix)`.
use super::Matrix;
use crate::math::vector::stream_real;
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Neg, Sub, SubAssign};
impl Matrix<'_> {
    /// Native TMultiplied(double), identical to `multiplied`; `matrix.t_multiplied(2.0)`.
    pub fn t_multiplied(&self, scalar: f64) -> Self {
        self.multiplied(scalar)
    }
    /// Writes the native `Dump` text, one declared cell per line; `matrix.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        write!(stream, "math_Matrix of RowNumber = {}", self.row_number())?;
        writeln!(stream, " and ColNumber = {}", self.col_number())?;
        for row in self.lower_row()..=self.upper_row() {
            for col in self.lower_col()..=self.upper_col() {
                write!(stream, "math_Matrix ( {row}, {col} ) = ")?;
                writeln!(stream, "{}", stream_real(self[(row, col)]))?;
            }
        }
        Ok(())
    }
}
/// The free `operator<<`, which forwards to `Dump`; `format!("{matrix}")`.
impl fmt::Display for Matrix<'_> {
    fn fmt(&self, stream: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.dump(stream)
    }
}
impl<'a> Mul<f64> for &Matrix<'a> {
    type Output = Matrix<'a>;
    fn mul(self, right: f64) -> Matrix<'a> {
        self.multiplied(right)
    }
}
/// The free `operator*(double, const math_Matrix&)`; `2.0 * &matrix`.
impl<'a> Mul<&Matrix<'a>> for f64 {
    type Output = Matrix<'a>;
    fn mul(self, right: &Matrix<'a>) -> Matrix<'a> {
        right.multiplied(self)
    }
}
impl<'a> Mul<&Matrix<'a>> for &Matrix<'a> {
    type Output = Matrix<'a>;
    fn mul(self, right: &Matrix<'a>) -> Matrix<'a> {
        self.multiplied_matrix(right).unwrap()
    }
}
impl<'a> Div<f64> for &Matrix<'a> {
    type Output = Matrix<'a>;
    fn div(self, right: f64) -> Matrix<'a> {
        self.divided(right).unwrap()
    }
}
impl<'a> Add<&Matrix<'a>> for &Matrix<'a> {
    type Output = Matrix<'a>;
    fn add(self, right: &Matrix<'a>) -> Matrix<'a> {
        self.added(right).unwrap()
    }
}
impl<'a> Sub<&Matrix<'a>> for &Matrix<'a> {
    type Output = Matrix<'a>;
    fn sub(self, right: &Matrix<'a>) -> Matrix<'a> {
        self.subtracted(right).unwrap()
    }
}
impl<'a> Neg for &Matrix<'a> {
    type Output = Matrix<'a>;
    fn neg(self) -> Matrix<'a> {
        self.opposite()
    }
}
impl AddAssign<&Matrix<'_>> for Matrix<'_> {
    fn add_assign(&mut self, right: &Matrix<'_>) {
        self.add(right).unwrap();
    }
}
impl SubAssign<&Matrix<'_>> for Matrix<'_> {
    fn sub_assign(&mut self, right: &Matrix<'_>) {
        self.subtract(right).unwrap();
    }
}
impl DivAssign<f64> for Matrix<'_> {
    fn div_assign(&mut self, right: f64) {
        self.divide(right).unwrap();
    }
}
