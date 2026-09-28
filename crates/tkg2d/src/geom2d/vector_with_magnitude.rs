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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_VectorWithMagnitude.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_VectorWithMagnitude`: a vector of any length as transient geometry.
use super::cell::{GeometryCell, ProtectedAccess};
use super::geometry::Geom2dGeometry;
use super::vector::Geom2dVector;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};

/// A vector with magnitude; `Handle::new(Geom2dVectorWithMagnitude::new(3.0, 4.0))`.
pub struct Geom2dVectorWithMagnitude {
    parent: StandardTransient,
    state: GeometryCell<Vector2>,
}
impl_transient!(
    Geom2dVectorWithMagnitude,
    "Geom2d_VectorWithMagnitude",
    dyn Geom2dVector,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dVector]
);
impl Clone for Geom2dVectorWithMagnitude {
    fn clone(&self) -> Self {
        Self::from_vec2d(self.vec2d())
    }
}
impl Geom2dVectorWithMagnitude {
    /// Vector from a gp vector (`Geom2d_VectorWithMagnitude(gp_Vec2d)`);
    /// `Geom2dVectorWithMagnitude::from_vec2d(Vector2::new(1.0, 0.0))`.
    pub fn from_vec2d(vector: Vector2) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(vector),
        }
    }
    /// Vector from coordinates (`Geom2d_VectorWithMagnitude(X, Y)`);
    /// `Geom2dVectorWithMagnitude::new(3.0, 4.0)`.
    pub fn new(x: f64, y: f64) -> Self {
        Self::from_vec2d(Vector2::new(x, y))
    }
    /// Vector from `first` to `second` (`Geom2d_VectorWithMagnitude(P1, P2)`);
    /// `Geom2dVectorWithMagnitude::between(a, b)`.
    pub fn between(first: Point2, second: Point2) -> Self {
        Self::from_vec2d(Vector2::between(first, second))
    }
    /// Set both coordinates (`SetCoord`); `vector.set_coord(1.0, 2.0)`.
    pub fn set_coord(&self, x: f64, y: f64) {
        *self.state.write() = Vector2::new(x, y);
    }
    /// Replace the vector (`SetVec2d`); `vector.set_vec2d(v)`.
    pub fn set_vec2d(&self, vector: Vector2) {
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
    /// Add `other` in place (`Add`, `+=`); `vector.add(&*other)`.
    pub fn add(&self, other: &dyn Geom2dVector) {
        let addend: Vector2 = other.vec2d();
        *self.state.write() += addend;
    }
    /// Sum as a new vector (`Added`, `+`); `vector.added(&*other)`.
    pub fn added(&self, other: &dyn Geom2dVector) -> Handle<Geom2dVectorWithMagnitude> {
        let mut temp: Vector2 = other.vec2d();
        temp += self.vec2d();
        Handle::new(Self::from_vec2d(temp))
    }
    /// Divide in place (`Divide`, `/=`); `vector.divide(2.0)`.
    pub fn divide(&self, scalar: f64) {
        *self.state.write() /= scalar;
    }
    /// Quotient as a new vector (`Divided`, `/`); `vector.divided(2.0)`.
    pub fn divided(&self, scalar: f64) -> Handle<Geom2dVectorWithMagnitude> {
        let mut value: Vector2 = self.vec2d();
        value /= scalar;
        Handle::new(Self::from_vec2d(value))
    }
    /// Product as a new vector (`Multiplied`, `*`); `vector.multiplied(2.0)`.
    pub fn multiplied(&self, scalar: f64) -> Handle<Geom2dVectorWithMagnitude> {
        let mut value: Vector2 = self.vec2d();
        value *= scalar;
        Handle::new(Self::from_vec2d(value))
    }
    /// Multiply in place (`Multiply`, `*=`); `vector.multiply(2.0)`.
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
    pub fn normalized(&self) -> Result<Handle<Geom2dVectorWithMagnitude>, Failure> {
        let value: Vector2 = self.vec2d().normalized()?;
        Ok(Handle::new(Self::from_vec2d(value)))
    }
    /// Subtract `other` in place (`Subtract`, `-=`); `vector.subtract(&*other)`.
    pub fn subtract(&self, other: &dyn Geom2dVector) {
        let subtrahend: Vector2 = other.vec2d();
        *self.state.write() -= subtrahend;
    }
    /// Difference as a new vector (`Subtracted`, `-`); `vector.subtracted(&*other)`.
    pub fn subtracted(&self, other: &dyn Geom2dVector) -> Handle<Geom2dVectorWithMagnitude> {
        let mut value: Vector2 = self.vec2d();
        value -= other.vec2d();
        Handle::new(Self::from_vec2d(value))
    }
}
impl Geom2dGeometry for Geom2dVectorWithMagnitude {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        self.state.write().transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
}
impl Geom2dVector for Geom2dVectorWithMagnitude {
    fn vector_cell(&self, _access: ProtectedAccess) -> &GeometryCell<Vector2> {
        &self.state
    }
    fn magnitude(&self) -> f64 {
        self.vec2d().magnitude()
    }
    fn square_magnitude(&self) -> f64 {
        self.vec2d().square_magnitude()
    }
    fn crossed(&self, other: &dyn Geom2dVector) -> f64 {
        let own: Vector2 = self.vec2d();
        own.crossed(other.vec2d())
    }
}
