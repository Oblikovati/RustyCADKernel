//! gp_Lin2d mirror, rotation, scale, transformation and translation members.
use super::super::{Axis2d, Point2, Transform2, Vector2};
use super::Line2;
impl Line2 {
    /// Mirrors through a point; `line.mirrored_point(center)`.
    pub fn mirrored_point(self, center: Point2) -> Self {
        Self::from(self.axis.mirrored_point(center))
    }
    /// Mirrors through a point in place; `line.mirror_point(center)`.
    pub fn mirror_point(&mut self, center: Point2) {
        *self = self.mirrored_point(center);
    }
    /// Mirrors about an axis; `line.mirrored_axis(axis)`.
    pub fn mirrored_axis(self, axis: Axis2d) -> Self {
        Self::from(self.axis.mirrored_axis(axis))
    }
    /// Mirrors about an axis in place; `line.mirror_axis(axis)`.
    pub fn mirror_axis(&mut self, axis: Axis2d) {
        *self = self.mirrored_axis(axis);
    }
    /// Rotates about a center; `line.rotated(center,0.5)`.
    pub fn rotated(self, center: Point2, angle: f64) -> Self {
        Self::from(self.axis.rotated(center, angle))
    }
    /// Rotates in place; `line.rotate(center,0.5)`.
    pub fn rotate(&mut self, center: Point2, angle: f64) {
        *self = self.rotated(center, angle);
    }
    /// Scales about a center, reversing for a negative factor; `line.scaled(center,2.0)`.
    pub fn scaled(self, center: Point2, factor: f64) -> Self {
        Self::from(self.axis.scaled(center, factor))
    }
    /// Scales in place; `line.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point2, factor: f64) {
        *self = self.scaled(center, factor);
    }
    /// Transforms the axis; `line.transformed(transform)`.
    pub fn transformed(self, transform: Transform2) -> Self {
        Self::from(self.axis.transformed(transform))
    }
    /// Transforms in place; `line.transform(transform)`.
    pub fn transform(&mut self, transform: Transform2) {
        *self = self.transformed(transform);
    }
    /// Translates by a vector; `line.translated(vector)`.
    pub fn translated(self, vector: Vector2) -> Self {
        Self::from(self.axis.translated(vector))
    }
    /// Translates in place; `line.translate(vector)`.
    pub fn translate(&mut self, vector: Vector2) {
        *self = self.translated(vector);
    }
    /// Copying form: translates by `gp_Vec2d(from, to)`, i.e. `to - from` first, unlike the
    /// in-place form; `line.translated_between(from,to)`.
    pub fn translated_between(self, from: Point2, to: Point2) -> Self {
        self.translated(Vector2::between(from, to))
    }
    /// In-place form: adds `to` and then subtracts `from`, as gp_Ax2d::Translate(P1,P2) does;
    /// `line.translate_between(from,to)`.
    pub fn translate_between(&mut self, from: Point2, to: Point2) {
        self.axis.translate_between(from, to);
    }
}
