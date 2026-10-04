//! gp_Trsf partial updates that re-derive the form, SetValues, and the derived parts.
use super::super::{
    Failure, Matrix3, Quaternion, Transform3, TransformForm, Vector3, Xyz, construction,
};
use TransformForm::*;
impl Transform3 {
    /// Replaces the translation; a squared length below gp::Resolution counts as null when
    /// the form is re-derived; `transform.set_translation_part(vector)`.
    pub fn set_translation_part(&mut self, vector: Vector3) {
        self.offset = *vector.xyz();
        let null = self.offset.square_modulus() < f64::MIN_POSITIVE;
        self.form = match self.form {
            Identity if !null => Translation,
            Translation if null => Identity,
            Identity | Translation => self.form,
            _ if !null => Compound,
            _ => self.form,
        };
    }
    /// Replaces the scale and re-derives the form; a scale at or below gp::Resolution raises
    /// before any change; `transform.set_scale_factor(2.0)?`.
    pub fn set_scale_factor(&mut self, scale: f64) -> Result<(), Failure> {
        if scale.abs() <= f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "gp_Trsf::SetScaleFactor - scale {scale} is not above gp::Resolution"
            )));
        }
        self.scale = scale;
        let unit = (scale - 1.0).abs() <= f64::MIN_POSITIVE;
        let minus_unit = (scale + 1.0).abs() <= f64::MIN_POSITIVE;
        self.form = scaled_form(self.form, unit, minus_unit);
        Ok(())
    }
    /// Returns the scaled matrix; scale and point-mirror forms scale only the diagonal;
    /// `transform.vectorial_part()`.
    pub fn vectorial_part(&self) -> Matrix3 {
        if self.scale == 1.0 {
            return self.matrix;
        }
        let mut matrix = self.matrix;
        if matches!(self.form, Scale | PointMirror) {
            let diagonal = matrix.diagonal();
            matrix.set_diagonal(
                self.scale * diagonal.x(),
                self.scale * diagonal.y(),
                self.scale * diagonal.z(),
            );
        } else {
            matrix *= self.scale;
        }
        matrix
    }
    /// Extracts the native quaternion from the unscaled matrix (gp_Trsf::GetRotation());
    /// `transform.rotation_quaternion()`.
    pub fn rotation_quaternion(&self) -> Quaternion {
        Quaternion::from_matrix(self.matrix)
    }
    /// Axis and angle of the rotation quaternion, as `GetRotation(gp_XYZ&, double&)`, which
    /// always returns true; `transform.rotation_axis_angle()`.
    pub fn rotation_axis_angle(&self) -> (Xyz, f64) {
        let (axis, angle) = self.rotation_quaternion().vector_and_angle();
        (*axis.xyz(), angle)
    }
    /// Replaces the matrix with the quaternion matrix (identity for the identity quaternion)
    /// and re-derives the form; `transform.set_rotation_part(q)`.
    pub fn set_rotation_part(&mut self, rotation: Quaternion) {
        let has_rotation = !rotation.is_equal(&Quaternion::default());
        self.matrix = if has_rotation {
            rotation.matrix()
        } else {
            let mut identity = Matrix3::default();
            identity.set_identity();
            identity
        };
        self.form = match self.form {
            Identity if has_rotation => Rotation,
            Rotation if !has_rotation => Identity,
            Identity | Rotation => self.form,
            _ if has_rotation => Compound,
            _ => self.form,
        };
    }
}
/// The gp_Trsf::SetScaleFactor switch; later assignments in a native case win.
fn scaled_form(form: TransformForm, unit: bool, minus_unit: bool) -> TransformForm {
    match form {
        Identity | Translation if minus_unit => PointMirror,
        Identity | Translation if !unit => Scale,
        Rotation if !unit => Compound,
        PointMirror | AxisMirror | PlaneMirror if unit => Identity,
        PointMirror | AxisMirror | PlaneMirror if !minus_unit => Scale,
        Scale if minus_unit => PointMirror,
        Scale if unit => Identity,
        _ => form,
    }
}
impl Transform3 {
    /// Sets a general similarity from a 3x4 matrix: the signed cube root of the determinant
    /// is factored out and the rest orthogonalized. A failed orthogonalization leaves the
    /// scale, form and divided matrix assigned, as natively;
    /// `transform.set_values([[1.,0.,0.,1.],[0.,1.,0.,2.],[0.,0.,1.,3.]])?`.
    pub fn set_values(&mut self, rows: [[f64; 4]; 3]) -> Result<(), Failure> {
        let column = |c: usize| Xyz::new(rows[0][c], rows[1][c], rows[2][c]);
        let mut matrix = Matrix3::from_columns(column(0), column(1), column(2));
        let determinant = matrix.determinant();
        if determinant.abs() < f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "gp_Trsf::SetValues, null determinant {determinant}"
            )));
        }
        let scale = if determinant > 0.0 {
            determinant.powf(1.0 / 3.0)
        } else {
            -(-determinant).powf(1.0 / 3.0)
        };
        matrix.divide(scale)?;
        self.scale = scale;
        self.form = Compound;
        self.matrix = matrix;
        self.orthogonalize()?;
        self.offset = column(3);
        Ok(())
    }
    /// Gram-Schmidt on columns then rows (gp_Trsf::Orthogonalize).
    /// Retains scale, form and translation; failures retain the original matrix.
    /// `transform.orthogonalize()?`.
    pub fn orthogonalize(&mut self) -> Result<(), Failure> {
        let mut matrix = self.matrix;
        let [first, second, third] =
            orthonormal_triple([matrix.column(1)?, matrix.column(2)?, matrix.column(3)?])?;
        matrix.set_columns(first, second, third);
        let [first, second, third] =
            orthonormal_triple([matrix.row(1)?, matrix.row(2)?, matrix.row(3)?])?;
        matrix.set_rows(first, second, third);
        self.matrix = matrix;
        Ok(())
    }
}
fn orthonormal_triple([mut first, mut second, mut third]: [Xyz; 3]) -> Result<[Xyz; 3], Failure> {
    first.normalize()?;
    second -= first * second.dot(first);
    second.normalize()?;
    third -= first * third.dot(first) + second * third.dot(second);
    third.normalize()?;
    Ok([first, second, third])
}
