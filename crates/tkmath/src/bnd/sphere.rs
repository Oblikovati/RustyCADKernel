//! Native bounding sphere behavior at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::gp::Xyz;
/// A bounding sphere with independent entity validity and integer parameters; `BoundingSphere::new(center,1.0,0,0)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BoundingSphere {
    center: Xyz,
    radius: f64,
    valid: bool,
    u: i32,
    v: i32,
}
impl BoundingSphere {
    /// Retains radius without validation and initially marks the entity invalid; `BoundingSphere::new(center,radius,u,v)`.
    pub const fn new(center: Xyz, radius: f64, u: i32, v: i32) -> Self {
        Self {
            center,
            radius,
            u,
            v,
            valid: false,
        }
    }
    /// Returns the integer U parameter; `sphere.u()`.
    pub const fn u(&self) -> i32 {
        self.u
    }
    /// Returns the integer V parameter; `sphere.v()`.
    pub const fn v(&self) -> i32 {
        self.v
    }
    /// Returns entity validity, independently of radius; `sphere.is_valid()`.
    pub const fn is_valid(&self) -> bool {
        self.valid
    }
    /// Changes entity validity; `sphere.set_valid(true)`.
    pub fn set_valid(&mut self, valid: bool) {
        self.valid = valid;
    }
    /// Borrows the center; `sphere.center()`.
    pub const fn center(&self) -> &Xyz {
        &self.center
    }
    /// Returns the stored radius; `sphere.radius()`.
    pub const fn radius(&self) -> f64 {
        self.radius
    }
    /// Computes center distance, not surface distance; `sphere.distance(point)`.
    pub fn distance(&self, point: Xyz) -> f64 {
        (point - self.center).modulus()
    }
    /// Computes squared center distance; `sphere.square_distance(point)`.
    pub fn square_distance(&self, point: Xyz) -> f64 {
        (point - self.center).square_modulus()
    }
    /// Returns minimum and maximum native distance estimates; `sphere.distances(point)`.
    pub fn distances(&self, point: Xyz) -> (f64, f64) {
        let distance = self.distance(point);
        let minimum = if distance - self.radius < 0.0 {
            0.0
        } else {
            distance - self.radius
        };
        (minimum, distance + self.radius)
    }
    /// Preserves OCCT's d²±r² estimates, rather than squaring Distances; `sphere.square_distances(point)`.
    pub fn square_distances(&self, point: Xyz) -> (f64, f64) {
        let squared = self.square_distance(point);
        let radius_squared = self.radius * self.radius;
        let minimum = if squared < radius_squared {
            0.0
        } else {
            squared - radius_squared
        };
        (minimum, squared + radius_squared)
    }
    /// Returns the center projection, center distance, inside and success flags; `sphere.project(point,&mut projected,&mut distance,&mut inside)`.
    pub fn project(
        &self,
        point: Xyz,
        projected: &mut Xyz,
        distance: &mut f64,
        inside: &mut bool,
    ) -> bool {
        *projected = self.center;
        *distance = (point - *projected).modulus();
        *inside = true;
        true
    }
    /// Expands or copies a sphere with native metadata rules; `sphere.add(&other)`.
    pub fn add(&mut self, other: &Self) {
        if self.radius < 0.0 {
            *self = *other;
            return;
        }
        let distance = (self.center - other.center).modulus();
        if self.radius + distance <= other.radius {
            *self = *other;
            return;
        }
        if other.radius + distance <= self.radius {
            return;
        }
        self.expand(other, distance);
    }
    fn expand(&mut self, other: &Self, distance: f64) {
        let radius = (distance + self.radius + other.radius) * 0.5;
        let parameter = (radius - self.radius) / distance;
        self.center = self.center * (1.0 - parameter) + other.center * parameter;
        self.radius = radius;
        self.valid = false;
    }
    /// Compares squared center distance against the squared radius sum; `sphere.is_out_sphere(&other)`.
    pub fn is_out_sphere(&self, other: &Self) -> bool {
        (self.center - other.center).square_modulus()
            > (self.radius + other.radius) * (self.radius + other.radius)
    }
    /// Tightens the distance bound only for valid entities; `sphere.is_out_point(point,&mut limit)`.
    pub fn is_out_point(&self, point: Xyz, maximum: &mut f64) -> bool {
        let (low, high) = self.distances(point);
        if low > *maximum {
            return true;
        }
        if self.valid && high < *maximum {
            *maximum = high;
        }
        false
    }
    /// Uses native multiplication order for the squared diameter; `sphere.square_extent()`.
    pub fn square_extent(&self) -> f64 {
        4.0 * self.radius * self.radius
    }
}
