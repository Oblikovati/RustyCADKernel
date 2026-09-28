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
// Rust translation of GC_MakeEllipse and GC_MakeHyperbola (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GC_MakeEllipse and GC_MakeHyperbola: Geom_Ellipse and Geom_Hyperbola.
use crate::gce::{self, ErrorType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomEllipse, GeomHyperbola};
use rustycad_tkmath::gp::{Axis2, Ellipse3, Hyperbola3, Point3};

/// GC_MakeEllipse: builds a Geom_Ellipse from a gp ellipse, a frame and radii, or
/// the ends of its axes and its center.
///
/// `let ellipse = MakeEllipse::new(Axis2::default(), 2.0, 1.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeEllipse {
    the_result: Handle<GeomEllipse>,
    the_error: ErrorType,
}
gc_result!(MakeEllipse, GeomEllipse, "GC_MakeEllipse");
gc_from_gce!(MakeEllipse, gce::MakeElips, GeomEllipse::from_elips);

impl MakeEllipse {
    /// GC_MakeEllipse(E): the Geom ellipse of `ellipse`. `MakeEllipse::from_elips(e)`.
    pub fn from_elips(ellipse: Ellipse3) -> Self {
        Self::built(GeomEllipse::from_elips(ellipse))
    }

    /// GC_MakeEllipse(A2, MajorRadius, MinorRadius): gce_NegativeRadius when
    /// `minor < 0`, then gce_InvertAxis (not gce_InvertRadius, as gce_MakeElips
    /// reports) when `major < minor`. `MakeEllipse::new(frame, 2.0, 1.0)?`.
    pub fn new(frame: Axis2, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        if minor_radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        if major_radius < minor_radius {
            return Ok(Self::failed(ErrorType::InvertAxis));
        }
        let ellipse = Ellipse3::new(frame, major_radius, minor_radius)?;
        Ok(Self::built(GeomEllipse::from_elips(ellipse)))
    }

    /// GC_MakeEllipse(S1, S2, Center): gce_MakeElips(S1, S2, Center).
    /// `MakeEllipse::from_points(s1, s2, center)?`.
    pub fn from_points(s1: Point3, s2: Point3, center: Point3) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeElips::from_points(s1, s2, center)?)
    }
}

/// GC_MakeHyperbola: builds a Geom_Hyperbola from a gp hyperbola, a frame and
/// radii, or the ends of its axes and its center.
///
/// `let hyperbola = MakeHyperbola::new(Axis2::default(), 2.0, 1.0)?.value()?;`
#[derive(Clone, Default)]
pub struct MakeHyperbola {
    the_result: Handle<GeomHyperbola>,
    the_error: ErrorType,
}
gc_result!(MakeHyperbola, GeomHyperbola, "GC_MakeHyperbola");
gc_from_gce!(MakeHyperbola, gce::MakeHypr, GeomHyperbola::from_hypr);

impl MakeHyperbola {
    /// GC_MakeHyperbola(H): the Geom hyperbola of `hyperbola`.
    /// `MakeHyperbola::from_hypr(h)`.
    pub fn from_hypr(hyperbola: Hyperbola3) -> Self {
        Self::built(GeomHyperbola::from_hypr(hyperbola))
    }

    /// GC_MakeHyperbola(A2, MajorRadius, MinorRadius): gce_NegativeRadius when a
    /// radius is negative. `MakeHyperbola::new(frame, 2.0, 1.0)?`.
    pub fn new(frame: Axis2, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        if major_radius < 0.0 || minor_radius < 0.0 {
            return Ok(Self::failed(ErrorType::NegativeRadius));
        }
        let hyperbola = Hyperbola3::new(frame, major_radius, minor_radius)?;
        Ok(Self::built(GeomHyperbola::from_hypr(hyperbola)))
    }

    /// GC_MakeHyperbola(S1, S2, Center): gce_MakeHypr(S1, S2, Center).
    /// `MakeHyperbola::from_points(s1, s2, center)?`.
    pub fn from_points(s1: Point3, s2: Point3, center: Point3) -> Result<Self, Failure> {
        Self::from_gce(gce::MakeHypr::from_points(s1, s2, center)?)
    }
}
