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
// Rust translation of ModelingData/TKG3d/Geom/Geom_Vector.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Vector`: vectors (unit or not) of space as transient geometry.
use super::geometry::GeomGeometry;
use super::protected::{PROTECTED, ProtectedAccess};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::cell::GeometryCell;
use rustycad_tkmath::gp::Vector3;

/// A vector with the native protected `gpVec` storage; `vector.dot(&*other)`.
///
/// The members that take another vector read it before they lock this one, so a
/// vector can be combined with itself (`v.cross(&*v)`), as natively.
pub trait GeomVector: GeomGeometry {
    /// The protected `gpVec` cell; only Geom calls it.
    #[doc(hidden)]
    fn vector_cell(&self, access: ProtectedAccess) -> &GeometryCell<Vector3>;
    /// Length (`Magnitude`); `vector.magnitude()`.
    fn magnitude(&self) -> f64;
    /// Squared length (`SquareMagnitude`); `vector.square_magnitude()`.
    fn square_magnitude(&self) -> f64;
    /// Cross product in place (`Cross`); a direction fails when the product is null;
    /// `vector.cross(&*other)?`.
    fn cross(&self, other: &dyn GeomVector) -> Result<(), Failure>;
    /// Cross product as a new vector of the same class (`Crossed`); `vector.crossed(&*other)?`.
    fn crossed(&self, other: &dyn GeomVector) -> Result<Handle<dyn GeomVector>, Failure>;
    /// Triple product `self ^ (v1 ^ v2)` in place (`CrossCross`); `vector.cross_cross(&*a, &*b)?`.
    fn cross_cross(&self, v1: &dyn GeomVector, v2: &dyn GeomVector) -> Result<(), Failure>;
    /// Triple product as a new vector (`CrossCrossed`); `vector.cross_crossed(&*a, &*b)?`.
    fn cross_crossed(
        &self,
        v1: &dyn GeomVector,
        v2: &dyn GeomVector,
    ) -> Result<Handle<dyn GeomVector>, Failure>;
    /// Reverse in place (`Reverse`); `vector.reverse()`.
    fn reverse(&self) {
        self.vector_cell(PROTECTED).write().reverse();
    }
    /// Reversed copy (`Reversed`); `let reversed = vector.reversed()?;`.
    fn reversed(&self) -> Result<Handle<dyn GeomVector>, Failure> {
        let copy: Handle<dyn GeomVector> = self.copy()?.cast::<dyn GeomVector>();
        copy.reverse();
        Ok(copy)
    }
    /// Angle in [0, Pi] to `other`; fails for a null vector (`Angle`);
    /// `vector.angle(&*other)?`.
    fn angle(&self, other: &dyn GeomVector) -> Result<f64, Failure> {
        let own: Vector3 = self.vec();
        own.angle(other.vec())
    }
    /// Signed angle in (-Pi, Pi] to `other` around `reference` (`AngleWithRef`);
    /// `vector.angle_with_ref(&*other, &*reference)?`.
    fn angle_with_ref(
        &self,
        other: &dyn GeomVector,
        reference: &dyn GeomVector,
    ) -> Result<f64, Failure> {
        let own: Vector3 = self.vec();
        own.angle_with_ref(other.vec(), reference.vec())
    }
    /// Coordinates as `(x, y, z)` (`Coord`); `vector.coord()`.
    fn coord(&self) -> (f64, f64, f64) {
        let own: Vector3 = self.vec();
        (own.x(), own.y(), own.z())
    }
    /// X coordinate (`X`); `vector.x()`.
    fn x(&self) -> f64 {
        self.vec().x()
    }
    /// Y coordinate (`Y`); `vector.y()`.
    fn y(&self) -> f64 {
        self.vec().y()
    }
    /// Z coordinate (`Z`); `vector.z()`.
    fn z(&self) -> f64 {
        self.vec().z()
    }
    /// Scalar product (`Dot`); `vector.dot(&*other)`.
    fn dot(&self, other: &dyn GeomVector) -> f64 {
        let own: Vector3 = self.vec();
        own.dot(other.vec())
    }
    /// Triple scalar product `self . (v1 ^ v2)` (`DotCross`); `vector.dot_cross(&*a, &*b)`.
    fn dot_cross(&self, v1: &dyn GeomVector, v2: &dyn GeomVector) -> f64 {
        let own: Vector3 = self.vec();
        own.dot_cross(v1.vec(), v2.vec())
    }
    /// A gp copy (`Vec`); `vector.vec()`.
    fn vec(&self) -> Vector3 {
        *self.vector_cell(PROTECTED).read()
    }
}
impl_transient_interface!(dyn GeomVector, "Geom_Vector", dyn GeomGeometry);
