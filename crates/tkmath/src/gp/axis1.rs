// Rust translation of FoundationClasses/TKMath/gp/gp_Ax1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Ax1 oriented line for rotations and axial symmetries.
use super::{Axis2, Direction3, Failure, Point3, StandardDirection3, Transform3, Vector3, Xyz};
use rustycad_tkernel::standard::dump::{self, DumpOutput};
use std::io;
/// An origin and oriented unit axis; `Axis1::new(Point3::default(),Direction3::Z)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Axis1 {
    location: Point3,
    direction: Direction3,
}
impl Default for Axis1 {
    fn default() -> Self {
        Self::new(Point3::default(), Direction3::Z)
    }
}
impl Axis1 {
    /// Retains the supplied origin and direction; `Axis1::new(origin,normal)`.
    pub const fn new(location: Point3, direction: Direction3) -> Self {
        Self {
            location,
            direction,
        }
    }
    /// gp_Ax1(gp_Pnt, gp_Dir::D); `Axis1::with_standard(origin,StandardDirection3::X)`.
    pub const fn with_standard(location: Point3, direction: StandardDirection3) -> Self {
        Self::new(location, Direction3::from_standard(direction))
    }
    /// gp_Ax1(gp_Dir::D) at the origin, replacing gp::OX/OY/OZ; `Axis1::from_standard(StandardDirection3::Z)`.
    pub const fn from_standard(direction: StandardDirection3) -> Self {
        Self::with_standard(Point3::new(0.0, 0.0, 0.0), direction)
    }
    /// Returns the origin; `axis.location()`.
    pub const fn location(&self) -> Point3 {
        self.location
    }
    /// Returns the unit direction; `axis.direction()`.
    pub const fn direction(&self) -> Direction3 {
        self.direction
    }
    /// Replaces the direction without touching the origin; `axis.set_direction(Direction3::X)`.
    pub const fn set_direction(&mut self, direction: Direction3) {
        self.direction = direction;
    }
    /// Replaces the origin; `axis.set_location(Point3::new(1.0,0.0,0.0))`.
    pub const fn set_location(&mut self, location: Point3) {
        self.location = location;
    }
    /// Requires equal directions and both mutual line distances within tolerance; `axis.is_coaxial(other,1e-12,1e-7)`.
    pub fn is_coaxial(&self, other: Self, angular: f64, linear: f64) -> bool {
        let first = (self.location.xyz() - other.location.xyz())
            .crossed(Xyz::from(other.direction))
            .modulus();
        let second = (other.location.xyz() - self.location.xyz())
            .crossed(Xyz::from(self.direction))
            .modulus();
        self.direction.is_equal(&other.direction, angular) && first <= linear && second <= linear
    }
    /// Compares directions with a right angle; `axis.is_normal(other,1e-12)`.
    pub fn is_normal(&self, other: Self, angular: f64) -> bool {
        self.direction.is_normal(other.direction, angular)
    }
    /// Compares directions with opposite orientation; `axis.is_opposite(other,1e-12)`.
    pub fn is_opposite(&self, other: Self, angular: f64) -> bool {
        self.direction.is_opposite(other.direction, angular)
    }
    /// Accepts parallel or antiparallel directions; `axis.is_parallel(other,1e-12)`.
    pub fn is_parallel(&self, other: Self, angular: f64) -> bool {
        self.direction.is_parallel(other.direction, angular)
    }
    /// Returns the unsigned angle between directions; `axis.angle(other)`.
    pub fn angle(&self, other: Self) -> f64 {
        self.direction.angle(&other.direction)
    }
    /// Reverses the direction in place; `axis.reverse()`.
    pub fn reverse(&mut self) {
        self.direction.reverse();
    }
    /// Returns the axis with reversed direction; `axis.reversed()`.
    pub fn reversed(self) -> Self {
        Self::new(self.location, self.direction.reversed())
    }
}
impl Axis1 {
    /// Mirrors the origin through a point and reverses the direction; `axis.mirrored_point(center)`.
    pub fn mirrored_point(self, center: Point3) -> Self {
        Self::new(
            self.location.mirrored_point(center),
            self.direction.reversed(),
        )
    }
    /// Mirrors through a point in place; `axis.mirror_point(center)`.
    pub fn mirror_point(&mut self, center: Point3) {
        *self = self.mirrored_point(center);
    }
    /// Uses gp_Dir's direction overload for the axis direction; `axis.mirrored_axis(symmetry)`.
    pub fn mirrored_axis(self, symmetry: Self) -> Self {
        Self::new(
            self.location.mirrored_axis(symmetry),
            self.direction.mirrored_direction(symmetry.direction),
        )
    }
    /// Mirrors about an axis in place; `axis.mirror_axis(symmetry)`.
    pub fn mirror_axis(&mut self, symmetry: Self) {
        *self = self.mirrored_axis(symmetry);
    }
    /// Mirrors through the frame's XY plane; `axis.mirrored_plane(frame)`.
    pub fn mirrored_plane(self, frame: Axis2) -> Self {
        Self::new(
            self.location.mirrored_plane(frame),
            self.direction.mirrored_plane(frame),
        )
    }
    /// Mirrors through a plane in place; `axis.mirror_plane(frame)`.
    pub fn mirror_plane(&mut self, frame: Axis2) {
        *self = self.mirrored_plane(frame);
    }
    /// Rotates origin and direction by their native paths; `axis.rotated(pivot,angle)?`.
    pub fn rotated(self, pivot: Self, angle: f64) -> Result<Self, Failure> {
        Ok(Self::new(
            self.location.rotated(pivot, angle)?,
            self.direction.rotated(pivot, angle)?,
        ))
    }
    /// Rotates in place; `axis.rotate(pivot,angle)?`.
    pub fn rotate(&mut self, pivot: Self, angle: f64) -> Result<(), Failure> {
        *self = self.rotated(pivot, angle)?;
        Ok(())
    }
    /// Scales the origin and reverses the direction for negative factors; `axis.scaled(center,-2.0)`.
    pub fn scaled(self, center: Point3, factor: f64) -> Self {
        let mut direction = self.direction;
        if factor < 0.0 {
            direction.reverse();
        }
        Self::new(self.location.scaled(center, factor), direction)
    }
    /// Scales in place; `axis.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point3, factor: f64) {
        *self = self.scaled(center, factor);
    }
    /// Transforms origin and direction independently; `axis.transformed(transform)`.
    pub fn transformed(self, transform: Transform3) -> Self {
        Self::new(
            self.location.transformed(transform),
            self.direction.transformed(transform),
        )
    }
    /// Transforms in place; `axis.transform(transform)`.
    pub fn transform(&mut self, transform: Transform3) {
        *self = self.transformed(transform);
    }
    /// Translates only the origin; `axis.translated(vector)`.
    pub fn translated(self, vector: Vector3) -> Self {
        Self::new(self.location.translated(vector), self.direction)
    }
    /// Translates in place; `axis.translate(vector)`.
    pub fn translate(&mut self, vector: Vector3) {
        *self = self.translated(vector);
    }
    /// Uses gp_Pnt's add-then-subtract translation; `axis.translated_between(from,to)`.
    pub fn translated_between(self, from: Point3, to: Point3) -> Self {
        Self::new(self.location.translated_between(from, to), self.direction)
    }
    /// Translates between two points in place; `axis.translate_between(from,to)`.
    pub fn translate_between(&mut self, from: Point3, to: Point3) {
        *self = self.translated_between(from, to);
    }
    /// Writes the native Location and Direction vectors; `axis.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(output, "Location", self.location.coordinates())?;
        dump::dump_vector(output, "Direction", self.direction.coordinates())
    }
    /// Parses into the stored origin directly, so a failed parse can leave a partial origin;
    /// `axis.init_from_json(text,&mut position)?`.
    pub fn init_from_json(&mut self, text: &str, position: &mut i32) -> Result<bool, Failure> {
        let mut local = *position;
        if !dump::init_vector(text, "Location", &mut local, &mut self.location.coordinates)? {
            return Ok(false);
        }
        let mut direction = [0.0; 3];
        if !dump::init_vector(text, "Direction", &mut local, &mut direction)? {
            return Ok(false);
        }
        self.direction = Direction3::new(direction[0], direction[1], direction[2])?;
        *position = local;
        Ok(true)
    }
}
