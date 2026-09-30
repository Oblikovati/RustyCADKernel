// Rust translation of FoundationClasses/TKMath/gp/gp_Ax3.hxx/.cxx members at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::super::direction::{CheckedProducts, DirectionProducts, UncheckedProducts};
use super::super::{Axis1, Axis2, Axis3, Failure, Point3, Transform3, Vector3};
impl Axis3 {
    /// Copies a right-handed frame, as gp_Ax3(const gp_Ax2&) does; `Axis3::from_axis2(frame)`.
    pub const fn from_axis2(frame: Axis2) -> Self {
        frame.axis3()
    }
    /// Builds the right-handed frame, reversing the normal of an indirect frame; `frame.ax2()?`.
    pub fn ax2(&self) -> Result<Axis2, Failure> {
        self.ax2_using::<CheckedProducts>()
    }
    /// gp_Ax3::Ax2 as compiled with `No_Exception` (ElSLib); `frame.ax2_unchecked()`.
    pub(crate) fn ax2_unchecked(&self) -> Axis2 {
        let Ok(frame) = self.ax2_using::<UncheckedProducts>();
        frame
    }
    fn ax2_using<P: DirectionProducts>(&self) -> Result<Axis2, P::Error> {
        let mut normal = self.direction;
        let direct = P::crossed(self.x_direction, self.y_direction)?.dot(self.direction) > 0.0;
        if !direct {
            normal.reverse();
        }
        Axis2::frame_using::<P>(self.location, normal, self.x_direction)
    }
    /// Returns the main axis (origin and normal); `frame.axis()`.
    pub fn axis(&self) -> Axis1 {
        Axis1::new(self.location, self.direction)
    }
    /// Returns the angle between main directions; `frame.angle(other)`.
    pub fn angle(&self, other: Self) -> f64 {
        self.axis().angle(other.axis())
    }
    /// Moves only the origin; `frame.set_location(point)`.
    pub fn set_location(&mut self, location: Point3) {
        self.location = location;
    }
    /// Requires both origins in both planes and parallel normals; `frame.is_coplanar(other,1e-7,1e-12)`.
    pub fn is_coplanar(&self, other: Self, linear: f64, angular: f64) -> bool {
        let offset = Vector3::between(self.location, other.location);
        let first = Vector3::from(self.direction).dot(offset).abs();
        let second = Vector3::from(other.direction).dot(offset).abs();
        first <= linear && second <= linear && self.axis().is_parallel(other.axis(), angular)
    }
    /// Requires the axis in this plane and normal to the frame normal; `frame.is_coplanar_axis(axis,1e-7,1e-12)`.
    pub fn is_coplanar_axis(&self, axis: Axis1, linear: f64, angular: f64) -> bool {
        let offset = Vector3::between(self.location, axis.location());
        let first = Vector3::from(self.direction).dot(offset).abs();
        let second = Vector3::from(axis.direction()).crossed(offset).magnitude();
        first <= linear && second <= linear && self.axis().is_normal(axis, angular)
    }
    /// Mirrors the main axis through a point and reverses X/Y; `frame.mirrored_point(center)`.
    pub fn mirrored_point(mut self, center: Point3) -> Self {
        self.location.mirror_point(center);
        self.direction.reverse();
        self.x_direction.reverse();
        self.y_direction.reverse();
        self
    }
    /// Uses gp_Dir's axis overload for X/Y (OCCT 3d097a Y-for-Z coefficient) and gp_Ax1's
    /// direction overload for the normal; `frame.mirrored_axis(axis)`.
    pub fn mirrored_axis(mut self, axis: Axis1) -> Self {
        self.y_direction.mirror_axis(axis);
        self.x_direction.mirror_axis(axis);
        self.location.mirror_axis(axis);
        self.direction.mirror_direction(axis.direction());
        self
    }
    /// Mirrors through a point in place; `frame.mirror_point(center)`.
    pub fn mirror_point(&mut self, center: Point3) {
        *self = self.mirrored_point(center);
    }
    /// Mirrors about an axis in place; `frame.mirror_axis(axis)`.
    pub fn mirror_axis(&mut self, axis: Axis1) {
        *self = self.mirrored_axis(axis);
    }
    /// Mirrors through a plane in place; `frame.mirror_plane(plane)`.
    pub fn mirror_plane(&mut self, plane: Axis2) {
        *self = self.mirrored_plane(plane);
    }
    /// Rotates in place; `frame.rotate(axis,angle)?`.
    pub fn rotate(&mut self, axis: Axis1, angle: f64) -> Result<(), Failure> {
        *self = self.rotated(axis, angle)?;
        Ok(())
    }
    /// Scales in place; `frame.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point3, factor: f64) {
        *self = self.scaled(center, factor);
    }
    /// Transforms in place; `frame.transform(transform)`.
    pub fn transform(&mut self, transform: Transform3) {
        *self = self.transformed(transform);
    }
    /// Translates in place; `frame.translate(vector)`.
    pub fn translate(&mut self, vector: Vector3) {
        *self = self.translated(vector);
    }
    /// Translates by gp_Vec(from,to), unlike gp_Ax2's add-then-subtract; `frame.translated_between(from,to)`.
    pub fn translated_between(self, from: Point3, to: Point3) -> Self {
        self.translated(Vector3::between(from, to))
    }
    /// Translates between points in place; `frame.translate_between(from,to)`.
    pub fn translate_between(&mut self, from: Point3, to: Point3) {
        *self = self.translated_between(from, to);
    }
}
