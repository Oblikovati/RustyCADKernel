//! NCollection_Mat4: a column-major 4x4 matrix, identity by default.
use super::square_matrix::{square_matrix_access, square_matrix_arithmetic, square_matrix_dump};
use super::{Mat3, MatrixScalar, Vec3, Vec4};
use std::ops::Neg;
mod inverse;
/// Sixteen elements in column-major order, identity by default; `Mat4::<f64>::identity()`.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Mat4<T> {
    columns: [[T; 4]; 4],
}
square_matrix_access!(Mat4, 4, 16, Vec4);
square_matrix_arithmetic!(Mat4, 4, Vec4);
square_matrix_dump!(Mat4, 4, "NCollection_Mat4");
impl<T> Mat4<T> {
    /// Number of rows, as `Rows` (NCollection_Mat3 has no such member); `Mat4::<f64>::rows() == 4`.
    pub const fn rows() -> usize {
        4
    }
    /// Number of columns, as `Cols`; `Mat4::<f64>::cols() == 4`.
    pub const fn cols() -> usize {
        4
    }
}
impl<T: Copy> Mat4<T> {
    /// Changes the first three row values, as `SetRow(Vec3)`; `matrix.set_row3(0, v)`.
    pub fn set_row3(&mut self, row: usize, values: Vec3<T>) {
        for (column, value) in values.data().into_iter().enumerate() {
            self.columns[column][row] = value;
        }
    }
    /// Changes the first three column values, as `SetColumn(Vec3)`; `matrix.set_column3(3, t)`.
    pub fn set_column3(&mut self, column: usize, values: Vec3<T>) {
        self.columns[column][..3].copy_from_slice(&values.data());
    }
    /// Changes the first three diagonal values, as `SetDiagonal(Vec3)`; `matrix.set_diagonal3(s)`.
    pub fn set_diagonal3(&mut self, values: Vec3<T>) {
        for (index, value) in values.data().into_iter().enumerate() {
            self.columns[index][index] = value;
        }
    }
}
impl<T: Copy + From<u8>> Mat4<T> {
    /// The upper-left 3x3 block, as `GetMat3`; `matrix.get_mat3()`.
    pub fn get_mat3(&self) -> Mat3<T> {
        let mut block: Mat3<T> = Mat3::default();
        for column in 0..3 {
            block.set_column(column, self.get_column(column).xyz());
        }
        block
    }
}
impl<T: MatrixScalar> Mat4<T> {
    /// Postmultiplies a translation matrix, as `Translate`; `matrix.translate(Vec3::new(1.0,2.0,3.0))`.
    pub fn translate(&mut self, offset: Vec3<T>) {
        let mut translation: Self = Self::default();
        translation.set_column3(3, offset);
        self.multiply(&translation);
    }
}

impl<T> Mat4<T>
where
    T: Copy + std::ops::Mul<Output = T> + std::ops::Add<Output = T> + std::ops::Sub<Output = T>,
{
    /// Determinant of the upper-left 3x3 block by the rule of Sarrus, as `DeterminantMat3`;
    /// `Mat4::<i32>::from_data([2,0,0,0,1,11,0,0,0,0,1,0,0,0,0,1]).determinant_mat3()`.
    pub fn determinant_mat3(&self) -> T {
        let v = |row: usize, column: usize| self.get_value(row, column);
        (v(0, 0) * v(1, 1) * v(2, 2) + v(0, 1) * v(1, 2) * v(2, 0) + v(0, 2) * v(1, 0) * v(2, 1))
            - (v(0, 2) * v(1, 1) * v(2, 0)
                + v(0, 0) * v(1, 2) * v(2, 1)
                + v(0, 1) * v(1, 0) * v(2, 2))
    }
}
impl<T: MatrixScalar + Neg<Output = T>> Mat4<T> {
    /// Rows are 4D cross products of the other rows, as `Adjoint`. The sign convention of
    /// `crossVec4` makes this the negated cofactor matrix: `Adjoint(I) = -I` and the inverse is
    /// `-Adjoint^T / det`, unlike NCollection_Mat3::Adjoint; `matrix.adjoint()`.
    pub fn adjoint(&self) -> Self {
        let rows: [Vec4<T>; 4] = std::array::from_fn(|row| self.get_row(row));
        let mut adjoint: Self = Self::default();
        adjoint.set_row(0, cross4(rows[1], rows[2], rows[3]));
        adjoint.set_row(1, cross4(negated_components(rows[0]), rows[2], rows[3]));
        adjoint.set_row(2, cross4(rows[0], rows[1], rows[3]));
        adjoint.set_row(3, cross4(negated_components(rows[0]), rows[1], rows[2]));
        adjoint
    }
}
/// The private `crossVec4`: the vector orthogonal to `b` and `c` weighted by `a`.
fn cross4<T: MatrixScalar + Neg<Output = T>>(a: Vec4<T>, b: Vec4<T>, c: Vec4<T>) -> Vec4<T> {
    let d1 = (b.z * c.w) - (b.w * c.z);
    let d2 = (b.y * c.w) - (b.w * c.y);
    let d3 = (b.y * c.z) - (b.z * c.y);
    let d4 = (b.x * c.w) - (b.w * c.x);
    let d5 = (b.x * c.z) - (b.z * c.x);
    let d6 = (b.x * c.y) - (b.y * c.x);
    Vec4::new(
        -a.y * d1 + a.z * d2 - a.w * d3,
        a.x * d1 - a.z * d4 + a.w * d5,
        -a.x * d2 + a.y * d4 - a.w * d6,
        a.x * d3 - a.y * d5 + a.z * d6,
    )
}

// Keep Mat4's existing scalar Neg contract independent of vector template policy adapters.
fn negated_components<T: MatrixScalar + Neg<Output = T>>(row: Vec4<T>) -> Vec4<T> {
    Vec4::new(
        row.x.vector_negate(),
        row.y.vector_negate(),
        row.z.vector_negate(),
        row.w.vector_negate(),
    )
}
