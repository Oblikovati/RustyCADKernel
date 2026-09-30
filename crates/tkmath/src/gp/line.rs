// Rust translation of FoundationClasses/TKMath/gp/gp_Lin.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Lin metric and transformation operations over an oriented axis.
use super::{Axis1, Direction3, Failure, Point3, Vector3, Xyz};
mod transforms;
/// An unbounded oriented line, initially the Z axis; `Line3::new(origin,direction)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Line3 {
    axis: Axis1,
}
impl Line3 {
    /// Retains a point and unit direction; `Line3::new(origin,direction)`.
    pub const fn new(location: Point3, direction: Direction3) -> Self {
        Self {
            axis: Axis1::new(location, direction),
        }
    }
    /// Creates a line from an axis; `Line3::from_axis(axis)`.
    pub const fn from_axis(axis: Axis1) -> Self {
        Self { axis }
    }
    /// Returns the located axis (gp_Lin::Position); `line.position()`.
    pub const fn position(&self) -> Axis1 {
        self.axis
    }
    /// Returns the origin; `line.location()`.
    pub const fn location(&self) -> Point3 {
        self.axis.location()
    }
    /// Returns the oriented direction; `line.direction()`.
    pub const fn direction(&self) -> Direction3 {
        self.axis.direction()
    }
    /// Replaces the direction; `line.set_direction(Direction3::X)`.
    pub const fn set_direction(&mut self, direction: Direction3) {
        self.axis.set_direction(direction);
    }
    /// Replaces the origin; `line.set_location(point)`.
    pub const fn set_location(&mut self, location: Point3) {
        self.axis.set_location(location);
    }
    /// Replaces the located axis; `line.set_position(axis)`.
    pub const fn set_position(&mut self, axis: Axis1) {
        self.axis = axis;
    }
    /// Reverses the direction; `line.reverse()`.
    pub fn reverse(&mut self) {
        self.axis.reverse();
    }
    /// Returns the line with reversed direction; `line.reversed()`.
    pub fn reversed(self) -> Self {
        Self::from_axis(self.axis.reversed())
    }
    /// Builds the perpendicular line through a point off this line; `line.normal(point)?`.
    pub fn normal(&self, point: Point3) -> Result<Self, Failure> {
        let origin = self.location();
        let toward = Direction3::new(
            point.x() - origin.x(),
            point.y() - origin.y(),
            point.z() - origin.z(),
        )?;
        let direction = self.direction();
        Ok(Self::new(
            point,
            direction.cross_crossed(toward, direction)?,
        ))
    }
    /// Computes the norm of the displacement cross direction; `line.distance(point)`.
    pub fn distance(&self, point: Point3) -> f64 {
        self.square_distance(point).sqrt()
    }
    /// Computes squared point distance; `line.square_distance(point)`.
    pub fn square_distance(&self, point: Point3) -> f64 {
        (point.xyz() - self.location().xyz())
            .crossed(Xyz::from(self.direction()))
            .square_modulus()
    }
    /// Computes line distance, using gp resolution for the parallel branch; the crossing
    /// branch keeps gp_Dir::Crossed's zero-norm raise; `line.distance_line(other)?`.
    pub fn distance_line(&self, other: Self) -> Result<f64, Failure> {
        if self.axis.is_parallel(other.axis, f64::MIN_POSITIVE) {
            return Ok(other.distance(self.location()));
        }
        let normal = self.direction().crossed(other.direction())?;
        Ok(Vector3::between(self.location(), other.location())
            .dot(Vector3::from(normal))
            .abs())
    }
    /// Squares the line distance (gp_Lin::SquareDistance(gp_Lin)); `line.square_distance_line(other)?`.
    pub fn square_distance_line(&self, other: Self) -> Result<f64, Failure> {
        let distance = self.distance_line(other)?;
        Ok(distance * distance)
    }
    /// Tests distance within tolerance; `line.contains(point,1e-7)`.
    pub fn contains(&self, point: Point3, tolerance: f64) -> bool {
        self.distance(point) <= tolerance
    }
    /// Returns the oriented-direction angle; `line.angle(other)`.
    pub fn angle(&self, other: Self) -> f64 {
        self.direction().angle(&other.direction())
    }
}
impl From<Axis1> for Line3 {
    fn from(axis: Axis1) -> Self {
        Self::from_axis(axis)
    }
}
