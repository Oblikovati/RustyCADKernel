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
// Rust translation of Extrema_POnCurv.hxx, Extrema_POnCurv2d.hxx, Extrema_POnSurf.hxx
// and Extrema_POnSurfParams.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The points of the extrema: a parameter with its point on a curve, two parameters
//! with their point on a surface, and the grid data of a surface sample.
use super::ElementType;
use rustycad_tkernel::precision;
use rustycad_tkmath::gp::{Point2, Point3};
use std::ops::{Deref, DerefMut};

/// A point on a curve with its parameter; `Extrema_POnCurv` and `Extrema_POnCurv2d`
/// are the same class over `gp_Pnt` and `gp_Pnt2d`;
/// `POnCurv::new(0.5, point).parameter() == 0.5`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct POnCurve<P> {
    u: f64,
    point: P,
}

/// `Extrema_POnCurv`: a point on a 3D curve.
pub type POnCurv = POnCurve<Point3>;
/// `Extrema_POnCurv2d`: a point on a 2D curve.
pub type POnCurv2d = POnCurve<Point2>;

impl<P: Copy> POnCurve<P> {
    /// `Extrema_POnCurv(theU, theP)`; `POnCurv::new(0.5, point)`.
    pub fn new(u: f64, point: P) -> Self {
        Self { u, point }
    }
    /// `SetValues(theU, theP)`; `pon.set_values(0.5, point)`.
    pub fn set_values(&mut self, u: f64, point: P) {
        self.u = u;
        self.point = point;
    }
    /// `Value()`: the point; `pon.value()`.
    pub fn value(&self) -> &P {
        &self.point
    }
    /// `Parameter()`: the parameter on the curve; `pon.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.u
    }
}

/// `Extrema_POnSurf`: a point on a surface with its parameters. The default point
/// (`Extrema_POnSurf()`) has every coordinate and parameter at Precision::Infinite();
/// `POnSurf::new(0.0, 1.0, point).parameter() == (0.0, 1.0)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct POnSurf {
    u: f64,
    v: f64,
    point: Point3,
}

impl Default for POnSurf {
    fn default() -> Self {
        let infinite: f64 = precision::infinite();
        Self::new(
            infinite,
            infinite,
            Point3::new(infinite, infinite, infinite),
        )
    }
}

impl POnSurf {
    /// `Extrema_POnSurf(theU, theV, theP)`; `POnSurf::new(0.0, 1.0, point)`.
    pub fn new(u: f64, v: f64, point: Point3) -> Self {
        Self { u, v, point }
    }
    /// `Value()`: the 3D point; `pon.value()`.
    pub fn value(&self) -> &Point3 {
        &self.point
    }
    /// `SetParameters(theU, theV, thePnt)`; `pon.set_parameters(0.0, 1.0, point)`.
    pub fn set_parameters(&mut self, u: f64, v: f64, point: Point3) {
        self.u = u;
        self.v = v;
        self.point = point;
    }
    /// `Parameter(theU, theV)`: the parameters, returned as `(u, v)`; `pon.parameter()`.
    pub fn parameter(&self) -> (f64, f64) {
        (self.u, self.v)
    }
}

/// `Extrema_POnSurfParams`: a surface point with the square distance to the point to
/// project and the grid element that holds it; the `Extrema_POnSurf` base is reached
/// through `Deref`; `POnSurfParams::new(0.0, 1.0, point).get_sqr_distance() == 0.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct POnSurfParams {
    base: POnSurf,
    sqr_distance: f64,
    element_type: ElementType,
    index_u: i32,
    index_v: i32,
}

impl Default for POnSurfParams {
    fn default() -> Self {
        Self::from_base(POnSurf::default())
    }
}

impl POnSurfParams {
    /// `Extrema_POnSurfParams(theU, theV, thePnt)`; `POnSurfParams::new(0.0, 1.0, point)`.
    pub fn new(u: f64, v: f64, point: Point3) -> Self {
        Self::from_base(POnSurf::new(u, v, point))
    }
    fn from_base(base: POnSurf) -> Self {
        Self {
            base,
            sqr_distance: 0.0,
            element_type: ElementType::Node,
            index_u: 0,
            index_v: 0,
        }
    }
    /// `SetSqrDistance(theSqrDistance)`; `params.set_sqr_distance(4.0)`.
    pub fn set_sqr_distance(&mut self, sqr_distance: f64) {
        self.sqr_distance = sqr_distance;
    }
    /// `GetSqrDistance()`; `params.get_sqr_distance()`.
    pub fn get_sqr_distance(&self) -> f64 {
        self.sqr_distance
    }
    /// `SetElementType(theElementType)`; `params.set_element_type(ElementType::Face)`.
    pub fn set_element_type(&mut self, element_type: ElementType) {
        self.element_type = element_type;
    }
    /// `GetElementType()`; `params.get_element_type()`.
    pub fn get_element_type(&self) -> ElementType {
        self.element_type
    }
    /// `SetIndices(theIndexU, theIndexV)`; `params.set_indices(2, 3)`.
    pub fn set_indices(&mut self, index_u: i32, index_v: i32) {
        self.index_u = index_u;
        self.index_v = index_v;
    }
    /// `GetIndices(theIndexU, theIndexV)`, returned as `(u, v)`; `params.get_indices()`.
    pub fn get_indices(&self) -> (i32, i32) {
        (self.index_u, self.index_v)
    }
    /// The `Extrema_POnSurf` part (the native slicing copy); `params.as_pon_surf()`.
    pub fn as_pon_surf(&self) -> POnSurf {
        self.base
    }
}

impl Deref for POnSurfParams {
    type Target = POnSurf;
    fn deref(&self) -> &POnSurf {
        &self.base
    }
}

impl DerefMut for POnSurfParams {
    fn deref_mut(&mut self) -> &mut POnSurf {
        &mut self.base
    }
}
