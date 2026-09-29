// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_CurveOnSurface.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_CurveOnSurface`: the B-spline approximation of a curve on a surface — its
//! 3D curve, its 2D curve, or both on one parametrization — with an exact 3D curve
//! for an iso-line of a Geom surface.
mod evaluator;
mod iso_line;
mod perform;

use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::geom_abs::Shape;

/// The options of `Perform`: `(MaxSegments, MaxDegree, Continuity, Only3d, Only2d)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurveOnSurfaceOptions {
    /// `theMaxSegments`.
    pub max_segments: i32,
    /// `theMaxDegree`.
    pub max_degree: i32,
    /// `theContinuity`.
    pub continuity: Shape,
    /// `theOnly3d`: approximate the 3D curve only.
    pub only_3d: bool,
    /// `theOnly2d`: approximate the 2D curve only.
    pub only_2d: bool,
}

/// `Approx_CurveOnSurface`: the approximated curves and their errors.
///
/// ```ignore
/// let options = CurveOnSurfaceOptions { max_segments: 16, max_degree: 14,
///     continuity: Shape::C2, only_3d: false, only_2d: false };
/// let approx = CurveOnSurface::new(&c2d, &surface, 0.0, 1.0, 1e-6, options)?;
/// ```
#[derive(Clone)]
pub struct CurveOnSurface {
    c2d: Handle<dyn Adaptor2dCurve2d>,
    surf: Handle<dyn Adaptor3dSurface>,
    first: f64,
    last: f64,
    tol: f64,
    curve_2d: Handle<Geom2dBSplineCurve>,
    curve_3d: Handle<GeomBSplineCurve>,
    is_done: bool,
    has_result: bool,
    error_3d: f64,
    error_2d_u: f64,
    error_2d_v: f64,
}

impl CurveOnSurface {
    /// `Approx_CurveOnSurface(C2D, Surf, First, Last, Tol, S, MaxDegree, MaxSegments,
    /// only3d, only2d)`: initializes and performs;
    /// `CurveOnSurface::new(&c2d, &s, 0.0, 1.0, 1e-6, options)?`.
    pub fn new(
        c2d: &Handle<dyn Adaptor2dCurve2d>,
        surf: &Handle<dyn Adaptor3dSurface>,
        first: f64,
        last: f64,
        tol: f64,
        options: CurveOnSurfaceOptions,
    ) -> Result<Self, Failure> {
        let mut approx: Self = Self::without_perform(c2d, surf, first, last, tol);
        approx.perform(options)?;
        Ok(approx)
    }

    /// `Approx_CurveOnSurface(theC2D, theSurf, theFirst, theLast, theTol)`: stores the
    /// data only, before [`Self::perform`];
    /// `CurveOnSurface::without_perform(&c2d, &s, 0.0, 1.0, 1e-6)`.
    pub fn without_perform(
        c2d: &Handle<dyn Adaptor2dCurve2d>,
        surf: &Handle<dyn Adaptor3dSurface>,
        first: f64,
        last: f64,
        tol: f64,
    ) -> Self {
        Self {
            c2d: c2d.clone(),
            surf: surf.clone(),
            first,
            last,
            tol,
            curve_2d: Handle::default(),
            curve_3d: Handle::default(),
            is_done: false,
            has_result: false,
            error_3d: 0.0,
            error_2d_u: 0.0,
            error_2d_v: 0.0,
        }
    }

    /// `IsDone()`; `approx.is_done()`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }

    /// `HasResult()`; `approx.has_result()`.
    pub fn has_result(&self) -> bool {
        self.has_result
    }

    /// `Curve3d()`: null unless computed; `approx.curve_3d()`.
    pub fn curve_3d(&self) -> Handle<GeomBSplineCurve> {
        self.curve_3d.clone()
    }

    /// `MaxError3d()`; `approx.max_error_3d()`.
    pub fn max_error_3d(&self) -> f64 {
        self.error_3d
    }

    /// `Curve2d()`: null unless computed; `approx.curve_2d()`.
    pub fn curve_2d(&self) -> Handle<Geom2dBSplineCurve> {
        self.curve_2d.clone()
    }

    /// `MaxError2dU()`; `approx.max_error_2d_u()`.
    pub fn max_error_2d_u(&self) -> f64 {
        self.error_2d_u
    }

    /// `MaxError2dV()`; `approx.max_error_2d_v()`.
    pub fn max_error_2d_v(&self) -> f64 {
        self.error_2d_v
    }
}
