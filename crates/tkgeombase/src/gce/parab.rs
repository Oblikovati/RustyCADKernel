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
// Rust translation of gce_MakeParab.hxx/.cxx and gce_MakeParab2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gce_MakeParab and gce_MakeParab2d: construction of gp_Parab and gp_Parab2d.
use super::{ErrorType, gce_result};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    self, Axis1, Axis2, Axis2d, Axis22d, Direction2, Parabola2, Parabola3, Point2, Point3,
};

/// gce_MakeParab: builds a parabola from a frame and a focal length, or from its
/// directrix and focus.
///
/// `let parabola = MakeParab::new(Axis2::default(), 1.0)?.value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeParab {
    the_result: Parabola3,
    the_error: ErrorType,
}
gce_result!(MakeParab, Parabola3, "gce_MakeParab");

impl MakeParab {
    /// gce_MakeParab(A2, Focal): gce_NullFocusLength when `focal < 0`.
    /// `MakeParab::new(frame, 1.0)?`.
    pub fn new(frame: Axis2, focal: f64) -> Result<Self, Failure> {
        if focal < 0.0 {
            return Ok(Self {
                the_error: ErrorType::NullFocusLength,
                ..Self::default()
            });
        }
        Ok(Self {
            the_result: Parabola3::new(frame, focal)?,
            the_error: ErrorType::Done,
        })
    }

    /// gce_MakeParab(D, F): the parabola of directrix `directrix` and focus
    /// `focus`; gp raises Standard_ConstructionError when the focus lies on the
    /// directrix. `MakeParab::from_directrix(axis, focus)?`.
    pub fn from_directrix(directrix: Axis1, focus: Point3) -> Result<Self, Failure> {
        let parabola = Parabola3::from_directrix(directrix, focus)?;
        Ok(Self {
            the_result: parabola,
            the_error: ErrorType::Done,
        })
    }
}

/// gce_MakeParab2d: builds a 2D parabola from a frame, a mirror axis, a
/// directrix and focus, or its apex and center.
///
/// `let parabola = MakeParab2d::from_frame(Axis22d::default(), 1.0)?.value()?;`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeParab2d {
    the_result: Parabola2,
    the_error: ErrorType,
}
gce_result!(MakeParab2d, Parabola2, "gce_MakeParab2d");

impl MakeParab2d {
    fn failed(error: ErrorType) -> Self {
        Self {
            the_error: error,
            ..Self::default()
        }
    }
    fn built(parabola: Parabola2) -> Self {
        Self {
            the_result: parabola,
            the_error: ErrorType::Done,
        }
    }

    /// gce_MakeParab2d(A, Focal): the parabola of frame `frame`;
    /// gce_NullFocusLength when `focal < 0`. `MakeParab2d::from_frame(frame, 1.0)?`.
    pub fn from_frame(frame: Axis22d, focal: f64) -> Result<Self, Failure> {
        if focal < 0.0 {
            return Ok(Self::failed(ErrorType::NullFocusLength));
        }
        Ok(Self::built(Parabola2::new(frame, focal)?))
    }

    /// gce_MakeParab2d(MirrorAxis, Focal, Sense): the parabola of symmetry axis
    /// `mirror_axis`; gce_NullFocusLength when `focal < 0`.
    /// `MakeParab2d::new(axis, 1.0, true)?`.
    pub fn new(mirror_axis: Axis2d, focal: f64, sense: bool) -> Result<Self, Failure> {
        if focal < 0.0 {
            return Ok(Self::failed(ErrorType::NullFocusLength));
        }
        Ok(Self::built(Parabola2::from_mirror_axis(
            mirror_axis,
            focal,
            sense,
        )?))
    }

    /// gce_MakeParab2d(D, F, Sense): the parabola of directrix `directrix` and
    /// focus `focus`; always done. `MakeParab2d::from_directrix(axis, f, true)?`.
    pub fn from_directrix(directrix: Axis2d, focus: Point2, sense: bool) -> Result<Self, Failure> {
        Ok(Self::built(Parabola2::from_directrix(
            directrix, focus, sense,
        )?))
    }

    /// gce_MakeParab2d(S, Center, Sense): the parabola of apex `center`, symmetry
    /// axis toward `s` and focal length `|center s|`; gce_NullAxis when the points
    /// are closer than gp::Resolution(). `MakeParab2d::from_points(s, c, true)?`.
    pub fn from_points(s: Point2, center: Point2, sense: bool) -> Result<Self, Failure> {
        if s.distance(center) < gp::resolution() {
            return Ok(Self::failed(ErrorType::NullAxis));
        }
        let x_axis: Direction2 = Direction2::from_xy(s.xy() - center.xy())?;
        let axis = Axis2d::new(center, x_axis);
        Ok(Self::built(Parabola2::from_mirror_axis(
            axis,
            s.distance(center),
            sense,
        )?))
    }
}
