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
// Rust translation of GC_MakeEllipse2d, GC_MakeHyperbola2d and GC_MakeParabola2d (.hxx/.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GC_MakeEllipse2d, GC_MakeHyperbola2d and GC_MakeParabola2d: the Geom2d conics.
use crate::gce::{self, ErrorType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dEllipse, Geom2dHyperbola, Geom2dParabola};
use rustycad_tkmath::gp::{Axis2d, Axis22d, Ellipse2, Hyperbola2, Parabola2, Point2};

/// GC_MakeEllipse2d: builds a Geom2d_Ellipse from a gp ellipse, a frame, a major
/// axis, or the ends of its axes and its center.
///
/// `let ellipse = MakeEllipse2d::from_frame(Axis22d::default(), 2.0, 1.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeEllipse2d {
    the_result: Handle<Geom2dEllipse>,
    the_error: ErrorType,
}
gc_result!(MakeEllipse2d, Geom2dEllipse, "GC_MakeEllipse2d");
gc_from_gce!(MakeEllipse2d, gce::MakeElips2d, Geom2dEllipse::from_elips2d);

impl MakeEllipse2d {
    /// GC_MakeEllipse2d(E): the Geom2d ellipse of `ellipse`.
    /// `MakeEllipse2d::from_elips(e)`.
    pub fn from_elips(ellipse: Ellipse2) -> Self {
        Self::built(Geom2dEllipse::from_elips2d(ellipse))
    }

    /// GC_MakeEllipse2d(Axis, MajorRadius, MinorRadius): gce_MakeElips2d with a
    /// frame. `MakeEllipse2d::from_frame(frame, 2.0, 1.0)?`.
    pub fn from_frame(
        frame: Axis22d,
        major_radius: f64,
        minor_radius: f64,
    ) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeElips2d::from_frame(
            frame,
            major_radius,
            minor_radius,
        )?)
    }

    /// GC_MakeEllipse2d(MajorAxis, MajorRadius, MinorRadius, Sense): gce_MakeElips2d
    /// with a major axis. `MakeEllipse2d::new(axis, 2.0, 1.0, true)?`.
    pub fn new(
        major_axis: Axis2d,
        major_radius: f64,
        minor_radius: f64,
        sense: bool,
    ) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeElips2d::new(
            major_axis,
            major_radius,
            minor_radius,
            sense,
        )?)
    }

    /// GC_MakeEllipse2d(S1, S2, Center): gce_MakeElips2d(S1, S2, Center).
    /// `MakeEllipse2d::from_points(s1, s2, center)?`.
    pub fn from_points(s1: Point2, s2: Point2, center: Point2) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeElips2d::from_points(s1, s2, center)?)
    }
}

/// GC_MakeHyperbola2d: builds a Geom2d_Hyperbola from a gp hyperbola, a major
/// axis, a frame, or the ends of its axes and its center.
///
/// `let hyperbola = MakeHyperbola2d::from_frame(Axis22d::default(), 2.0, 1.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeHyperbola2d {
    the_result: Handle<Geom2dHyperbola>,
    the_error: ErrorType,
}
gc_result!(MakeHyperbola2d, Geom2dHyperbola, "GC_MakeHyperbola2d");
gc_from_gce!(
    MakeHyperbola2d,
    gce::MakeHypr2d,
    Geom2dHyperbola::from_hypr2d
);

impl MakeHyperbola2d {
    /// GC_MakeHyperbola2d(H): the Geom2d hyperbola of `hyperbola`.
    /// `MakeHyperbola2d::from_hypr(h)`.
    pub fn from_hypr(hyperbola: Hyperbola2) -> Self {
        Self::built(Geom2dHyperbola::from_hypr2d(hyperbola))
    }

