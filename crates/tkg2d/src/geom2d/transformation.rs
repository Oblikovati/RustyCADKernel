// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Transformation.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_Transformation`: a shared gp_Trsf2d.
use super::cell::GeometryCell;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::{Axis2d, Point2, Transform2, TransformForm, Vector2};

/// A transient similarity of the plane; `Handle::new(Geom2dTransformation::default())`.
#[derive(Default)]
pub struct Geom2dTransformation {
    parent: StandardTransient,
    state: GeometryCell<Transform2>,
}
impl_transient!(
    Geom2dTransformation,
    "Geom2d_Transformation",
    StandardTransient,
    parent
);
impl Clone for Geom2dTransformation {
    fn clone(&self) -> Self {
        Self::new(self.trsf2d())
    }
}
impl Geom2dTransformation {
    /// Transformation from a gp one (`Geom2d_Transformation(gp_Trsf2d)`);
    /// `Geom2dTransformation::new(transform)`.
    pub fn new(transform: Transform2) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(transform),
        }
    }
    /// Point symmetry (`SetMirror(gp_Pnt2d)`); `t.set_mirror_point(p)`.
    pub fn set_mirror_point(&self, point: Point2) {
        self.state.write().set_mirror_point(point);
    }
    /// Axis symmetry (`SetMirror(gp_Ax2d)`); `t.set_mirror_axis(axis)`.
    pub fn set_mirror_axis(&self, axis: Axis2d) {
        self.state.write().set_mirror_axis(axis);
    }
    /// Rotation (`SetRotation`); `t.set_rotation(p, 0.5)`.
    pub fn set_rotation(&self, point: Point2, angle: f64) {
        self.state.write().set_rotation(point, angle);
    }
    /// Scaling (`SetScale`); `t.set_scale(p, 2.0)`.
    pub fn set_scale(&self, point: Point2, factor: f64) {
        self.state.write().set_scale(point, factor);
    }
    /// Change of coordinate system (`SetTransformation(FromSystem1, ToSystem2)`);
    /// `t.set_transformation_between(from, to)`.
    pub fn set_transformation_between(&self, from: Axis2d, to: Axis2d) {
        self.state.write().set_transformation_between(from, to);
    }
    /// Change to a coordinate system (`SetTransformation(ToSystem)`); `t.set_transformation_to(axis)`.
    pub fn set_transformation_to(&self, axis: Axis2d) {
        self.state.write().set_transformation_to(axis);
    }
    /// Translation (`SetTranslation(gp_Vec2d)`); `t.set_translation(v)`.
    pub fn set_translation(&self, vector: Vector2) {
        self.state.write().set_translation(vector);
    }
    /// Translation between points (`SetTranslation(P1, P2)`); `t.set_translation_between(a, b)`.
    pub fn set_translation_between(&self, from: Point2, to: Point2) {
        self.state.write().set_translation_between(from, to);
    }
    /// Replace by a gp transformation (`SetTrsf2d`); `t.set_trsf2d(transform)`.
    pub fn set_trsf2d(&self, transform: Transform2) {
        *self.state.write() = transform;
    }
    /// Whether the determinant is negative (`IsNegative`); `t.is_negative()`.
    pub fn is_negative(&self) -> bool {
        self.state.read().is_negative()
    }
    /// The form (`Form`); `t.form()`.
    pub fn form(&self) -> TransformForm {
        self.state.read().form()
    }
    /// The scale factor (`ScaleFactor`); `t.scale_factor()`.
    pub fn scale_factor(&self) -> f64 {
        self.state.read().scale_factor()
    }
    /// The gp transformation (`Trsf2d`); `t.trsf2d()`.
    pub fn trsf2d(&self) -> Transform2 {
        *self.state.read()
    }
    /// Coefficient at row 1..2, column 1..3 (`Value`); `t.value(1, 3)?`.
    pub fn value(&self, row: i32, col: i32) -> Result<f64, Failure> {
        self.state.read().value(row, col)
    }
    /// Invert in place; fails for a null scale (`Invert`); `t.invert()?`.
    pub fn invert(&self) -> Result<(), Failure> {
        self.state.write().invert()
    }
    /// Inverse as a new object (`Inverted`); `t.inverted()?`.
    pub fn inverted(&self) -> Result<Handle<Geom2dTransformation>, Failure> {
        let inverse: Transform2 = self.trsf2d().inverted()?;
        Ok(Handle::new(Self::new(inverse)))
    }
    /// `self * other` as a new object (`Multiplied`, `*`); `t.multiplied(&other)`.
    pub fn multiplied(&self, other: &Geom2dTransformation) -> Handle<Geom2dTransformation> {
        let own: Transform2 = self.trsf2d();
        Handle::new(Self::new(own.multiplied(other.trsf2d())))
    }
    /// `self = self * other` (`Multiply`, `*=`); `t.multiply(&other)`.
    pub fn multiply(&self, other: &Geom2dTransformation) {
        let right: Transform2 = other.trsf2d();
        self.state.write().multiply(right);
    }
    /// Raise to the power `n` in place (`Power`); `t.power(3)?`.
    pub fn power(&self, n: i32) -> Result<(), Failure> {
        self.state.write().power(n)
    }
    /// Power as a new object (`Powered`); `t.powered(3)?`.
    pub fn powered(&self, n: i32) -> Result<Handle<Geom2dTransformation>, Failure> {
        let mut temp: Transform2 = self.trsf2d();
        temp.power(n)?;
        Ok(Handle::new(Self::new(temp)))
    }
    /// `self = other * self` (`PreMultiply`); `t.pre_multiply(&other)`.
    pub fn pre_multiply(&self, other: &Geom2dTransformation) {
        let left: Transform2 = other.trsf2d();
        self.state.write().pre_multiply(left);
    }
    /// Transform coordinates (`Transforms(X, Y)`); `let (x, y) = t.transforms(1.0, 2.0);`.
    pub fn transforms(&self, x: f64, y: f64) -> (f64, f64) {
        self.trsf2d().transforms_coordinates(x, y)
    }
    /// A new object with the same transformation (`Copy`); `t.copy()`.
    pub fn copy(&self) -> Handle<Geom2dTransformation> {
        Handle::new(self.clone())
    }
}
