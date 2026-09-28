// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of FoundationClasses/TKMath/gp/gp_Ax22d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Ax22d planar coordinate system whose Y direction encodes the orientation sense.
use super::{Axis2d, Direction2, Failure, Point2, Transform2, Vector2};
use rustycad_tkernel::standard::dump::{self, DumpOutput};
use std::io;
/// Origin with X and a quarter-turned Y direction; `Axis22d::new(origin,Direction2::X,true)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Axis22d {
    point: Point2,
    y_direction: Direction2,
    x_direction: Direction2,
}
impl Default for Axis22d {
    fn default() -> Self {
        Self {
            point: Point2::default(),
            y_direction: Direction2::Y,
            x_direction: Direction2::X,
        }
    }
}
impl Axis22d {
    /// Keeps X and rebuilds Y on the side given by the sign of `x ^ y`; only the sense of `y`
    /// is used, so parallel inputs are not rejected; `Axis22d::with_directions(origin,x,y)?`.
    pub fn with_directions(point: Point2, x: Direction2, y: Direction2) -> Result<Self, Failure> {
        Self::new(point, x, x.crossed(y) >= 0.0)
    }
    /// Builds Y as X turned counterclockwise when `sense` is true, raising
    /// Standard_ConstructionError when X has zero norm; `Axis22d::new(origin,x,true)?`.
    pub fn new(point: Point2, x: Direction2, sense: bool) -> Result<Self, Failure> {
        Ok(Self {
            point,
            y_direction: x.quarter_turn(sense)?,
            x_direction: x,
        })
    }
    /// gp_Ax22d(P, V, sense) for the native `noexcept` callers that pass a stored unit X,
    /// whose quarter turn cannot raise; `Axis22d::from_unit(origin,x,true)`.
    pub(in crate::gp) fn from_unit(point: Point2, x: Direction2, sense: bool) -> Self {
        Self {
            point,
            y_direction: x.quarter_turn_unit(sense),
            x_direction: x,
        }
    }
    /// Uses the axis as origin and X direction; `Axis22d::from_axis(axis,true)?`.
    pub fn from_axis(axis: Axis2d, sense: bool) -> Result<Self, Failure> {
        Self::new(axis.location(), axis.direction(), sense)
    }
    /// Copies another frame; `frame.set_axis(other)`.
    pub fn set_axis(&mut self, other: Self) {
        *self = other;
    }
    /// Replaces origin and X, keeping the current sense; on the native raise the origin and
    /// X are already replaced while Y is kept; `frame.set_x_axis(axis)?`.
    pub fn set_x_axis(&mut self, axis: Axis2d) -> Result<(), Failure> {
        let sense = self.is_counter_clockwise();
        self.point = axis.location();
        self.x_direction = axis.direction();
        self.y_direction = self.x_direction.quarter_turn(sense)?;
        Ok(())
    }
    /// Replaces origin and Y, deriving X with the current sense; on the native raise the
    /// origin and Y are already replaced while X is kept; `frame.set_y_axis(axis)?`.
    pub fn set_y_axis(&mut self, axis: Axis2d) -> Result<(), Failure> {
        let sense = self.is_counter_clockwise();
        self.point = axis.location();
        self.y_direction = axis.direction();
        self.x_direction = self.y_direction.quarter_turn(!sense)?;
        Ok(())
    }
    /// Replaces the origin; `frame.set_location(point)`.
    pub fn set_location(&mut self, point: Point2) {
        self.point = point;
    }
    /// Replaces X, keeping the current sense; `frame.set_x_direction(direction)?`.
    pub fn set_x_direction(&mut self, direction: Direction2) -> Result<(), Failure> {
        let sense = self.is_counter_clockwise();
        self.x_direction = direction;
        self.y_direction = direction.quarter_turn(sense)?;
        Ok(())
    }
    /// Replaces Y, deriving X with the current sense; `frame.set_y_direction(direction)?`.
    pub fn set_y_direction(&mut self, direction: Direction2) -> Result<(), Failure> {
        let sense = self.is_counter_clockwise();
        self.y_direction = direction;
        self.x_direction = direction.quarter_turn(!sense)?;
        Ok(())
    }
    fn is_counter_clockwise(&self) -> bool {
        self.x_direction.crossed(self.y_direction) >= 0.0
    }
    /// Returns the X axis; `frame.x_axis()`.
    pub fn x_axis(&self) -> Axis2d {
        Axis2d::new(self.point, self.x_direction)
    }
    /// Returns the Y axis; `frame.y_axis()`.
    pub fn y_axis(&self) -> Axis2d {
        Axis2d::new(self.point, self.y_direction)
    }
    /// Returns the origin; `frame.location()`.
    pub const fn location(&self) -> Point2 {
        self.point
    }
    /// Returns X; `frame.x_direction()`.
    pub const fn x_direction(&self) -> Direction2 {
        self.x_direction
    }
    /// Returns Y; `frame.y_direction()`.
    pub const fn y_direction(&self) -> Direction2 {
        self.y_direction
    }
}
impl Axis22d {
    /// Mirrors the origin and reverses both directions; `frame.mirrored_point(center)`.
    pub fn mirrored_point(mut self, center: Point2) -> Self {
        self.point = self.point.mirrored_point(center);
        self.x_direction.reverse();
        self.y_direction.reverse();
        self
    }
    /// Mirrors through a point in place; `frame.mirror_point(center)`.
    pub fn mirror_point(&mut self, center: Point2) {
        *self = self.mirrored_point(center);
    }
    /// Mirrors both directions and the origin about an axis; `frame.mirrored_axis(axis)`.
    pub fn mirrored_axis(mut self, axis: Axis2d) -> Self {
        self.y_direction = self.y_direction.mirrored_axis(axis);
        self.x_direction = self.x_direction.mirrored_axis(axis);
        self.point = self.point.mirrored_axis(axis);
        self
    }
    /// Mirrors about an axis in place; `frame.mirror_axis(axis)`.
    pub fn mirror_axis(&mut self, axis: Axis2d) {
        *self = self.mirrored_axis(axis);
    }
    /// Rotates the origin about a center and both directions by the angle; `frame.rotated(center,angle)`.
    pub fn rotated(mut self, center: Point2, angle: f64) -> Self {
        self.point = self.point.rotated(center, angle);
        self.x_direction = self.x_direction.rotated(angle);
        self.y_direction = self.y_direction.rotated(angle);
        self
    }
    /// Rotates in place; `frame.rotate(center,angle)`.
    pub fn rotate(&mut self, center: Point2, angle: f64) {
        *self = self.rotated(center, angle);
    }
    /// Scales the origin and reverses both directions for negative factors; `frame.scaled(center,-1.0)`.
    pub fn scaled(mut self, center: Point2, factor: f64) -> Self {
        self.point = self.point.scaled(center, factor);
        if factor < 0.0 {
            self.x_direction.reverse();
            self.y_direction.reverse();
        }
        self
    }
    /// Scales in place; `frame.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point2, factor: f64) {
        *self = self.scaled(center, factor);
    }
    /// Transforms the origin and each direction independently; `frame.transformed(transform)`.
    pub fn transformed(mut self, transform: Transform2) -> Self {
        self.point = self.point.transformed(transform);
        self.x_direction = self.x_direction.transformed(transform);
        self.y_direction = self.y_direction.transformed(transform);
        self
    }
    /// Transforms in place; `frame.transform(transform)`.
    pub fn transform(&mut self, transform: Transform2) {
        *self = self.transformed(transform);
    }
    /// Translates the origin; `frame.translated(vector)`.
    pub fn translated(mut self, vector: Vector2) -> Self {
        self.point = self.point.translated(vector);
        self
    }
    /// Translates in place; `frame.translate(vector)`.
    pub fn translate(&mut self, vector: Vector2) {
        *self = self.translated(vector);
    }
    /// Translates the origin with gp_Pnt2d's add-then-subtract order; `frame.translated_between(from,to)`.
    pub fn translated_between(mut self, from: Point2, to: Point2) -> Self {
        self.point = self.point.translated_between(from, to);
        self
    }
    /// Translates between points in place; `frame.translate_between(from,to)`.
    pub fn translate_between(&mut self, from: Point2, to: Point2) {
        *self = self.translated_between(from, to);
    }
    /// Writes Location, XAxis and YAxis vectors; `frame.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(output, "Location", self.point.xy().coordinates())?;
        dump::dump_vector(output, "XAxis", self.x_direction.xy().coordinates())?;
        dump::dump_vector(output, "YAxis", self.y_direction.xy().coordinates())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gp::Xy;

    // A zero direction cannot be built through the public API, so the native
    // gp_Dir2d::SetCoord raise is exercised on a stored zero vector.
    #[test]
    fn zero_x_direction_raises_the_native_construction_error() {
        let zero = Direction2::from_stored(Xy::new(0.0, 0.0));
        let failure = Axis22d::new(Point2::default(), zero, true).unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_ConstructionError");
        assert!(
            failure.message().contains("(-0, 0)"),
            "{}",
            failure.message()
        );
        let axis = Axis2d::new(Point2::new(1.0, 2.0), zero);
        assert!(Axis22d::from_axis(axis, false).is_err());
        assert!(Axis22d::with_directions(Point2::default(), zero, Direction2::Y).is_err());
        let mut frame = Axis22d::default();
        assert!(frame.set_x_axis(axis).is_err());
        // Native partial update: origin and X are stored before Y's SetCoord raises.
        assert_eq!(frame.location(), Point2::new(1.0, 2.0));
        assert_eq!(frame.y_direction(), Direction2::Y);
        assert!(frame.set_y_direction(zero).is_err());
        assert!(frame.set_x_direction(zero).is_err());
        assert!(Axis22d::default().set_y_axis(axis).is_err());
    }
}
