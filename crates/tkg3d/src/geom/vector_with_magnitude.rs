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
// Rust translation of ModelingData/TKG3d/Geom/Geom_VectorWithMagnitude.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_VectorWithMagnitude`: a vector of any length as transient geometry.
use super::geometry::GeomGeometry;
use super::protected::ProtectedAccess;
use super::vector::GeomVector;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::cell::GeometryCell;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

/// A vector with magnitude; `Handle::new(GeomVectorWithMagnitude::new(3.0, 4.0, 0.0))`.
pub struct GeomVectorWithMagnitude {
    parent: StandardTransient,
    state: GeometryCell<Vector3>,
}
impl_transient!(
    GeomVectorWithMagnitude,
    "Geom_VectorWithMagnitude",
    dyn GeomVector,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomVector]
);
impl Clone for GeomVectorWithMagnitude {
    fn clone(&self) -> Self {
        Self::from_vec(self.vec())
    }
}
impl GeomVectorWithMagnitude {
    /// Vector from a gp vector (`Geom_VectorWithMagnitude(gp_Vec)`);
    /// `GeomVectorWithMagnitude::from_vec(Vector3::new(1.0, 0.0, 0.0))`.
    pub fn from_vec(vector: Vector3) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(vector),
        }
    }
    /// Vector from coordinates (`Geom_VectorWithMagnitude(X, Y, Z)`);
    /// `GeomVectorWithMagnitude::new(3.0, 4.0, 0.0)`.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self::from_vec(Vector3::new(x, y, z))
    }
    /// Vector from `first` to `second` (`Geom_VectorWithMagnitude(P1, P2)`);
    /// `GeomVectorWithMagnitude::between(a, b)`.
    pub fn between(first: Point3, second: Point3) -> Self {
        Self::from_vec(Vector3::between(first, second))
    }
    /// Set the coordinates (`SetCoord`); `vector.set_coord(1.0, 2.0, 3.0)`.
    pub fn set_coord(&self, x: f64, y: f64, z: f64) {
        *self.state.write() = Vector3::new(x, y, z);
    }
    /// Replace the vector (`SetVec`); `vector.set_vec(v)`.
    pub fn set_vec(&self, vector: Vector3) {
        *self.state.write() = vector;
    }
    /// Set X (`SetX`); `vector.set_x(1.0)`.
    pub fn set_x(&self, x: f64) {
        self.state.write().set_x(x);
    }
    /// Set Y (`SetY`); `vector.set_y(1.0)`.
    pub fn set_y(&self, y: f64) {
        self.state.write().set_y(y);
    }
    /// Set Z (`SetZ`); `vector.set_z(1.0)`.
    pub fn set_z(&self, z: f64) {
        self.state.write().set_z(z);
    }
    /// Add `other` in place (`Add`); `vector.add(&*other)`.
    pub fn add(&self, other: &dyn GeomVector) {
        let addend: Vector3 = other.vec();
        *self.state.write() += addend;
    }
    /// Sum as a new vector (`Added`); `vector.added(&*other)`.
    pub fn added(&self, other: &dyn GeomVector) -> Handle<GeomVectorWithMagnitude> {
        let mut v1: Vector3 = self.vec();
        v1 += other.vec();
        Handle::new(Self::from_vec(v1))
    }
    /// Divide in place (`Divide`); `vector.divide(2.0)`.
    pub fn divide(&self, scalar: f64) {
        *self.state.write() /= scalar;
    }
    /// Quotient as a new vector (`Divided`); `vector.divided(2.0)`.
    pub fn divided(&self, scalar: f64) -> Handle<GeomVectorWithMagnitude> {
        let mut value: Vector3 = self.vec();
        value /= scalar;
        Handle::new(Self::from_vec(value))
    }
    /// Product as a new vector (`Multiplied`); `vector.multiplied(2.0)`.
    pub fn multiplied(&self, scalar: f64) -> Handle<GeomVectorWithMagnitude> {
        let mut value: Vector3 = self.vec();
        value *= scalar;
        Handle::new(Self::from_vec(value))
    }
    /// Multiply in place (`Multiply`); `vector.multiply(2.0)`.
    pub fn multiply(&self, scalar: f64) {
        *self.state.write() *= scalar;
    }
    /// Normalize in place; Standard_ConstructionError for a null vector (`Normalize`);
    /// `vector.normalize()?`.
    pub fn normalize(&self) -> Result<(), Failure> {
        self.state.write().normalize()
    }
    /// Unit copy; Standard_ConstructionError for a null vector (`Normalized`);
    /// `vector.normalized()?`.
    pub fn normalized(&self) -> Result<Handle<GeomVectorWithMagnitude>, Failure> {
        let mut value: Vector3 = self.vec();
        value.normalize()?;
        Ok(Handle::new(Self::from_vec(value)))
    }
    /// Subtract `other` in place (`Subtract`); `vector.subtract(&*other)`.
    pub fn subtract(&self, other: &dyn GeomVector) {
        let subtrahend: Vector3 = other.vec();
        *self.state.write() -= subtrahend;
    }
    /// Difference as a new vector (`Subtracted`); `vector.subtracted(&*other)`.
    pub fn subtracted(&self, other: &dyn GeomVector) -> Handle<GeomVectorWithMagnitude> {
        let mut value: Vector3 = self.vec();
        value -= other.vec();
        Handle::new(Self::from_vec(value))
    }
}
impl GeomGeometry for GeomVectorWithMagnitude {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        self.state.write().transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
}
impl GeomVector for GeomVectorWithMagnitude {
    fn vector_cell(&self, _access: ProtectedAccess) -> &GeometryCell<Vector3> {
        &self.state
    }
    fn magnitude(&self) -> f64 {
        self.vec().magnitude()
    }
    fn square_magnitude(&self) -> f64 {
        self.vec().square_magnitude()
    }
    fn cross(&self, other: &dyn GeomVector) -> Result<(), Failure> {
        let right: Vector3 = other.vec();
        self.state.write().cross(right);
        Ok(())
    }
    fn crossed(&self, other: &dyn GeomVector) -> Result<Handle<dyn GeomVector>, Failure> {
        let mut value: Vector3 = self.vec();
        value.cross(other.vec());
        Ok(Handle::new(Self::from_vec(value)).cast())
    }
    fn cross_cross(&self, v1: &dyn GeomVector, v2: &dyn GeomVector) -> Result<(), Failure> {
        let (first, second) = (v1.vec(), v2.vec());
        self.state.write().cross_cross(first, second);
        Ok(())
    }
    fn cross_crossed(
        &self,
        v1: &dyn GeomVector,
        v2: &dyn GeomVector,
    ) -> Result<Handle<dyn GeomVector>, Failure> {
        let mut value: Vector3 = self.vec();
        value.cross_cross(v1.vec(), v2.vec());
        Ok(Handle::new(Self::from_vec(value)).cast())
    }
}
