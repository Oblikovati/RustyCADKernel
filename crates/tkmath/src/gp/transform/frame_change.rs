// Rust translation of FoundationClasses/TKMath/gp/gp_Trsf.cxx SetTransformation/SetDisplacement
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::super::{Axis3, Failure, Matrix3, Transform3, TransformForm, Xyz};
/// The frame directions as matrix rows (`SetRows(X, Y, Z)`).
fn frame_rows_matrix(frame: Axis3) -> Matrix3 {
    let mut matrix = Matrix3::default();
    matrix.set_rows(
        Xyz::from(frame.x_direction()),
        Xyz::from(frame.y_direction()),
        Xyz::from(frame.direction()),
    );
    matrix
}
/// The frame directions as matrix columns (`gp_Mat(X, Y, Z)`).
fn frame_columns_matrix(frame: Axis3) -> Matrix3 {
    Matrix3::from_columns(
        frame.x_direction().into(),
        frame.y_direction().into(),
        frame.direction().into(),
    )
}
impl Transform3 {
    /// Maps global coordinates into a frame's local system (gp_Trsf::SetTransformation(gp_Ax3));
    /// `Transform3::transformation_to(frame)`.
    pub fn transformation_to(frame: Axis3) -> Self {
        let matrix = frame_rows_matrix(frame);
        let offset = matrix.apply(frame.location().xyz()).reversed();
        Self {
            form: TransformForm::Compound,
            matrix,
            offset,
            scale: 1.0,
        }
    }
    /// Maps coordinates in `from` to coordinates in `to` (gp_Trsf::SetTransformation(A1,A2));
    /// `Transform3::transformation_between(from,to)`.
    pub fn transformation_between(from: Axis3, to: Axis3) -> Self {
        let mut transform = Self::transformation_to(to);
        let from_matrix = frame_columns_matrix(from);
        transform.offset += transform.matrix.apply(from.location().xyz());
        transform.matrix *= from_matrix;
        transform
    }
    /// Moves the `from` frame onto the `to` frame (gp_Trsf::SetDisplacement) using native
    /// multiplication order; `Transform3::displacement(from,to)`.
    pub fn displacement(from: Axis3, to: Axis3) -> Self {
        let mut matrix = frame_columns_matrix(to);
        let inverse = frame_columns_matrix(from).transposed();
        let from_origin = inverse.apply(from.location().xyz()).reversed();
        let offset = to.location().xyz() + matrix.apply(from_origin);
        matrix *= inverse;
        Self {
            form: TransformForm::Compound,
            scale: 1.0,
            matrix,
            offset,
        }
    }
    /// Reads a scaled matrix coefficient or the translation (column 4) by one-based index;
    /// `transform.value(1,4)?`.
    pub fn value(&self, row: i32, column: i32) -> Result<f64, Failure> {
        if !(1..=3).contains(&row) || !(1..=4).contains(&column) {
            return Err(Failure::out_of_range(&format!(
                "gp_Trsf::Value() - index ({row}, {column}) outside rows [1, 3] and columns [1, 4]"
            )));
        }
        Ok(self.coefficient(row as usize, column as usize))
    }
    /// Reads `value` for indices already known to be in range; `transform.coefficient(1,4)`.
    pub(in crate::gp) fn coefficient(&self, row: usize, column: usize) -> f64 {
        if column < 4 {
            return self.scale * self.matrix.coefficients()[row - 1][column - 1];
        }
        self.offset.coordinates()[row - 1]
    }
    /// Reads the 3x4 `value` table of a frame change in row order; `Transform3::frame_rows(frame)`.
    pub(in crate::gp) fn frame_rows(frame: Axis3) -> [[f64; 4]; 3] {
        let transform = Self::transformation_to(frame);
        std::array::from_fn(|row| {
            std::array::from_fn(|column| transform.coefficient(row + 1, column + 1))
        })
    }
}
