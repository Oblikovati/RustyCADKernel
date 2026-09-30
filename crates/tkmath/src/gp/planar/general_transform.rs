// Rust translation of FoundationClasses/TKMath/gp/gp_GTrsf2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_GTrsf2d: a planar affine transformation. Non-orthogonal matrices carry the `Other`
//! form and a zero scale; every other form is an embedded gp_Trsf2d.
use super::{Axis2d, Failure, Matrix2, Transform2, TransformForm, Xy, construction};
use rustycad_tkernel::precision;
mod algebra;
/// Planar affine transformation with gp_GTrsf2d's form bookkeeping; `GeneralTransform2::default()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeneralTransform2 {
    matrix: Matrix2,
    offset: Xy,
    form: TransformForm,
    scale: f64,
}
impl Default for GeneralTransform2 {
    fn default() -> Self {
        Self::from(Transform2::default())
    }
}
/// gp_GTrsf2d(const gp_Trsf2d&) and SetTrsf2d copy all four fields.
impl From<Transform2> for GeneralTransform2 {
    fn from(transform: Transform2) -> Self {
        Self {
            matrix: transform.matrix,
            offset: transform.offset,
            form: transform.form,
            scale: transform.scale,
        }
    }
}
impl GeneralTransform2 {
    /// A general transformation `X' = M*X + V` with form Other and scale 0;
    /// `GeneralTransform2::new(matrix,offset)`.
    pub const fn new(matrix: Matrix2, offset: Xy) -> Self {
        Self {
            matrix,
            offset,
            form: TransformForm::Other,
            scale: 0.0,
        }
    }
    /// Affinity with respect to an axis: points on the axis stay fixed and distances to it
    /// are multiplied by the ratio; `GeneralTransform2::affinity(axis,2.0)`.
    pub fn affinity(axis: Axis2d, ratio: f64) -> Self {
        let (a, b) = (axis.direction().x(), axis.direction().y());
        let cross = (1.0 - ratio) * a * b;
        let matrix = Matrix2::new([
            [(1.0 - ratio) * a * a + ratio, cross],
            [cross, (1.0 - ratio) * b * b + ratio],
        ]);
        let origin = axis.location().xy();
        Self::new(matrix, origin.reversed().multiplied_matrix(matrix) + origin)
    }
    /// gp_GTrsf2d::SetAffinity; `transform.set_affinity(axis,2.0)`.
    pub fn set_affinity(&mut self, axis: Axis2d, ratio: f64) {
        *self = Self::affinity(axis, ratio);
    }
    /// Sets one coefficient (column 3 is the translation) and forces the Other form;
    /// `transform.set_value(1,3,2.0)?`.
    pub fn set_value(&mut self, row: i32, column: i32, value: f64) -> Result<(), Failure> {
        check_cell(row, column)?;
        if column == 3 {
            self.offset.set_coord(row, value)?;
        } else {
            self.matrix.set_value(row, column, value)?;
        }
        self.form = TransformForm::Other;
        Ok(())
    }
    /// Replaces the translation and re-derives the form; `transform.set_translation_part(xy)`.
    pub fn set_translation_part(&mut self, offset: Xy) {
        self.offset = offset;
        self.form = match self.form {
            TransformForm::Compound | TransformForm::Other | TransformForm::Translation => {
                self.form
            }
            TransformForm::Identity => TransformForm::Translation,
            _ => TransformForm::Compound,
        };
    }
    /// Copies a similarity (gp_GTrsf2d::SetTrsf2d); `transform.set_trsf2d(similarity)`.
    pub fn set_trsf2d(&mut self, transform: Transform2) {
        *self = Self::from(transform);
    }
    /// Replaces the matrix, forcing form Other and scale 0; `transform.set_vectorial_part(matrix)`.
    pub fn set_vectorial_part(&mut self, matrix: Matrix2) {
        self.matrix = matrix;
        self.form = TransformForm::Other;
        self.scale = 0.0;
    }
}
fn check_cell(row: i32, column: i32) -> Result<(), Failure> {
    if !(1..=2).contains(&row) || !(1..=3).contains(&column) {
        return Err(Failure::out_of_range(&format!(
            "gp_GTrsf2d - cell ({row}, {column}) outside rows [1, 2] and columns [1, 3]"
        )));
    }
    Ok(())
}
impl GeneralTransform2 {
    /// Tests the sign of the matrix determinant; `transform.is_negative()`.
    pub fn is_negative(&self) -> bool {
        self.matrix.determinant() < 0.0
    }
    /// Tests the matrix determinant against gp::Resolution; `transform.is_singular()`.
    pub fn is_singular(&self) -> bool {
        self.matrix.is_singular()
    }
    /// Returns the form; `transform.form()`.
    pub const fn form(&self) -> TransformForm {
        self.form
    }
    /// Returns the translation part; `transform.translation_part()`.
    pub const fn translation_part(&self) -> Xy {
        self.offset
    }
    /// Returns the stored matrix, unscaled for similarity forms; `transform.vectorial_part()`.
    pub const fn vectorial_part(&self) -> Matrix2 {
        self.matrix
    }
    /// Reads a coefficient (also operator()); similarity forms are scaled, Other is not;
    /// `transform.value(1,1)?`.
    pub fn value(&self, row: i32, column: i32) -> Result<f64, Failure> {
        check_cell(row, column)?;
        if column == 3 {
            return self.offset.coord(row);
        }
        let coefficient = self.matrix.coefficients[row as usize - 1][column as usize - 1];
        if self.form == TransformForm::Other {
            return Ok(coefficient);
        }
        Ok(self.scale * coefficient)
    }
    /// Applies matrix, the scale of a non-unit similarity, then the translation;
    /// `transform.transformed(xy)`.
    pub fn transformed(&self, xy: Xy) -> Xy {
        let mut result = xy.multiplied_matrix(self.matrix);
        if self.form != TransformForm::Other && self.scale != 1.0 {
            result *= self.scale;
        }
        result + self.offset
    }
    /// gp_GTrsf2d::Transforms(gp_XY&); `transform.transforms(&mut xy)`.
    pub fn transforms(&self, xy: &mut Xy) {
        *xy = self.transformed(*xy);
    }
    /// gp_GTrsf2d::Transforms(double&,double&); `transform.transforms_coordinates(1.0,2.0)`.
    pub fn transforms_coordinates(&self, x: f64, y: f64) -> (f64, f64) {
        let result = self.transformed(Xy::new(x, y));
        (result.x(), result.y())
    }
    /// Converts back to a similarity after the native orthogonality checks
    /// (2*Precision::Angular on column norms, Precision::Angular on their dot product);
    /// `transform.trsf2d()?`.
    pub fn trsf2d(&self) -> Result<Transform2, Failure> {
        if self.form == TransformForm::Other {
            return Err(construction(
                "gp_GTrsf2d::Trsf2d() - non-orthogonal GTrsf2d(0)",
            ));
        }
        let m = self.matrix.coefficients;
        let columns = [
            m[0][0] * m[0][0] + m[1][0] * m[1][0],
            m[0][1] * m[0][1] + m[1][1] * m[1][1],
        ];
        for (index, norm) in columns.into_iter().enumerate() {
            if (norm - 1.0).abs() > 2.0 * precision::angular() {
                return Err(non_orthogonal(index + 1, norm));
            }
        }
        let dot = m[0][0] * m[0][1] + m[1][0] * m[1][1];
        if dot.abs() > precision::angular() {
            return Err(non_orthogonal(3, dot));
        }
        Ok(self.as_trsf2d())
    }
    fn as_trsf2d(&self) -> Transform2 {
        Transform2 {
            form: self.form,
            matrix: self.matrix,
            offset: self.offset,
            scale: self.scale,
        }
    }
}
fn non_orthogonal(check: usize, value: f64) -> Failure {
    construction(&format!(
        "gp_GTrsf2d::Trsf2d() - non-orthogonal GTrsf2d({check}): {value}"
    ))
}
