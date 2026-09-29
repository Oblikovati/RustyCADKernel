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

//! The `ThePoint`/`TheVector` parameters of the Extrema class templates: the members
//! of `gp_Pnt`/`gp_Vec` and `gp_Pnt2d`/`gp_Vec2d` that the generic code calls.
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// A gp point of the Extrema templates (`gp_Pnt` or `gp_Pnt2d`).
pub trait GpPoint: Copy + Default + Debug + PartialEq + 'static {
    /// The matching gp vector.
    type Vector: GpVector;
    /// 3 or 2.
    const DIMENSION: usize;
    /// `Coord(index + 1)` (`X()`, `Y()`, `Z()`); `point.coordinate(0)`.
    fn coordinate(&self, index: usize) -> f64;
    /// `SquareDistance(theOther)`; `point.square_distance_to(&other)`.
    fn square_distance_to(&self, other: &Self) -> f64;
    /// `Distance(theOther)`; `point.distance_to(&other)`.
    fn distance_to(&self, other: &Self) -> f64;
    /// `gp_Vec(*this, theTarget)`; `point.vector_to(&target)`.
    fn vector_to(&self, target: &Self) -> Self::Vector;
}

/// A gp vector of the Extrema templates (`gp_Vec` or `gp_Vec2d`), with the native
/// operators.
pub trait GpVector:
    Copy
    + Default
    + Debug
    + 'static
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<f64, Output = Self>
    + Div<f64, Output = Self>
    + Neg<Output = Self>
{
    /// `Dot(theOther)`; `vector.dot_with(&other)`.
    fn dot_with(&self, other: &Self) -> f64;
    /// `Coord(index + 1)` (`X()`, `Y()`, `Z()`); `vector.coordinate(0)`.
    fn coordinate(&self, index: usize) -> f64;
    /// `Magnitude()`; `vector.norm()`.
    fn norm(&self) -> f64;
}

impl GpPoint for Point3 {
    type Vector = Vector3;
    const DIMENSION: usize = 3;
    fn coordinate(&self, index: usize) -> f64 {
        [self.x(), self.y(), self.z()][index]
    }
    fn square_distance_to(&self, other: &Self) -> f64 {
        self.square_distance(other)
    }
    fn distance_to(&self, other: &Self) -> f64 {
        self.distance(other)
    }
    fn vector_to(&self, target: &Self) -> Vector3 {
        Vector3::between(*self, *target)
    }
}

impl GpPoint for Point2 {
    type Vector = Vector2;
    const DIMENSION: usize = 2;
    fn coordinate(&self, index: usize) -> f64 {
        [self.x(), self.y()][index]
    }
    fn square_distance_to(&self, other: &Self) -> f64 {
        self.square_distance(*other)
    }
    fn distance_to(&self, other: &Self) -> f64 {
        self.distance(*other)
    }
    fn vector_to(&self, target: &Self) -> Vector2 {
        Vector2::between(*self, *target)
    }
}

impl GpVector for Vector3 {
    fn dot_with(&self, other: &Self) -> f64 {
        self.dot(*other)
    }
    fn coordinate(&self, index: usize) -> f64 {
        [self.x(), self.y(), self.z()][index]
    }
    fn norm(&self) -> f64 {
        self.magnitude()
    }
}

impl GpVector for Vector2 {
    fn dot_with(&self, other: &Self) -> f64 {
        self.dot(*other)
    }
    fn coordinate(&self, index: usize) -> f64 {
        [self.x(), self.y()][index]
    }
    fn norm(&self) -> f64 {
        self.magnitude()
    }
}
