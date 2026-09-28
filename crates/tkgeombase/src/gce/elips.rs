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
// Rust translation of gce_MakeElips.hxx/.cxx and gce_MakeElips2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gce_MakeElips and gce_MakeElips2d: construction of gp_Elips and gp_Elips2d.
use super::{ErrorType, gce_result};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    self, Axis2, Axis2d, Axis22d, Direction2, Direction3, Ellipse2, Ellipse3, Line2, Line3, Point2,
    Point3, Xyz,
};

/// gce_MakeElips: builds an ellipse from a frame and radii, or from the ends of
/// its two axes and its center.
///
/// `let ellipse = MakeElips::new(Axis2::default(), 2.0, 1.0)?.value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeElips {
    the_result: Ellipse3,
    the_error: ErrorType,
}
gce_result!(MakeElips, Ellipse3, "gce_MakeElips");

impl MakeElips {
    fn failed(error: ErrorType) -> Self {
        Self {
            the_error: error,
            ..Self::default()
        }
    }

    /// gce_MakeElips(A2, MajorRadius, MinorRadius): gce_InvertRadius when
    /// `major < minor`, then gce_NegativeRadius when `minor < 0`.
    /// `MakeElips::new(frame, 2.0, 1.0)?`.
    pub fn new(frame: Axis2, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        if major_radius < minor_radius {
            return Ok(Self::failed(ErrorType::InvertRadius));
        }
        if minor_radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        let ellipse = Ellipse3::new(frame, major_radius, minor_radius)?;
        Ok(Self {
            the_result: ellipse,
            the_error: ErrorType::Done,
        })
    }

    /// gce_MakeElips(S1, S2, Center): the ellipse of center `center` whose major
    /// axis ends at `s1` and whose minor radius is the distance of `s2` from the
    /// major axis; gce_NullAxis when `s1` is the center, gce_InvertAxis when the
    /// minor radius exceeds the major one or is null.
    /// `MakeElips::from_points(s1, s2, center)?`.
    pub fn from_points(s1: Point3, s2: Point3, center: Point3) -> Result<Self, Failure> {
        let d1: f64 = s1.distance(&center);
        if d1 < gp::resolution() {
            return Ok(Self::failed(ErrorType::NullAxis));
        }
        let x_axis: Direction3 = Direction3::from_xyz(s1.xyz() - center.xyz())?;
        let d2: f64 = Line3::new(center, x_axis).distance(s2);
        if d1 < d2 || d2 < gp::resolution() {
            return Ok(Self::failed(ErrorType::InvertAxis));
        }
        let cross: Xyz = x_axis.xyz().crossed(s2.xyz() - center.xyz());
        if cross.modulus() <= gp::resolution() {
            return Ok(Self::failed(ErrorType::InvertAxis));
        }
        let frame = Axis2::with_x_direction(center, Direction3::from_xyz(cross)?, x_axis)?;
        Ok(Self {
            the_result: Ellipse3::new(frame, d1, d2)?,
            the_error: ErrorType::Done,
        })
    }
}

/// gce_MakeElips2d: builds a 2D ellipse from a major axis, a frame, or the ends
/// of its two axes and its center.
///
/// `let ellipse = MakeElips2d::from_frame(Axis22d::default(), 2.0, 1.0)?.value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeElips2d {
    the_result: Ellipse2,
    the_error: ErrorType,
}
gce_result!(MakeElips2d, Ellipse2, "gce_MakeElips2d");

impl MakeElips2d {
    /// The radius checks of the axis and frame constructors: gce_NegativeRadius
    /// when `major < 0`, then gce_InvertRadius when `major < minor`.
    fn checked(
        major: f64,
        minor: f64,
        build: impl FnOnce() -> Result<Ellipse2, Failure>,
    ) -> Result<Self, Failure> {
        let error: ErrorType = if major < 0.0 {
            ErrorType::NegativeRadius
        } else if major < minor {
            ErrorType::InvertRadius
        } else {
            return Ok(Self {
                the_result: build()?,
                the_error: ErrorType::Done,
            });
        };
        Ok(Self {
            the_error: error,
            ..Self::default()
        })
    }

    /// gce_MakeElips2d(MajorAxis, MajorRadius, MinorRadius, Sense): the ellipse of
    /// major axis `major_axis`. `MakeElips2d::new(axis, 2.0, 1.0, true)?`.
    pub fn new(
        major_axis: Axis2d,
        major_radius: f64,
        minor_radius: f64,
        sense: bool,
    ) -> Result<Self, Failure> {
        Self::checked(major_radius, minor_radius, || {
            Ellipse2::from_x_axis(major_axis, major_radius, minor_radius, sense)
        })
    }

    /// gce_MakeElips2d(A, MajorRadius, MinorRadius): the ellipse of frame `frame`.
    /// `MakeElips2d::from_frame(frame, 2.0, 1.0)?`.
    pub fn from_frame(
        frame: Axis22d,
        major_radius: f64,
        minor_radius: f64,
    ) -> Result<Self, Failure> {
        Self::checked(major_radius, minor_radius, || {
            Ellipse2::new(frame, major_radius, minor_radius)
        })
    }

    /// gce_MakeElips2d(S1, S2, Center): the ellipse of center `center` whose major
    /// axis ends at `s1`, its Y direction toward `s2`; gce_InvertAxis when the
    /// minor radius exceeds the major one, gce_NullAxis when it is null. The
    /// directions are built before any check, so `s1` or `s2` at the center
    /// raises Standard_ConstructionError. `MakeElips2d::from_points(s1, s2, c)?`.
    pub fn from_points(s1: Point2, s2: Point2, center: Point2) -> Result<Self, Failure> {
        let d1: f64 = s1.distance(center);
        let x_axis: Direction2 = Direction2::from_xy(s1.xy() - center.xy())?;
        let y_axis: Direction2 = Direction2::from_xy(s2.xy() - center.xy())?;
        let d2: f64 = Line2::new(center, x_axis).distance(s2);
        let error: ErrorType = if d1 < d2 {
            ErrorType::InvertAxis
        } else if d2 < gp::resolution() {
            ErrorType::NullAxis
        } else {
            let frame = Axis22d::with_directions(center, x_axis, y_axis)?;
            return Ok(Self {
                the_result: Ellipse2::new(frame, d1, d2)?,
                the_error: ErrorType::Done,
            });
        };
        Ok(Self {
            the_error: error,
            ..Self::default()
        })
    }
}
