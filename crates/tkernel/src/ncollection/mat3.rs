//! NCollection_Mat3: a column-major 3x3 matrix, identity by default.
use super::square_matrix::{square_matrix_access, square_matrix_arithmetic, square_matrix_dump};
use super::{MatrixScalar, Vec3};
use crate::standard::Failure;
/// Nine elements in column-major order, identity by default; `Mat3::<f64>::identity()`.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Mat3<T> {
    columns: [[T; 3]; 3],
}
square_matrix_access!(Mat3, 3, 9, Vec3);
square_matrix_arithmetic!(Mat3, 3, Vec3);
square_matrix_dump!(Mat3, 3, "NCollection_Mat3");
impl<T> Mat3<T>
where
    T: Copy + std::ops::Mul<Output = T> + std::ops::Add<Output = T> + std::ops::Sub<Output = T>,
{
    /// Rule of Sarrus in native term order, as `Determinant`;
    /// `Mat3::<i32>::from_data([2,0,0,1,11,0,0,0,1]).determinant()`.
    pub fn determinant(&self) -> T {
        let v = |row: usize, column: usize| self.get_value(row, column);
        (v(0, 0) * v(1, 1) * v(2, 2) + v(0, 1) * v(1, 2) * v(2, 0) + v(0, 2) * v(1, 0) * v(2, 1))
            - (v(0, 2) * v(1, 1) * v(2, 0)
                + v(0, 0) * v(1, 2) * v(2, 1)
                + v(0, 1) * v(1, 0) * v(2, 2))
    }
}

impl<T: MatrixScalar> Mat3<T> {
    /// Rows are the cross products of the other two rows, as `Adjoint`; `matrix.adjoint()`.
    pub fn adjoint(&self) -> Self {
        let rows: [Vec3<T>; 3] = std::array::from_fn(|row| self.get_row(row));
        let mut adjoint: Self = Self::default();
        adjoint.set_row(0, Vec3::cross(&rows[1], &rows[2]));
        adjoint.set_row(1, Vec3::cross(&rows[2], &rows[0]));
        adjoint.set_row(2, Vec3::cross(&rows[0], &rows[1]));
        adjoint
    }
    /// `Inverted(inv, det)`: the determinant is the first adjugate row dotted with the first
    /// row; a zero determinant leaves `output` untouched, otherwise it receives the transposed
    /// adjugate divided (not multiplied by a reciprocal) by the determinant;
    /// `matrix.inverted_into(&mut out, &mut det)`.
    pub fn inverted_into(&self, output: &mut Self, determinant: &mut T) -> bool {
        let adjoint: Self = self.adjoint();
        *determinant = adjoint.get_row(0).dot(&self.get_row(0));
        if *determinant == T::from(0) {
            return false;
        }
        *output = adjoint.transposed().divided(*determinant);
        true
    }
    /// `Inverted(inv)` without the determinant; `matrix.inverted_to(&mut out)`.
    pub fn inverted_to(&self, output: &mut Self) -> bool {
        let mut determinant: T = T::from(0);
        self.inverted_into(output, &mut determinant)
    }
    /// Returns the inverse or `Standard_ConstructionError`, as `Inverted()`; `matrix.inverted()?`.
    pub fn inverted(&self) -> Result<Self, Failure> {
        let mut output: Self = Self::default();
        if self.inverted_to(&mut output) {
            return Ok(output);
        }
        Err(Failure::with_type(
            "Standard_ConstructionError",
            "NCollection_Mat3::Inverted() - matrix has zero determinant",
            "",
        ))
    }
}
