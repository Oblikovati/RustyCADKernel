// Note: implementation is based on free samples from
// http://www.gamedev.ru/code/articles/?id=4215&page=3
// and maths found in Wikipedia and elsewhere

//! gp_Quaternion rotations between vectors, axis/angle access and matrix assignment.
use super::Quaternion;
use crate::gp::{Failure, Matrix3, Vector3};
impl Quaternion {
    /// The shortest rotation taking `from` to `to`, as `gp_Quaternion(from, to)`;
    /// `Quaternion::from_vectors(a, b)`.
    pub fn from_vectors(from: Vector3, to: Vector3) -> Self {
        let mut quaternion = Self::default();
        quaternion.set_rotation(from, to);
        quaternion
    }
    /// As `from_vectors`, turning about `from x help` for opposite vectors, as
    /// `gp_Quaternion(from, to, help)`; `Quaternion::from_vectors_with_help(a, b, c)`.
    pub fn from_vectors_with_help(from: Vector3, to: Vector3, help: Vector3) -> Self {
        let mut quaternion = Self::default();
        quaternion.set_rotation_with_help(from, to, help);
        quaternion
    }
    /// Halves the angle of the `(from x to, from . to)` quaternion by adding 1 to the
    /// normalized W, as `SetRotation(from, to)`. For opposite vectors the axis is `from`
    /// crossed with X or Z, whichever has the larger component in `from`;
    /// `quaternion.set_rotation(a, b)`.
    pub fn set_rotation(&mut self, from: Vector3, to: Vector3) {
        self.set_half_rotation(from, to);
        if self.w <= f64::MIN_POSITIVE {
            let w = self.w;
            if from.z() * from.z() > from.x() * from.x() {
                self.set(0.0, from.z(), -from.y(), w);
            } else {
                self.set(from.y(), -from.x(), 0.0, w);
            }
        }
        self.normalize();
    }
    /// `SetRotation(from, to, help)`: opposite vectors turn about `from x help`;
    /// `quaternion.set_rotation_with_help(a, b, c)`.
    pub fn set_rotation_with_help(&mut self, from: Vector3, to: Vector3, help: Vector3) {
        self.set_half_rotation(from, to);
        if self.w <= f64::MIN_POSITIVE {
            let axis = from.crossed(help);
            let w = self.w;
            self.set(axis.x(), axis.y(), axis.z(), w);
        }
        self.normalize();
    }
    fn set_half_rotation(&mut self, from: Vector3, to: Vector3) {
        let cross = from.crossed(to);
        self.set(cross.x(), cross.y(), cross.z(), from.dot(to));
        self.normalize();
        self.w += 1.0;
    }
    /// Rotation about a normalized axis, as `SetVectorAndAngle`; a zero axis raises
    /// `Standard_ConstructionError`; `quaternion.set_vector_and_angle(axis, 0.5)?`.
    pub fn set_vector_and_angle(&mut self, axis: Vector3, angle: f64) -> Result<(), Failure> {
        *self = Self::from_axis_angle(axis, angle)?;
        Ok(())
    }
    /// The unit axis and the angle in `[-2pi, 2pi]`; a vector part at or below gp::Resolution
    /// gives `(0, 0, 1)` and 0, as `GetVectorAndAngle`; `quaternion.vector_and_angle()`.
    pub fn vector_and_angle(&self) -> (Vector3, f64) {
        let length = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if length > f64::MIN_POSITIVE {
            let inverse = 1.0 / length;
            let axis = Vector3::new(self.x * inverse, self.y * inverse, self.z * inverse);
            if self.w < 0.0 {
                return (axis, 2.0 * (-length).atan2(-self.w));
            }
            return (axis, 2.0 * length.atan2(self.w));
        }
        (Vector3::new(0.0, 0.0, 1.0), 0.0)
    }
    /// Replaces the quaternion from a rotation matrix, as `SetMatrix`; `quaternion.set_matrix(m)`.
    pub fn set_matrix(&mut self, matrix: Matrix3) {
        *self = Self::from_matrix(matrix);
    }
}
