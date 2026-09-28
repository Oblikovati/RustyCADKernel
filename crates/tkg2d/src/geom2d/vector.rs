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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Vector.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_Vector`: vectors (unit or not) of the plane as transient geometry.
use super::cell::{GeometryCell, PROTECTED, ProtectedAccess};
use super::geometry::Geom2dGeometry;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Vector2;

/// A vector with the native protected `gpVec2d` storage; `vector.dot(&*other)`.
pub trait Geom2dVector: Geom2dGeometry {
    /// The protected `gpVec2d` cell; only Geom2d calls it.
    #[doc(hidden)]
    fn vector_cell(&self, access: ProtectedAccess) -> &GeometryCell<Vector2>;
    /// Length (`Magnitude`); `vector.magnitude()`.
    fn magnitude(&self) -> f64;
    /// Squared length (`SquareMagnitude`); `vector.square_magnitude()`.
    fn square_magnitude(&self) -> f64;
    /// Cross product `self ^ other` (`Crossed`); `vector.crossed(&*other)`.
    fn crossed(&self, other: &dyn Geom2dVector) -> f64;
    /// Reverse in place (`Reverse`); `vector.reverse()`.
    fn reverse(&self) {
        self.vector_cell(PROTECTED).write().reverse();
    }
    /// Reversed copy (`Reversed`); `let reversed = vector.reversed()?;`.
    fn reversed(&self) -> Result<Handle<dyn Geom2dVector>, Failure> {
        let copy: Handle<dyn Geom2dVector> = self.copy()?.cast::<dyn Geom2dVector>();
        copy.reverse();
        Ok(copy)
    }
    /// Signed angle in (-Pi, Pi] to `other`; fails for a null vector (`Angle`);
    /// `vector.angle(&*other)?`.
    fn angle(&self, other: &dyn Geom2dVector) -> Result<f64, Failure> {
        let own: Vector2 = self.vec2d();
        own.angle(other.vec2d())
    }
    /// Coordinates as `(x, y)` (`Coord`); `vector.coord()`.
    fn coord(&self) -> (f64, f64) {
        let own: Vector2 = self.vec2d();
        (own.x(), own.y())
    }
    /// X coordinate (`X`); `vector.x()`.
    fn x(&self) -> f64 {
        self.vec2d().x()
    }
    /// Y coordinate (`Y`); `vector.y()`.
    fn y(&self) -> f64 {
        self.vec2d().y()
    }
    /// Scalar product (`Dot`); `vector.dot(&*other)`.
    fn dot(&self, other: &dyn Geom2dVector) -> f64 {
        let own: Vector2 = self.vec2d();
        own.dot(other.vec2d())
    }
    /// A gp copy (`Vec2d`); `vector.vec2d()`.
    fn vec2d(&self) -> Vector2 {
        *self.vector_cell(PROTECTED).read()
    }
}
impl_transient_interface!(dyn Geom2dVector, "Geom2d_Vector", dyn Geom2dGeometry);
