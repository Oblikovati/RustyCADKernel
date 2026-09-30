// Rust translation of FoundationClasses/TKMath/gp/gp_Circ.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::placement::{axis2_curve_placement, scaled_length};
use super::{Axis2, Failure, Point3, Transform3, Vector3, construction};
/// A located circle retaining frame orientation; `Circle3::new(frame,5.0)?`.
#[derive(Clone, Copy, Debug)]
pub struct Circle3 {
    position: Axis2,
    radius: f64,
}
impl Default for Circle3 {
    fn default() -> Self {
        Self {
            position: Axis2::default(),
            radius: f64::MAX,
        }
    }
}
axis2_curve_placement!(Circle3);
impl Circle3 {
    /// Rejects negative radii while accepting zero; `Circle3::new(frame,radius)?`.
    pub fn new(position: Axis2, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Err(construction(&format!(
                "gp_Circ() - radius should be positive number, got {radius}"
            )));
        }
        Ok(Self { position, radius })
    }
    /// Builds a circle without the radius check, as ElSLib's `No_Exception` build does;
    /// `Circle3::from_parts_unchecked(frame,-1.0)`.
    pub(crate) const fn from_parts_unchecked(position: Axis2, radius: f64) -> Self {
        Self { position, radius }
    }
    /// Changes radius after the native check; `circle.set_radius(2.0)?`.
    pub fn set_radius(&mut self, radius: f64) -> Result<(), Failure> {
        if radius < 0.0 {
            return Err(construction(&format!(
                "gp_Circ::SetRadius() - radius should be positive number, got {radius}"
            )));
        }
        self.radius = radius;
        Ok(())
    }
    /// Moves the centre only; `circle.set_location(point)`.
    pub fn set_location(&mut self, location: Point3) {
        self.position.set_location(location);
    }
    /// Returns the stored radius; `circle.radius()`.
    pub const fn radius(&self) -> f64 {
        self.radius
    }
    /// Uses native multiplication order; `circle.area()`.
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    /// Computes native circumference; `circle.length()`.
    pub fn length(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
    /// Resolves the point into stored frame directions; `circle.square_distance(point)`.
    pub fn square_distance(&self, point: Point3) -> f64 {
        let displacement = Vector3::between(self.location(), point);
        let x = displacement.dot(Vector3::from(self.position.x_direction()));
        let y = displacement.dot(Vector3::from(self.position.y_direction()));
        let z = displacement.dot(Vector3::from(self.position.direction()));
        let radial = (x * x + y * y).sqrt() - self.radius;
        radial * radial + z * z
    }
    /// Computes minimum distance to the circumference; `circle.distance(point)`.
    pub fn distance(&self, point: Point3) -> f64 {
        self.square_distance(point).sqrt()
    }
    /// Includes points within the given circumference tolerance; `circle.contains(point,1e-7)`.
    pub fn contains(&self, point: Point3, tolerance: f64) -> bool {
        self.distance(point) <= tolerance
    }
    /// Scales radius and frame, retaining signed zero in radius; `circle.scaled(center,-2.0)`.
    pub fn scaled(mut self, center: Point3, factor: f64) -> Self {
        self.scale(center, factor);
        self
    }
    /// Scales in place; `circle.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point3, factor: f64) {
        self.radius = scaled_length(self.radius, factor);
        self.position.scale(center, factor);
    }
    /// Applies the transform's scale to radius and rebuilds frame orientation; `circle.transformed(transform)?`.
    pub fn transformed(mut self, transform: Transform3) -> Result<Self, Failure> {
        self.transform(transform)?;
        Ok(self)
    }
    /// Transforms in place; `circle.transform(transform)?`.
    pub fn transform(&mut self, transform: Transform3) -> Result<(), Failure> {
        self.radius = scaled_length(self.radius, transform.scale_factor());
        self.position.transform(transform)
    }
}
