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
// Rust translation of GC_MakeLine and GC_MakeSegment (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GC_MakeLine and GC_MakeSegment: Geom_Line and trimmed Geom lines.
use crate::gce::{self, ErrorType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis1, Direction3, Line3, Point3};

/// GC_MakeLine: builds a Geom_Line from a point and a direction, an axis, a gp
/// line, two points, or a parallel line.
///
/// `let line = MakeLine::from_axis(Axis1::default()).value()?;`
#[derive(Clone, Default)]
pub struct MakeLine {
    the_result: Handle<GeomLine>,
    the_error: ErrorType,
}
gc_result!(MakeLine, GeomLine, "GC_MakeLine");
gc_from_gce!(MakeLine, gce::MakeLin, GeomLine::from_lin);

impl MakeLine {
    /// GC_MakeLine(P, V): the line through `point` of direction `direction`.
    /// `MakeLine::new(p, d)`.
    pub fn new(point: Point3, direction: Direction3) -> Self {
        Self::built(GeomLine::from_point(point, direction))
    }

    /// GC_MakeLine(A1): the line of axis `axis`. `MakeLine::from_axis(axis)`.
    pub fn from_axis(axis: Axis1) -> Self {
        Self::built(GeomLine::new(axis))
    }

    /// GC_MakeLine(L): the Geom line of `line`. `MakeLine::from_lin(line)`.
    pub fn from_lin(line: Line3) -> Self {
        Self::built(GeomLine::from_lin(line))
    }

    /// GC_MakeLine(P1, P2): gce_MakeLin(P1, P2). `MakeLine::from_points(a, b)?`.
    pub fn from_points(p1: Point3, p2: Point3) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeLin::from_points(p1, p2)?)
    }

    /// GC_MakeLine(Lin, Point): gce_MakeLin(Lin, Point).
    /// `MakeLine::from_line_point(line, p)?`.
    pub fn from_line_point(line: Line3, point: Point3) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeLin::from_line_point(line, point))
    }
}

/// GC_MakeSegment: builds a trimmed Geom_Line between two points, or on a line
/// between points or parameters.
///
/// `let segment = MakeSegment::from_parameters(line, 0.0, 1.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeSegment {
    the_result: Handle<GeomTrimmedCurve>,
    the_error: ErrorType,
}
gc_result!(MakeSegment, GeomTrimmedCurve, "GC_MakeSegment");

impl MakeSegment {
    /// `Geom_TrimmedCurve(L, U1, U2, true)`: the segment of `line` between two
    /// parameters.
    fn trimmed(line: Handle<GeomLine>, u1: f64, u2: f64) -> Result<Self, Failure> {
        let basis: Handle<dyn GeomCurve> = line.cast();
        Ok(Self::built(GeomTrimmedCurve::new(
            &basis, u1, u2, true, true,
        )?))
    }

    /// GC_MakeSegment(P1, P2): the line of GC_MakeLine(P1, P2) (its conversion
    /// raises StdFail_NotDone for confused points) trimmed to `[0, |P1 P2|]`.
    /// `MakeSegment::from_points(a, b)?`.
    pub fn from_points(p1: Point3, p2: Point3) -> Result<Self, Failure> {
        let dist: f64 = p1.distance(&p2);
        let line: Handle<GeomLine> = Handle::try_from(&MakeLine::from_points(p1, p2)?)?;
        Self::trimmed(line, 0.0, dist)
    }

    /// GC_MakeSegment(Line, Point, U): the segment from the parameter of `point`
    /// to `u`. `MakeSegment::from_point_parameter(line, p, 1.0)?`.
    pub fn from_point_parameter(line: Line3, point: Point3, u: f64) -> Result<Self, Failure> {
        let u_first: f64 = elclib::parameter(&line, point)?;
        Self::trimmed(Handle::new(GeomLine::from_lin(line)), u_first, u)
    }

    /// GC_MakeSegment(Line, P1, P2): the segment between the parameters of `p1`
    /// and `p2`. `MakeSegment::from_line_points(line, a, b)?`.
    pub fn from_line_points(line: Line3, p1: Point3, p2: Point3) -> Result<Self, Failure> {
        let u_first: f64 = elclib::parameter(&line, p1)?;
        let u_last: f64 = elclib::parameter(&line, p2)?;
        Self::trimmed(Handle::new(GeomLine::from_lin(line)), u_first, u_last)
    }

    /// GC_MakeSegment(Line, U1, U2): the segment between two parameters.
    /// `MakeSegment::from_parameters(line, 0.0, 1.0)?`.
    pub fn from_parameters(line: Line3, u1: f64, u2: f64) -> Result<Self, Failure> {
        Self::trimmed(Handle::new(GeomLine::from_lin(line)), u1, u2)
    }
}
