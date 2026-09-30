//! Cartesian algebra preserving gp_XYZ operation order and tolerance rules.
use super::{Failure, construction, cross, dot};
use rustycad_tkernel::standard::dump::{self, DumpOutput};
use std::{io, ops};
/// Three Cartesian scalars for algebra rather than geometric direction; `Xyz::new(1.0,2.0,3.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Xyz {
    coordinates: [f64; 3],
}
impl Xyz {
    /// Constructs unmodified coordinates; `Xyz::new(1.0,2.0,3.0)`.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            coordinates: [x, y, z],
        }
    }
    /// Borrows contiguous coordinates; `xyz.coordinates()`.
    pub const fn coordinates(&self) -> &[f64; 3] {
        &self.coordinates
    }
    /// Borrows writable contiguous coordinates; `xyz.coordinates_mut()[0]=2.0`.
    pub const fn coordinates_mut(&mut self) -> &mut [f64; 3] {
        &mut self.coordinates
    }
    /// Reads X; `xyz.x()`.
    pub const fn x(&self) -> f64 {
        self.coordinates[0]
    }
    /// Reads Y; `xyz.y()`.
    pub const fn y(&self) -> f64 {
        self.coordinates[1]
    }
    /// Reads Z; `xyz.z()`.
    pub const fn z(&self) -> f64 {
        self.coordinates[2]
    }
    /// Replaces X; `xyz.set_x(1.0)`.
    pub const fn set_x(&mut self, value: f64) {
        self.coordinates[0] = value;
    }
    /// Replaces Y; `xyz.set_y(1.0)`.
    pub const fn set_y(&mut self, value: f64) {
        self.coordinates[1] = value;
    }
    /// Replaces Z; `xyz.set_z(1.0)`.
    pub const fn set_z(&mut self, value: f64) {
        self.coordinates[2] = value;
    }
    /// Replaces the coordinate triple; `xyz.set_coordinates(1.0,2.0,3.0)`.
    pub const fn set_coordinates(&mut self, x: f64, y: f64, z: f64) {
        self.coordinates = [x, y, z];
    }
    /// Reads a one-based coordinate; `xyz.coord(1)?`.
    pub fn coord(&self, index: i32) -> Result<f64, Failure> {
        Ok(self.coordinates[coordinate_index(index)?])
    }
    /// Borrows a one-based coordinate; `*xyz.change_coord(1)?=2.0`.
    pub fn change_coord(&mut self, index: i32) -> Result<&mut f64, Failure> {
        Ok(&mut self.coordinates[coordinate_index(index)?])
    }
    /// Sets a one-based coordinate; `xyz.set_coord(1,2.0)?`.
    pub fn set_coord(&mut self, index: i32, value: f64) -> Result<(), Failure> {
        *self.change_coord(index)? = value;
        Ok(())
    }
    /// Computes squared norm in native evaluation order; `xyz.square_modulus()`.
    pub fn square_modulus(&self) -> f64 {
        dot(self.coordinates, self.coordinates)
    }
    /// Computes Euclidean norm; `xyz.modulus()`.
    pub fn modulus(&self) -> f64 {
        self.square_modulus().sqrt()
    }
    /// Uses strict per-coordinate tolerances, including false for zero tolerance; `xyz.is_equal(&other,1e-7)`.
    pub fn is_equal(&self, other: &Self, tolerance: f64) -> bool {
        (0..3).all(|i| (self.coordinates[i] - other.coordinates[i]).abs() < tolerance)
    }
    /// Computes a vector product without normalization; `xyz.crossed(other)`.
    pub fn crossed(self, other: Self) -> Self {
        Self {
            coordinates: cross(self.coordinates, other.coordinates),
        }
    }
    /// Replaces this triple with its vector product; `xyz.cross(other)`.
    pub fn cross(&mut self, other: Self) {
        *self = self.crossed(other);
    }
    /// Computes cross-product squared norm; `xyz.cross_square_magnitude(other)`.
    pub fn cross_square_magnitude(self, other: Self) -> f64 {
        self.crossed(other).square_modulus()
    }
    /// Computes cross-product norm; `xyz.cross_magnitude(other)`.
    pub fn cross_magnitude(self, other: Self) -> f64 {
        self.cross_square_magnitude(other).sqrt()
    }
    /// Preserves nested cross-product rounding; `xyz.cross_crossed(first,second)`.
    pub fn cross_crossed(self, first: Self, second: Self) -> Self {
        self.crossed(first.crossed(second))
    }
    /// Replaces this triple with a nested vector product; `xyz.cross_cross(first,second)`.
    pub fn cross_cross(&mut self, first: Self, second: Self) {
        *self = self.cross_crossed(first, second);
    }
    /// Computes scalar product; `xyz.dot(other)`.
    pub fn dot(self, other: Self) -> f64 {
        dot(self.coordinates, other.coordinates)
    }
    /// Computes scalar triple product; `xyz.dot_cross(first,second)`.
    pub fn dot_cross(self, first: Self, second: Self) -> f64 {
        self.dot(first.crossed(second))
    }
    /// Multiplies coordinate pairs rather than taking a dot product; `xyz.multiplied_components(other)`.
    pub fn multiplied_components(self, other: Self) -> Self {
        Self {
            coordinates: std::array::from_fn(|i| self.coordinates[i] * other.coordinates[i]),
        }
    }
    /// Divides by the norm with the original exception message; `xyz.normalized()?`.
    pub fn normalized(self) -> Result<Self, Failure> {
        self.normalize_with_message("gp_XYZ::Normalized() - vector has zero norm")
    }
    /// Normalizes in place, preserving this triple on failure; `xyz.normalize()?`.
    pub fn normalize(&mut self) -> Result<(), Failure> {
        *self = self.normalize_with_message("gp_XYZ::Normalize() - vector has zero norm")?;
        Ok(())
    }
    fn normalize_with_message(self, message: &str) -> Result<Self, Failure> {
        let norm = self.modulus();
        if norm <= f64::MIN_POSITIVE {
            return Err(construction(message));
        }
        Ok(self / norm)
    }
    /// Reverses all component signs; `xyz.reversed()`.
    pub fn reversed(self) -> Self {
        Self {
            coordinates: self.coordinates.map(|v| -v),
        }
    }
    /// Reverses this triple in place; `xyz.reverse()`.
    pub fn reverse(&mut self) {
        *self = self.reversed();
    }
    /// Forms two weighted terms without a zero accumulator; `xyz.set_linear_form(2.0,a,3.0,b)`.
    pub fn set_linear_form(&mut self, a: f64, first: Self, b: f64, second: Self) {
        *self = first * a + second * b;
    }
    /// Sets `a*first + b*second + c*third`, summed left to right;
    /// `xyz.set_linear_form_three(1.0,a,2.0,b,3.0,c)`.
    pub fn set_linear_form_three(&mut self, weights: [f64; 3], terms: [Self; 3]) {
        *self = terms[0] * weights[0] + terms[1] * weights[1] + terms[2] * weights[2];
    }
    /// Sets `a*first + b*second + c*third + offset`;
    /// `xyz.set_linear_form_three_offset([1.0,2.0,3.0],[a,b,c],d)`.
    pub fn set_linear_form_three_offset(
        &mut self,
        weights: [f64; 3],
        terms: [Self; 3],
        offset: Self,
    ) {
        self.set_linear_form_three(weights, terms);
        *self += offset;
    }
    /// Sets `a*first + b*second + offset`; `xyz.set_linear_form_offset(2.0,a,3.0,b,c)`.
    pub fn set_linear_form_offset(
        &mut self,
        a: f64,
        first: Self,
        b: f64,
        second: Self,
        offset: Self,
    ) {
        *self = first * a + second * b + offset;
    }
    /// Sets `a*first + second`; `xyz.set_scaled_sum(2.0,a,b)`.
    pub fn set_scaled_sum(&mut self, a: f64, first: Self, second: Self) {
        *self = first * a + second;
    }
    /// Sets `first + second`; `xyz.set_sum(a,b)`.
    pub fn set_sum(&mut self, first: Self, second: Self) {
        *self = first + second;
    }
    /// Writes a gp_XYZ fragment; `xyz.dump_json(&mut stream)?`.
    pub fn dump_json(&self, stream: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(stream, "gp_XYZ", &self.coordinates)
    }
    /// Reads with native partial writes and one-based cursor; `xyz.init_from_json(text,&mut position)?`.
    pub fn init_from_json(&mut self, text: &str, position: &mut i32) -> Result<bool, Failure> {
        dump::init_vector(text, "gp_XYZ", position, &mut self.coordinates)
    }
}
fn coordinate_index(index: i32) -> Result<usize, Failure> {
    if !(1..=3).contains(&index) {
        return Err(Failure::out_of_range(""));
    }
    Ok(index as usize - 1)
}
impl ops::Add for Xyz {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            coordinates: std::array::from_fn(|i| self.coordinates[i] + rhs.coordinates[i]),
        }
    }
}
impl ops::Sub for Xyz {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            coordinates: std::array::from_fn(|i| self.coordinates[i] - rhs.coordinates[i]),
        }
    }
}
impl ops::Mul<f64> for Xyz {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            coordinates: self.coordinates.map(|v| v * rhs),
        }
    }
}
impl ops::Mul<Xyz> for f64 {
    type Output = Xyz;
    fn mul(self, rhs: Xyz) -> Xyz {
        rhs * self
    }
}
impl ops::Mul for Xyz {
    type Output = f64;
    fn mul(self, rhs: Self) -> f64 {
        self.dot(rhs)
    }
}
impl ops::Div<f64> for Xyz {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self {
            coordinates: self.coordinates.map(|v| v / rhs),
        }
    }
}
impl ops::BitXor for Xyz {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        self.crossed(rhs)
    }
}
impl ops::AddAssign for Xyz {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl ops::SubAssign for Xyz {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl ops::MulAssign<f64> for Xyz {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}
impl ops::MulAssign for Xyz {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.multiplied_components(rhs);
    }
}
impl ops::DivAssign<f64> for Xyz {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}
impl ops::BitXorAssign for Xyz {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.cross(rhs);
    }
}

impl From<super::Point3> for Xyz {
    fn from(point: super::Point3) -> Self {
        Self {
            coordinates: *point.coordinates(),
        }
    }
}
impl From<super::Direction3> for Xyz {
    fn from(direction: super::Direction3) -> Self {
        Self {
            coordinates: *direction.coordinates(),
        }
    }
}
impl From<Xyz> for super::Point3 {
    fn from(xyz: Xyz) -> Self {
        Self {
            coordinates: xyz.coordinates,
        }
    }
}