    /// GC_MakeHyperbola2d(MajorAxis, MajorRadius, MinorRadius, Sense):
    /// gce_MakeHypr2d with a major axis. `MakeHyperbola2d::new(axis, 2.0, 1.0, true)?`.
    pub fn new(
        major_axis: Axis2d,
        major_radius: f64,
        minor_radius: f64,
        sense: bool,
    ) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeHypr2d::new(
            major_axis,
            major_radius,
            minor_radius,
            sense,
        )?)
    }

    /// GC_MakeHyperbola2d(Axis, MajorRadius, MinorRadius): gce_MakeHypr2d with a
    /// frame. `MakeHyperbola2d::from_frame(frame, 2.0, 1.0)?`.
    pub fn from_frame(
        frame: Axis22d,
        major_radius: f64,
        minor_radius: f64,
    ) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeHypr2d::from_frame(
            frame,
            major_radius,
            minor_radius,
        )?)
    }

    /// GC_MakeHyperbola2d(S1, S2, Center): gce_MakeHypr2d(S1, S2, Center).
    /// `MakeHyperbola2d::from_points(s1, s2, center)?`.
    pub fn from_points(s1: Point2, s2: Point2, center: Point2) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeHypr2d::from_points(s1, s2, center)?)
    }
}

/// GC_MakeParabola2d: builds a Geom2d_Parabola from a gp parabola, a mirror axis,
/// a frame, a directrix and focus, or its apex and a point of its axis.
///
/// `let parabola = MakeParabola2d::from_frame(Axis22d::default(), 1.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeParabola2d {
    the_result: Handle<Geom2dParabola>,
    the_error: ErrorType,
}
gc_result!(MakeParabola2d, Geom2dParabola, "GC_MakeParabola2d");
gc_from_gce!(
    MakeParabola2d,
    gce::MakeParab2d,
    Geom2dParabola::from_parab2d
);

impl MakeParabola2d {
    /// GC_MakeParabola2d(Prb): the Geom2d parabola of `parabola`.
    /// `MakeParabola2d::from_parab(p)`.
    pub fn from_parab(parabola: Parabola2) -> Self {
        Self::built(Geom2dParabola::from_parab2d(parabola))
    }

    /// GC_MakeParabola2d(MirrorAxis, Focal, Sense): gce_NullFocusLength when
    /// `focal < 0`. `MakeParabola2d::new(axis, 1.0, true)?`.
    pub fn new(mirror_axis: Axis2d, focal: f64, sense: bool) -> Result<Self, Failure> {
        if focal < 0.0 {
            return Ok(Self::failed(ErrorType::NullFocusLength));
        }
        Ok(Self::built(Geom2dParabola::from_axis(
            mirror_axis,
            focal,
            sense,
        )?))
    }

    /// GC_MakeParabola2d(Axis, Focal): gce_NullFocusLength when `focal < 0`.
    /// `MakeParabola2d::from_frame(frame, 1.0)?`.
    pub fn from_frame(frame: Axis22d, focal: f64) -> Result<Self, Failure> {
        if focal < 0.0 {
            return Ok(Self::failed(ErrorType::NullFocusLength));
        }
        Ok(Self::built(Geom2dParabola::new(frame, focal)?))
    }

    /// GC_MakeParabola2d(D, F, Sense): the parabola `gp_Parab2d(D, F, Sense)`,
    /// always done. `MakeParabola2d::from_directrix(axis, focus, true)?`.
    pub fn from_directrix(directrix: Axis2d, focus: Point2, sense: bool) -> Result<Self, Failure> {
        let parabola = Parabola2::from_directrix(directrix, focus, sense)?;
        Ok(Self::built(Geom2dParabola::from_parab2d(parabola)))
    }

    /// GC_MakeParabola2d(S1, O): gce_MakeParab2d(S1, O) with the default sense.
    /// `MakeParabola2d::from_points(s1, apex)?`.
    pub fn from_points(s1: Point2, apex: Point2) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeParab2d::from_points(s1, apex, true)?)
    }
}
