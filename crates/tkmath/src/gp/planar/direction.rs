// Rust translation of FoundationClasses/TKMath/gp/gp_Dir2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Dir2d: a planar unit direction.
use super::{Failure, Vector2, Xy, construction, oriented_angle};
use rustycad_tkernel::standard::dump::{self, DumpOutput};
use std::{io, ops};
mod geometry;
/// gp_Dir2d::D, the axis-aligned planar directions; `Direction2::from_standard(StandardDirection2::Y)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandardDirection2 {
    /// Positive X axis `(1, 0)`.
    X,
    /// Positive Y axis `(0, 1)`.
    Y,
    /// Negative X axis `(-1, 0)`.
    NegX,
    /// Negative Y axis `(0, -1)`.
    NegY,
}
/// A normalized planar direction; native mirrored/parsed states may depart from unit length; `Direction2::new(1.0,0.0)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Direction2 {
    coordinates: Xy,
}
impl Default for Direction2 {
    fn default() -> Self {
        Self::X
    }
}
impl From<StandardDirection2> for Direction2 {
    fn from(direction: StandardDirection2) -> Self {
        Self::from_standard(direction)
    }
}
impl Direction2 {
    /// Positive X axis; `Direction2::X`.
    pub const X: Self = Self::from_standard(StandardDirection2::X);
    /// Positive Y axis; `Direction2::Y`.
    pub const Y: Self = Self::from_standard(StandardDirection2::Y);
    /// Negative X axis; `Direction2::NEG_X`.
    pub const NEG_X: Self = Self::from_standard(StandardDirection2::NegX);
    /// Negative Y axis; `Direction2::NEG_Y`.
    pub const NEG_Y: Self = Self::from_standard(StandardDirection2::NegY);
    /// gp_Dir2d(gp_Dir2d::D): exact unit components; `Direction2::from_standard(StandardDirection2::NegX)`.
    pub const fn from_standard(direction: StandardDirection2) -> Self {
        let (x, y) = match direction {
            StandardDirection2::X => (1.0, 0.0),
            StandardDirection2::Y => (0.0, 1.0),
            StandardDirection2::NegX => (-1.0, 0.0),
            StandardDirection2::NegY => (0.0, -1.0),
        };
        Self {
            coordinates: Xy::new(x, y),
        }
    }
    /// Normalizes using the native squared-norm fast path; `Direction2::new(3.0,4.0)?`.
    pub fn new(x: f64, y: f64) -> Result<Self, Failure> {
        Self::normalized_with(Xy::new(x, y), "gp_Dir2d() - input vector has zero norm")
    }
    /// Normalizes with the native fast path and a caller-specific zero-norm message;
    /// `Direction2::normalized_with(xy,"gp_Dir2d::SetCoord() - result vector has zero norm")?`.
    pub(in crate::gp) fn normalized_with(xy: Xy, message: &str) -> Result<Self, Failure> {
        let squared = xy.square_modulus();
        // 1 +/- gp::Resolution() rounds to 1 and Resolution^2 underflows to 0.
        if squared == 1.0 {
            return Ok(Self { coordinates: xy });
        }
        if squared <= 0.0 {
            return Err(construction(&format!(
                "{message}: ({}, {})",
                xy.x(),
                xy.y()
            )));
        }
        Ok(Self {
            coordinates: xy / squared.sqrt(),
        })
    }
    /// Constructs from planar algebra; `Direction2::from_xy(xy)?`.
    pub fn from_xy(xy: Xy) -> Result<Self, Failure> {
        Self::new(xy.x(), xy.y())
    }
    /// Constructs from a vector; `Direction2::from_vector(vector)?`.
    pub fn from_vector(vector: Vector2) -> Result<Self, Failure> {
        Self::from_xy(vector.xy())
    }
    /// Stores coordinates the caller has already derived from unit directions.
    pub(in crate::gp) const fn from_stored(coordinates: Xy) -> Self {
        Self { coordinates }
    }
    /// Returns coordinate algebra; `direction.xy()`.
    pub const fn xy(&self) -> Xy {
        self.coordinates
    }
    /// Reads X; `direction.x()`.
    pub const fn x(&self) -> f64 {
        self.coordinates.x()
    }
    /// Reads Y; `direction.y()`.
    pub const fn y(&self) -> f64 {
        self.coordinates.y()
    }
    /// Reads a one-based coordinate; `direction.coord(1)?`.
    pub fn coord(&self, index: i32) -> Result<f64, Failure> {
        self.coordinates.coord(index)
    }
    /// Replaces both coordinates and renormalizes, keeping the old value on a zero norm;
    /// `direction.set_coordinates(3.0,4.0)?`.
    pub fn set_coordinates(&mut self, x: f64, y: f64) -> Result<(), Failure> {
        *self = Self::normalized_with(
            Xy::new(x, y),
            "gp_Dir2d::SetCoord() - result vector has zero norm",
        )?;
        Ok(())
    }
    /// Replaces one coordinate and renormalizes; `direction.set_coord(2,1.0)?`.
    pub fn set_coord(&mut self, index: i32, value: f64) -> Result<(), Failure> {
        match index {
            1 => self.set_x(value),
            2 => self.set_y(value),
            _ => Err(Failure::out_of_range(&format!(
                "gp_Dir2d::SetCoord() - index {index} is out of range [1, 2]"
            ))),
        }
    }
    /// Replaces X and renormalizes; `direction.set_x(1.0)?`.
    pub fn set_x(&mut self, x: f64) -> Result<(), Failure> {
        self.set_coordinates(x, self.y())
    }
    /// Replaces Y and renormalizes; `direction.set_y(1.0)?`.
    pub fn set_y(&mut self, y: f64) -> Result<(), Failure> {
        self.set_coordinates(self.x(), y)
    }
    /// Replaces both coordinates from planar algebra; `direction.set_xy(xy)?`.
    pub fn set_xy(&mut self, xy: Xy) -> Result<(), Failure> {
        self.set_coordinates(xy.x(), xy.y())
    }
}
impl Direction2 {
    /// Computes the signed native angle, including -PI for opposite X axes; `direction.angle(other)`.
    #[allow(clippy::approx_constant)]
    pub fn angle(self, other: Self) -> f64 {
        oriented_angle(self.dot(other), self.crossed(other), 0.70710678118655)
    }
    /// Compares angular separation; `direction.is_equal(other,1e-12)`.
    pub fn is_equal(self, other: Self, tolerance: f64) -> bool {
        self.angle(other).abs() <= tolerance
    }
    /// Tests both signs of a right angle; `direction.is_normal(other,1e-12)`.
    pub fn is_normal(self, other: Self, tolerance: f64) -> bool {
        (std::f64::consts::FRAC_PI_2 - self.angle(other).abs()).abs() <= tolerance
    }
    /// Tests opposite directions; `direction.is_opposite(other,1e-12)`.
    pub fn is_opposite(self, other: Self, tolerance: f64) -> bool {
        std::f64::consts::PI - self.angle(other).abs() <= tolerance
    }
    /// Accepts parallel or opposite directions; `direction.is_parallel(other,1e-12)`.
    pub fn is_parallel(self, other: Self, tolerance: f64) -> bool {
        let angle = self.angle(other).abs();
        angle <= tolerance || std::f64::consts::PI - angle <= tolerance
    }
    /// Computes scalar product; `direction.dot(other)`.
    pub fn dot(self, other: Self) -> f64 {
        self.coordinates.dot(other.coordinates)
    }
    /// Computes signed planar cross product; `direction.crossed(other)`.
    pub fn crossed(self, other: Self) -> f64 {
        self.coordinates.crossed(other.coordinates)
    }
    /// Negates stored components without normalization; `direction.reversed()`.
    pub fn reversed(self) -> Self {
        Self::from_stored(self.coordinates.reversed())
    }
    /// Reverses in place; `direction.reverse()`.
    pub fn reverse(&mut self) {
        *self = self.reversed();
    }
    /// Writes the native `gp_Dir2d` vector fragment; `direction.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(output, "gp_Dir2d", self.coordinates.coordinates())
    }
}
/// `gp_Dir2d::operator^` is the signed planar cross product.
impl ops::BitXor for Direction2 {
    type Output = f64;
    fn bitxor(self, other: Self) -> f64 {
        self.crossed(other)
    }
}
/// `gp_Dir2d::operator*` is the scalar product.
impl ops::Mul for Direction2 {
    type Output = f64;
    fn mul(self, other: Self) -> f64 {
        self.dot(other)
    }
}
impl ops::Neg for Direction2 {
    type Output = Self;
    fn neg(self) -> Self {
        self.reversed()
    }
}
