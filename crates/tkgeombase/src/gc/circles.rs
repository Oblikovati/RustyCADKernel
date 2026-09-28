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
// Rust translation of GC_MakeCircle and GC_MakeCircle2d (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GC_MakeCircle and GC_MakeCircle2d: Geom_Circle and Geom2d_Circle.
use crate::gce::{self, ErrorType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCircle;
use rustycad_tkg3d::geom::GeomCircle;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis2d, Axis22d, Circle2, Circle3, Direction3, Point2, Point3,
};

/// GC_MakeCircle: builds a Geom_Circle from a gp circle, a frame, a parallel
/// circle, three points, a center and a normal, or an axis.
///
/// `let circle = MakeCircle::new(Axis2::default(), 1.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeCircle {
    the_result: Handle<GeomCircle>,
    the_error: ErrorType,
}
gc_result!(MakeCircle, GeomCircle, "GC_MakeCircle");
gc_from_gce!(MakeCircle, gce::MakeCirc, GeomCircle::from_circ);

impl MakeCircle {
    /// GC_MakeCircle(C): the Geom circle of `circle`. `MakeCircle::from_circ(c)`.
    pub fn from_circ(circle: Circle3) -> Self {
        Self::built(GeomCircle::from_circ(circle))
    }

    /// GC_MakeCircle(A2, Radius): gce_NegativeRadius when `radius < 0`.
    /// `MakeCircle::new(frame, 1.0)?`.
    pub fn new(frame: Axis2, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(GeomCircle::from_circ(Circle3::new(
            frame, radius,
        )?)))
    }

    /// GC_MakeCircle(Circ, Point): the conversion of gce_MakeCirc(Circ, Point),
    /// always done. `MakeCircle::from_circ_point(c, p)?`.
    pub fn from_circ_point(circle: Circle3, point: Point3) -> Result<Self, Failure> {
        let circ: Circle3 = gce::MakeCirc::from_circ_point(circle, point)?.operator()?;
        Ok(Self::built(GeomCircle::from_circ(circ)))
    }

    /// GC_MakeCircle(Circ, Dist): gce_MakeCirc(Circ, Dist).
    /// `MakeCircle::from_circ_distance(c, 1.0)?`.
    pub fn from_circ_distance(circle: Circle3, dist: f64) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCirc::from_circ_distance(circle, dist)?)
    }

    /// GC_MakeCircle(P1, P2, P3): gce_MakeCirc(P1, P2, P3).
    /// `MakeCircle::through_points(a, b, c)?`.
    pub fn through_points(p1: Point3, p2: Point3, p3: Point3) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCirc::through_points(p1, p2, p3)?)
    }

    /// GC_MakeCircle(Point, Norm, Radius): gce_MakeCirc(Point, Norm, Radius).
    /// `MakeCircle::from_center_normal(p, n, 1.0)?`.
    pub fn from_center_normal(
        center: Point3,
        normal: Direction3,
        radius: f64,
    ) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCirc::from_center_normal(center, normal, radius)?)
    }

    /// GC_MakeCircle(Point, PtAxis, Radius): gce_MakeCirc(Point, PtAxis, Radius).
    /// `MakeCircle::from_center_axis_point(c, p, 1.0)?`.
    pub fn from_center_axis_point(
        center: Point3,
        axis_point: Point3,
        radius: f64,
    ) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCirc::from_center_axis_point(
            center, axis_point, radius,
        )?)
    }

    /// GC_MakeCircle(Axis, Radius): gce_MakeCirc(Axis, Radius).
    /// `MakeCircle::from_axis(axis, 1.0)?`.
    pub fn from_axis(axis: Axis1, radius: f64) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCirc::from_axis(axis, radius)?)
    }
}

/// GC_MakeCircle2d: builds a Geom2d_Circle from a gp circle, an axis, a frame, a
/// parallel circle, three points or a center.
///
/// `let circle = MakeCircle2d::from_frame(Axis22d::default(), 1.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeCircle2d {
    the_result: Handle<Geom2dCircle>,
    the_error: ErrorType,
}
gc_result!(MakeCircle2d, Geom2dCircle, "GC_MakeCircle2d");
gc_from_gce!(MakeCircle2d, gce::MakeCirc2d, Geom2dCircle::from_circ2d);

impl MakeCircle2d {
    /// GC_MakeCircle2d(C): the Geom2d circle of `circle`.
    /// `MakeCircle2d::from_circ(c)`.
    pub fn from_circ(circle: Circle2) -> Self {
        Self::built(Geom2dCircle::from_circ2d(circle))
    }

    /// GC_MakeCircle2d(A, Radius, Sense): gce_NegativeRadius when `radius < 0`.
    /// `MakeCircle2d::new(axis, 1.0, true)?`.
    pub fn new(axis: Axis2d, radius: f64, sense: bool) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(Geom2dCircle::from_axis(axis, radius, sense)?))
    }

    /// GC_MakeCircle2d(A, Radius) with a gp_Ax22d frame: gce_NegativeRadius when
    /// `radius < 0`. `MakeCircle2d::from_frame(frame, 1.0)?`.
    pub fn from_frame(frame: Axis22d, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        Ok(Self::built(Geom2dCircle::new(frame, radius)?))
    }

    /// GC_MakeCircle2d(Circ, Point): gce_MakeCirc2d(Circ, Point).
    /// `MakeCircle2d::from_circ_point(c, p)?`.
    pub fn from_circ_point(circle: Circle2, point: Point2) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCirc2d::from_circ_point(circle, point)?)
    }

    /// GC_MakeCircle2d(Circ, Dist): gce_MakeCirc2d(Circ, Dist).
    /// `MakeCircle2d::from_circ_distance(c, 1.0)?`.
    pub fn from_circ_distance(circle: Circle2, dist: f64) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCirc2d::from_circ_distance(circle, dist)?)
    }

    /// GC_MakeCircle2d(P1, P2, P3): gce_MakeCirc2d(P1, P2, P3).
    /// `MakeCircle2d::through_points(a, b, c)?`.
    pub fn through_points(p1: Point2, p2: Point2, p3: Point2) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCirc2d::through_points(p1, p2, p3)?)
    }

    /// GC_MakeCircle2d(Point, Radius, Sense): gce_MakeCirc2d(Point, Radius, Sense).
    /// `MakeCircle2d::from_center(p, 1.0, true)?`.
    pub fn from_center(center: Point2, radius: f64, sense: bool) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCirc2d::from_center(center, radius, sense)?)
    }

    /// GC_MakeCircle2d(Center, Point, Sense): gce_MakeCirc2d(Center, Point, Sense).
    /// `MakeCircle2d::from_center_point(c, p, true)?`.
    pub fn from_center_point(center: Point2, point: Point2, sense: bool) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeCirc2d::from_center_point(center, point, sense)?)
    }
}
