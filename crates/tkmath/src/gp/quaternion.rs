// Note: implementation is based on free samples from
// http://www.gamedev.ru/code/articles/?id=4215&page=3
// and maths found in Wikipedia and elsewhere

//! gp_Quaternion rotation representations with native arithmetic ordering.
use super::{Failure, Vector3};
mod sequence;
pub use sequence::EulerSequence;
mod algebra;
mod euler;
mod interpolation;
mod matrix;
mod rotation;
pub use interpolation::{QuaternionNLerp, QuaternionSLerp};
/// Quaternion components stored as x,y,z,w; `Quaternion::new(0.0,0.0,0.0,1.0)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}
impl Default for Quaternion {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
}
impl Quaternion {
    /// Retains supplied components without normalization; `Quaternion::new(x,y,z,w)`.
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }
    /// Replaces components in native x,y,z,w order; `quaternion.set(x,y,z,w)`.
    pub fn set(&mut self, x: f64, y: f64, z: f64, w: f64) {
        *self = Self::new(x, y, z, w);
    }
    /// Returns all stored components; `quaternion.components()`.
    pub const fn components(&self) -> [f64; 4] {
        [self.x, self.y, self.z, self.w]
    }
    /// Computes the squared norm with native left association; `quaternion.square_norm()`.
    pub fn square_norm(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }
    /// Computes the quaternion norm; `quaternion.norm()`.
    pub fn norm(&self) -> f64 {
        self.square_norm().sqrt()
    }
    /// Scales each component; `quaternion.scaled(0.5)`.
    pub fn scaled(self, factor: f64) -> Self {
        Self::new(
            self.x * factor,
            self.y * factor,
            self.z * factor,
            self.w * factor,
        )
    }
    /// Normalizes the vector before applying half-angle sine; `Quaternion::from_axis_angle(axis,angle)?`.
    pub fn from_axis_angle(axis: Vector3, angle: f64) -> Result<Self, Failure> {
        let normal = axis.normalized()?;
        let half = 0.5 * angle;
        let sine = half.sin();
        Ok(Self::new(
            normal.x() * sine,
            normal.y() * sine,
            normal.z() * sine,
            half.cos(),
        ))
    }
    /// Uses native quaternion-vector products and squared-norm division; `quaternion.multiply_vector(vector)`.
    pub fn multiply_vector(&self, vector: Vector3) -> Vector3 {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        let q = Self::new(
            vector.x() * w + vector.z() * y - vector.y() * z,
            vector.y() * w + vector.x() * z - vector.z() * x,
            vector.z() * w + vector.y() * x - vector.x() * y,
            vector.x() * x + vector.y() * y + vector.z() * z,
        );
        Vector3::new(
            w * q.x + x * q.w + y * q.z - z * q.y,
            w * q.y + y * q.w + z * q.x - x * q.z,
            w * q.z + z * q.w + x * q.y - y * q.x,
        ) * (1.0 / self.square_norm())
    }
}
