//! gp_Trsf::DumpJson, gp_Trsf::InitFromJson and the templated gp_Trsf::GetMat4.
use super::super::{Failure, Matrix3, Transform3, TransformForm, Vector3};
use rustycad_tkernel::ncollection::{ElementCast, Mat4};
use rustycad_tkernel::standard::dump::{self, DumpOutput};
use std::io;
impl Transform3 {
    /// Writes `Location`, the row-order `Matrix`, `shape` and `scale`;
    /// `transform.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(output, "Location", self.offset.coordinates())?;
        let rows: [f64; 9] = std::array::from_fn(|i| self.matrix.coefficients()[i / 3][i % 3]);
        dump::dump_vector(output, "Matrix", &rows)?;
        dump::dump_integer(output, "shape", self.form as i32)?;
        dump::dump_real(output, "scale", self.scale)
    }
    /// Reads the `dump_json` fields. As natively, a parsed `Location` is applied through
    /// `SetTranslation` before the matrix is read, so a later failure leaves a translation; a
    /// `shape` outside gp_TrsfForm, which native code would store as-is, is an error;
    /// `transform.init_from_json(text, &mut position)?`.
    pub fn init_from_json(&mut self, text: &str, position: &mut i32) -> Result<bool, Failure> {
        let mut local = *position;
        let mut location = [0.0; 3];
        if !dump::init_vector(text, "Location", &mut local, &mut location)? {
            return Ok(false);
        }
        self.set_translation(Vector3::new(location[0], location[1], location[2]));
        let mut values = [0.0; 9];
        if !dump::init_vector(text, "Matrix", &mut local, &mut values)? {
            return Ok(false);
        }
        self.matrix = Matrix3::new(std::array::from_fn(|r| {
            std::array::from_fn(|c| values[r * 3 + c])
        }));
        let mut shape: i32 = 0;
        if !dump::init_integer_field(text, "shape", &mut local, &mut shape)? {
            return Ok(false);
        }
        self.form = TransformForm::from_native(shape).ok_or_else(|| {
            Failure::out_of_range(&format!(
                "gp_Trsf::InitFromJson - shape {shape} is not a gp_TrsfForm"
            ))
        })?;
        if !dump::init_real_field(text, "scale", &mut local, &mut self.scale)? {
            return Ok(false);
        }
        *position = local;
        Ok(true)
    }
    /// Writes the homogeneous 4x4 matrix (scaled rotation, translation column, `0 0 0 1`
    /// row), converting each value with `static_cast`; an identity form writes the identity
    /// without reading the fields; `transform.get_mat4(&mut matrix)`.
    pub fn get_mat4<T: Copy + From<u8>>(&self, output: &mut Mat4<T>)
    where
        f64: ElementCast<T>,
    {
        if self.form == TransformForm::Identity {
            output.init_identity();
            return;
        }
        for row in 0..3 {
            for column in 0..4 {
                output.set_value(row, column, self.coefficient(row + 1, column + 1).cast());
            }
        }
        for column in 0..3 {
            output.set_value(3, column, T::from(0));
        }
        output.set_value(3, 3, T::from(1));
    }
}
