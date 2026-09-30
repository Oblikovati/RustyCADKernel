//! gp_Pln symmetries, motions and JSON dump, each applied to the stored gp_Ax3.
use super::super::{Axis1, Axis2, Failure, Plane3, Point3, Transform3, Vector3};
use rustycad_tkernel::standard::dump::{self, DumpOutput, DumpStream};
use std::io;
impl Plane3 {
    /// Mirrors the frame through a point; `plane.mirrored_point(center)`.
    pub fn mirrored_point(self, center: Point3) -> Self {
        Self::from_position(self.position.mirrored_point(center))
    }
    /// Mirrors through a point in place; `plane.mirror_point(center)`.
    pub fn mirror_point(&mut self, center: Point3) {
        self.position.mirror_point(center);
    }
    /// Mirrors the frame about an axis; `plane.mirrored_axis(axis)`.
    pub fn mirrored_axis(self, axis: Axis1) -> Self {
        Self::from_position(self.position.mirrored_axis(axis))
    }
    /// Mirrors about an axis in place; `plane.mirror_axis(axis)`.
    pub fn mirror_axis(&mut self, axis: Axis1) {
        self.position.mirror_axis(axis);
    }
    /// Mirrors the complete frame, preserving native handedness; `plane.mirrored_plane(frame)`.
    pub fn mirrored_plane(self, frame: Axis2) -> Self {
        Self::from_position(self.position.mirrored_plane(frame))
    }
    /// Mirrors through a plane in place; `plane.mirror_plane(frame)`.
    pub fn mirror_plane(&mut self, frame: Axis2) {
        self.position.mirror_plane(frame);
    }
    /// Rotates the frame around an oriented line; `plane.rotated(axis,angle)?`.
    pub fn rotated(self, axis: Axis1, angle: f64) -> Result<Self, Failure> {
        Ok(Self::from_position(self.position.rotated(axis, angle)?))
    }
    /// Rotates in place; `plane.rotate(axis,angle)?`.
    pub fn rotate(&mut self, axis: Axis1, angle: f64) -> Result<(), Failure> {
        self.position.rotate(axis, angle)
    }
    /// Scales the frame with native negative-scale direction reversal; `plane.scaled(center,2.0)`.
    pub fn scaled(self, center: Point3, factor: f64) -> Self {
        Self::from_position(self.position.scaled(center, factor))
    }
    /// Scales in place; `plane.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point3, factor: f64) {
        self.position.scale(center, factor);
    }
    /// Transforms the complete stored frame; `plane.transformed(transform)`.
    pub fn transformed(self, transform: Transform3) -> Self {
        Self::from_position(self.position.transformed(transform))
    }
    /// Transforms in place; `plane.transform(transform)`.
    pub fn transform(&mut self, transform: Transform3) {
        self.position.transform(transform);
    }
    /// Translates the stored origin; `plane.translated(vector)`.
    pub fn translated(self, vector: Vector3) -> Self {
        Self::from_position(self.position.translated(vector))
    }
    /// Translates in place; `plane.translate(vector)`.
    pub fn translate(&mut self, vector: Vector3) {
        self.position.translate(vector);
    }
    /// Translates by gp_Vec(from,to) through gp_Ax3; `plane.translated_between(from,to)`.
    pub fn translated_between(self, from: Point3, to: Point3) -> Self {
        Self::from_position(self.position.translated_between(from, to))
    }
    /// Translates between points in place; `plane.translate_between(from,to)`.
    pub fn translate_between(&mut self, from: Point3, to: Point3) {
        self.position.translate_between(from, to);
    }
    /// Writes `"Position": {frame}` unless the depth is 0 (OCCT_DUMP_FIELD_VALUES_DUMPED);
    /// `plane.dump_json(&mut stream,-1)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput, depth: i32) -> io::Result<()> {
        if depth == 0 {
            return Ok(());
        }
        let mut position = DumpStream::default();
        self.position.dump_json(&mut position)?;
        dump::dump_key_to_class(output, "Position", position.text())
    }
}
