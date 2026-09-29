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
// Rust translation of Geom2dInt_TheProjPCurOfGInter.hxx and
// Geom2dInt_TheProjPCurOfGInter_0.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dInt_TheProjPCurOfGInter`: the parameter of a point near an
//! `Adaptor2d_Curve2d`, by sampling then a local extremum search.
use super::geom2d_curve_tool::{Curve2d, Geom2dCurveTool};
use super::{TheCurveLocatorOfTheProjPCurOfGInter, TheLocateExtPCOfTheProjPCurOfGInter};
use crate::int_imp_par_gen::ProjectOnPCurveTool;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::extrema::POnCurv2d;
use rustycad_tkmath::gp::Point2;

/// `Geom2dInt_TheProjPCurOfGInter`: the nearest of `NbSamples(C)` samples between
/// the bounds, refined by Extrema's local search (kept when that fails or finds a
/// maximum). The curve's exceptions are returned as `Err`;
/// `TheProjPCurOfGInter::find_parameter(&adaptor, point, 1e-7)?`.
pub struct TheProjPCurOfGInter;

impl TheProjPCurOfGInter {
    /// `FindParameter(C, Pnt, LowParameter, HighParameter, Tol)` (the tolerance is
    /// unused). `TheProjPCurOfGInter::find_parameter_in(&adaptor, point, 0.0, 1.0, 1e-7)?`.
    pub fn find_parameter_in(
        c: &Curve2d,
        p: Point2,
        low_parameter: f64,
        high_parameter: f64,
        _tol: f64,
    ) -> Result<f64, Failure> {
        let nb_pts: i32 = Geom2dCurveTool::nb_samples(c)?;
        let the_eps_x: f64 = Geom2dCurveTool::eps_x(c);
        let pon_c: POnCurv2d = TheCurveLocatorOfTheProjPCurOfGInter::locate_range(
            p,
            c,
            nb_pts,
            low_parameter,
            high_parameter,
        )?;
        let defaultparam: f64 = pon_c.parameter();
        let loc = TheLocateExtPCOfTheProjPCurOfGInter::with_curve(p, c, defaultparam, the_eps_x)?;
        if !loc.is_done() || !loc.is_min()? {
            return Ok(defaultparam);
        }
        Ok(loc.point()?.parameter())
    }
    /// `FindParameter(C, Pnt, Tol)`: between the bounds of the curve.
    /// `TheProjPCurOfGInter::find_parameter(&adaptor, point, 1e-7)?`.
    pub fn find_parameter(c: &Curve2d, p: Point2, tol: f64) -> Result<f64, Failure> {
        let (first, last) = (
            Geom2dCurveTool::first_parameter(c)?,
            Geom2dCurveTool::last_parameter(c)?,
        );
        Self::find_parameter_in(c, p, first, last, tol)
    }
}

impl ProjectOnPCurveTool for TheProjPCurOfGInter {
    type Curve = Curve2d;
    fn find_parameter(curve: &Curve2d, pnt: Point2, tol: f64) -> Result<f64, Failure> {
        TheProjPCurOfGInter::find_parameter(curve, pnt, tol)
    }
    fn find_parameter_in(
        curve: &Curve2d,
        pnt: Point2,
        low_parameter: f64,
        high_parameter: f64,
        tol: f64,
    ) -> Result<f64, Failure> {
        TheProjPCurOfGInter::find_parameter_in(curve, pnt, low_parameter, high_parameter, tol)
    }
}
