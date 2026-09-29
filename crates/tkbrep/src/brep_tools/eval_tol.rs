// Created on: 1993-01-21
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of BRepTools::EvalAndUpdateTol
// (ModelingData/TKBRep/BRepTools/BRepTools.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools::EvalAndUpdateTol`: the deviation of a p-curve from a 3D curve.
use crate::brep::errors::null_object;
use crate::brep::{BRepBuilder, BRepTool};
use crate::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::geom_lib::CheckCurveOnSurface;

/// The three curves the tolerance is measured on.
struct CurvesOnSurface<'a> {
    c3d: &'a dyn GeomCurve,
    c2d: &'a dyn Geom2dCurve,
    surface: &'a dyn GeomSurface,
}

/// The object of a handle; Standard_NullObject for a null one (a crash natively).
fn live<'a, T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &'a Handle<T>,
    what: &str,
) -> Result<&'a T, Failure> {
    handle
        .get()
        .ok_or_else(|| null_object(&format!("BRepTools::EvalAndUpdateTol: null {what}")))
}

/// `BRepTools::EvalAndUpdateTol(theE, C3d, C2d, S, f, l)`: the maximal distance between
/// the 3D curve and the p-curve on the surface over `[f, l]` (narrowed to the curves'
/// ranges when they are not periodic), measured by GeomLib_CheckCurveOnSurface or, when
/// that check fails with status 3 (or 2 on a periodic curve), by 23 samples times 1.2.
/// The edge tolerance is raised to it when larger; returns the resulting tolerance.
/// A null curve or surface raises Standard_NullObject (a crash natively);
/// `eval_and_update_tol(&edge, &c3d, &c2d, &surface, 0.0, 1.0)?`.
pub fn eval_and_update_tol(
    edge: &Edge,
    c3d: &Handle<dyn GeomCurve>,
    c2d: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    first: f64,
    last: f64,
) -> Result<f64, Failure> {
    let curves = CurvesOnSurface {
        c3d: live(c3d, "3D curve")?,
        c2d: live(c2d, "p-curve")?,
        surface: live(surface, "surface")?,
    };
    let (first, last) = narrowed_range(&curves, first, last);
    let new_tolerance = measured_deviation(c3d, c2d, surface, &curves, (first, last))?;
    let mut tolerance = BRepTool::edge_tolerance(edge)?;
    if new_tolerance > tolerance {
        tolerance = new_tolerance;
        BRepBuilder::default().update_edge_tolerance(edge, tolerance)?;
    }
    Ok(tolerance)
}

/// The range narrowed to the bounds of each non-periodic curve (it avoids the error
/// status 2 of the too strict limit check of GeomLib_CheckCurveOnSurface).
fn narrowed_range(curves: &CurvesOnSurface<'_>, mut first: f64, mut last: f64) -> (f64, f64) {
    if !curves.c3d.is_periodic() {
        first = first.max(curves.c3d.first_parameter());
        last = last.min(curves.c3d.last_parameter());
    }
    if !curves.c2d.is_periodic() {
        first = first.max(curves.c2d.first_parameter());
        last = last.min(curves.c2d.last_parameter());
    }
    (first, last)
}

/// The deviation by GeomLib_CheckCurveOnSurface, or by samples when the check fails
/// with status 3 (or 2 with a periodic curve); 0 for any other failure.
fn measured_deviation(
    c3d: &Handle<dyn GeomCurve>,
    c2d: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    curves: &CurvesOnSurface<'_>,
    range: (f64, f64),
) -> Result<f64, Failure> {
    let curve: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(c3d, range.0, range.1)?).cast();
    let pcurve: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(c2d, range.0, range.1)?).cast();
    let adapted: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(surface)?).cast();
    let on_surface = Handle::new(Adaptor3dCurveOnSurface::with_curve(&pcurve, &adapted)?);
    let mut check = CheckCurveOnSurface::with_curve(&curve, precision::p_confusion());
    check.perform(&on_surface);
    if check.is_done() {
        return Ok(check.max_distance());
    }
    let status = check.error_status();
    let periodic = curves.c3d.is_periodic() || curves.c2d.is_periodic();
    if status == 3 || (status == 2 && periodic) {
        return sampled_deviation(curves, range);
    }
    Ok(0.0)
}

/// 1.2 times the largest distance between the curves at 22 steps over the range
/// (steps of at least `Precision::Confusion()`), the last parameter included.
fn sampled_deviation(
    curves: &CurvesOnSurface<'_>,
    (first, last): (f64, f64),
) -> Result<f64, Failure> {
    const NB_INTERVALS: i32 = 22;
    let step = ((last - first) / f64::from(NB_INTERVALS)).max(precision::confusion());
    let mut max_square = 0.0;
    let mut count = 0;
    let mut t = first;
    while t <= last {
        count += 1;
        max_square = f64::max(max_square, square_gap(curves, t)?);
        t += step;
    }
    if count < NB_INTERVALS + 1 {
        let square = square_gap(curves, last)?;
        if square > max_square {
            max_square = square;
        }
    }
    Ok(1.2 * max_square.sqrt())
}

/// The squared distance between the 3D curve and the p-curve on the surface at `t`.
fn square_gap(curves: &CurvesOnSurface<'_>, t: f64) -> Result<f64, Failure> {
    let uv = curves.c2d.d0(t)?;
    let on_curve = curves.c3d.d0(t)?;
    let on_surface = curves.surface.d0(uv.x(), uv.y())?;
    Ok(on_surface.square_distance(&on_curve))
}
