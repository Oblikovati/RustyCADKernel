// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/gp/gp_Pnt2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Pnt2d: a planar point.
use super::{Axis2d, Failure, Transform2, TransformForm, Vector2, Xy};
use rustycad_tkernel::standard::dump::{self, DumpOutput};
use std::io;
/// A planar point, initially the origin; `Point2::new(1.0,2.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point2 {
    coordinates: Xy,
}
impl Point2 {
    /// Stores unmodified coordinates; `Point2::new(1.0,2.0)`.
    pub const fn new(x: f64, y: f64) -> Self {
        Self {
            coordinates: Xy::new(x, y),
        }
    }
    /// Returns component algebra (gp_Pnt2d::XY and gp_Pnt2d::Coord()); `point.xy()`.
    pub const fn xy(&self) -> Xy {
        self.coordinates
    }
    /// Borrows the writable coordinates (gp_Pnt2d::ChangeCoord); `point.change_coord().set_x(1.0)`.
    pub const fn change_coord(&mut self) -> &mut Xy {
        &mut self.coordinates
    }
    /// Reads X; `point.x()`.
    pub const fn x(&self) -> f64 {
        self.coordinates.x()
    }
    /// Reads Y; `point.y()`.
    pub const fn y(&self) -> f64 {
        self.coordinates.y()
    }
    /// Reads a one-based coordinate; `point.coord(1)?`.
    pub fn coord(&self, index: i32) -> Result<f64, Failure> {
        self.coordinates.coord(index)
    }
    /// Replaces a one-based coordinate; `point.set_coord(1,2.0)?`.
    pub fn set_coord(&mut self, index: i32, value: f64) -> Result<(), Failure> {
        self.coordinates.set_coord(index, value)
    }
    /// Replaces X; `point.set_x(1.0)`.
    pub fn set_x(&mut self, x: f64) {
        self.coordinates.set_x(x);
    }
    /// Replaces Y; `point.set_y(1.0)`.
    pub fn set_y(&mut self, y: f64) {
        self.coordinates.set_y(y);
    }
    /// Replaces both components; `point.set_coordinates(1.0,2.0)`.
    pub fn set_coordinates(&mut self, x: f64, y: f64) {
        self.coordinates = Xy::new(x, y);
    }
    /// Replaces the coordinates from planar algebra; `point.set_xy(xy)`.
    pub fn set_xy(&mut self, xy: Xy) {
        self.coordinates = xy;
    }
    /// Computes squared distance; `point.square_distance(other)`.
    pub fn square_distance(self, other: Self) -> f64 {
        (self.coordinates - other.coordinates).square_modulus()
    }
    /// Computes Euclidean distance; `point.distance(other)`.
    pub fn distance(self, other: Self) -> f64 {
        self.square_distance(other).sqrt()
    }
    /// Uses distance less than or equal to tolerance; `point.is_equal(other,1e-7)`.
    pub fn is_equal(self, other: Self, tolerance: f64) -> bool {
        self.distance(other) <= tolerance
    }
    /// Writes the native `gp_Pnt2d` vector fragment; `point.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(output, "gp_Pnt2d", self.coordinates.coordinates())
    }
}
impl Point2 {
    /// Adds a vector; `point.translated(vector)`.
    pub fn translated(self, vector: Vector2) -> Self {
        Self::from(self.coordinates + vector.xy())
    }
    /// Translates in place; `point.translate(vector)`.
    pub fn translate(&mut self, vector: Vector2) {
        *self = self.translated(vector);
    }
    /// Adds the destination then subtracts the origin, as gp_Pnt2d::Translate does; `point.translated_between(from,to)`.
    pub fn translated_between(self, from: Self, to: Self) -> Self {
        Self::from(self.coordinates + to.coordinates - from.coordinates)
    }
    /// Translates between points in place; `point.translate_between(from,to)`.
    pub fn translate_between(&mut self, from: Self, to: Self) {
        *self = self.translated_between(from, to);
    }
    /// Scales about a center with native multiplication order; `point.scaled(center,2.0)`.
    pub fn scaled(self, center: Self, factor: f64) -> Self {
        Self::from(self.coordinates * factor + center.coordinates * (1.0 - factor))
    }
    /// Scales in place; `point.scale(center,2.0)`.
    pub fn scale(&mut self, center: Self, factor: f64) {
        *self = self.scaled(center, factor);
    }
    /// Rotates about a center using gp_Trsf2d arithmetic; `point.rotated(center,angle)`.
    pub fn rotated(self, center: Self, angle: f64) -> Self {
        Self::from(Transform2::rotation(center, angle).apply_coordinates(self.coordinates))
    }
    /// Rotates in place; `point.rotate(center,angle)`.
    pub fn rotate(&mut self, center: Self, angle: f64) {
        *self = self.rotated(center, angle);
    }
    /// Mirrors through a center; `point.mirrored_point(center)`.
    pub fn mirrored_point(self, center: Self) -> Self {
        Self::from(self.coordinates.reversed() + center.coordinates * 2.0)
    }
    /// Mirrors through a center in place; `point.mirror_point(center)`.
    pub fn mirror_point(&mut self, center: Self) {
        *self = self.mirrored_point(center);
    }
    /// Mirrors through an oriented axis; `point.mirrored_axis(axis)`.
    pub fn mirrored_axis(self, axis: Axis2d) -> Self {
        Self::from(Transform2::mirror_axis(axis).apply_coordinates(self.coordinates))
    }
    /// Mirrors through an axis in place; `point.mirror_axis(axis)`.
    pub fn mirror_axis(&mut self, axis: Axis2d) {
        *self = self.mirrored_axis(axis);
    }
    /// gp_Pnt2d::Transform: identity, translation, scale and point-mirror shortcuts, otherwise
    /// gp_Trsf2d::Transforms; `point.transformed(transform)`.
    pub fn transformed(self, transform: Transform2) -> Self {
        let offset = transform.translation_part();
        Self::from(match transform.form() {
            TransformForm::Identity => return self,
            TransformForm::Translation => self.coordinates + offset,
            TransformForm::Scale => self.coordinates * transform.scale_factor() + offset,
            TransformForm::PointMirror => self.coordinates.reversed() + offset,
            _ => transform.apply_coordinates(self.coordinates),
        })
    }
    /// Transforms in place; `point.transform(transform)`.
    pub fn transform(&mut self, transform: Transform2) {
        *self = self.transformed(transform);
    }
}
impl From<Xy> for Point2 {
    fn from(coordinates: Xy) -> Self {
        Self { coordinates }
    }
}
