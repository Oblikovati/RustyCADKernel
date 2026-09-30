// Rust translation of FoundationClasses/TKMath/gp/gp_Ax2.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::super::{Axis1, Axis2, Direction3, Failure, Point3, Transform3, Vector3, Xyz};
use rustycad_tkernel::{
    precision,
    standard::dump::{self, DumpOutput},
};
use std::io;
impl Axis2 {
    /// Mirrors X/Y with gp_Dir's axis overload (OCCT 3d097a Y-for-Z coefficient kept) and
    /// rebuilds the normal from them; `frame.mirrored_axis(axis)?`.
    pub fn mirrored_axis(self, axis: Axis1) -> Result<Self, Failure> {
        let y = self.y_direction().mirrored_axis(axis);
        let x = self.x_direction().mirrored_axis(axis);
        let location = self.location().mirrored_axis(axis);
        Ok(Self::from_parts(location, x.crossed(y)?, x, y))
    }
    /// Mirrors about an axis in place; `frame.mirror_axis(axis)?`.
    pub fn mirror_axis(&mut self, axis: Axis1) -> Result<(), Failure> {
        *self = self.mirrored_axis(axis)?;
        Ok(())
    }
    /// Mirrors X/Y and the origin through a plane, then rebuilds the normal; `frame.mirrored_plane(plane)?`.
    pub fn mirrored_plane(self, plane: Self) -> Result<Self, Failure> {
        let y = self.y_direction().mirrored_plane(plane);
        let x = self.x_direction().mirrored_plane(plane);
        let location = self.location().mirrored_plane(plane);
        Ok(Self::from_parts(location, x.crossed(y)?, x, y))
    }
    /// Mirrors through a plane in place; `frame.mirror_plane(plane)?`.
    pub fn mirror_plane(&mut self, plane: Self) -> Result<(), Failure> {
        *self = self.mirrored_plane(plane)?;
        Ok(())
    }
    /// Mirrors through a point in place; `frame.mirror_point(center)`.
    pub fn mirror_point(&mut self, center: Point3) {
        *self = self.mirrored_point(center);
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
    /// Transforms in place; `frame.transform(transform)?`.
    pub fn transform(&mut self, transform: Transform3) -> Result<(), Failure> {
        *self = self.transformed(transform)?;
        Ok(())
    }
    /// Translates in place; `frame.translate(vector)`.
    pub fn translate(&mut self, vector: Vector3) {
        *self = self.translated(vector);
    }
    /// Uses gp_Pnt's add-then-subtract translation of the origin; `frame.translated_between(from,to)`.
    pub fn translated_between(self, from: Point3, to: Point3) -> Self {
        let location = self.location().translated_between(from, to);
        Self::from_parts(
            location,
            self.direction(),
            self.x_direction(),
            self.y_direction(),
        )
    }
    /// Translates between points in place; `frame.translate_between(from,to)`.
    pub fn translate_between(&mut self, from: Point3, to: Point3) {
        *self = self.translated_between(from, to);
    }
    /// Writes the four native vectors; `frame.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        self.axis3().dump_json(output)
    }
    /// Parses with gp_Ax2's commit order: origin first, then each direction; `frame.init_from_json(text,&mut position)?`.
    pub fn init_from_json(&mut self, text: &str, position: &mut i32) -> Result<bool, Failure> {
        let mut local = *position;
        let mut values = [[0.0; 3]; 4];
        if !dump::init_vector(text, "Location", &mut local, &mut values[0])? {
            return Ok(false);
        }
        self.set_location(Point3::from(Xyz::new(
            values[0][0],
            values[0][1],
            values[0][2],
        )));
        for (name, index) in [("Direction", 1), ("XDirection", 2), ("YDirection", 3)] {
            if !dump::init_vector(text, name, &mut local, &mut values[index])? {
                return Ok(false);
            }
        }
        self.store_parsed_directions(&values)?;
        let parsed = direction_of(values[1])?;
        if !self.direction().is_equal(&parsed, precision::confusion()) {
            return Ok(false);
        }
        *position = local;
        Ok(true)
    }
    fn store_parsed_directions(&mut self, values: &[[f64; 3]; 4]) -> Result<(), Failure> {
        let (location, x, y) = (self.location(), self.x_direction(), self.y_direction());
        let normal = direction_of(values[1])?;
        *self = Self::from_parts(location, normal, x, y);
        let x = direction_of(values[2])?;
        *self = Self::from_parts(location, normal, x, y);
        let y = direction_of(values[3])?;
        *self = Self::from_parts(location, normal, x, y);
        Ok(())
    }
}
fn direction_of(values: [f64; 3]) -> Result<Direction3, Failure> {
    Direction3::new(values[0], values[1], values[2])
}
