// Created on: 2014-11-14
// Copyright (c) 2014 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_Tool.hxx/.cxx (Parameter,
// Parameters) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib_Tool`: the parameters of a point on a curve or a surface.
//!
//! The functions compute the parameter(s) of a point lying on (or close to) a curve
//! or a surface; the point must be close enough to give an unambiguous result, and
//! `max_dist` is the trusted distance. They return None (the native false, with the
//! output parameters reset to 0) beyond `max_dist` or when the computation fails.
pub use super::tool_deviation::{DeviationResult, compute_deviation, compute_deviation_pso};
use crate::extrema::{ExtAlgo, ExtFlag, ExtPC, ExtPC2d, ExtPS};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::gp::{Point2, Point3};

// The parameter tolerance of the surface projection.
const PARTOLERANCE: f64 = 1.0e-9;

/// The static members of `GeomLib_Tool`; the two ComputeDeviation overloads are
/// [`compute_deviation`] and [`compute_deviation_pso`].
///
/// `let u = Tool::parameter(&curve, point, 1e-7)?;`
pub struct Tool;

impl Tool {
    /// `GeomLib_Tool::Parameter(Curve, Point, MaxDist, U)`: the parameter of the
    /// nearest Extrema_ExtPC extremum (the first of equal distances) when within
    /// `max_dist`; None for a null curve, a failed or empty projection, or a point too
    /// far; `Tool::parameter(&curve, point, 1e-7)?`.
    pub fn parameter(
        curve: &Handle<dyn GeomCurve>,
        point: Point3,
        max_dist: f64,
    ) -> Result<Option<f64>, Failure> {
        if curve.is_null() {
            return Ok(None);
        }
        let tol: f64 = max_dist * max_dist;
        let gac = GeomAdaptorCurve::with_curve(curve)?;
        let extrema = ExtPC::with_curve(point, &gac, 1.0e-10)?;
        if !extrema.is_done() {
            return Ok(None);
        }
        let Some(i_min) = nearest(extrema.nb_ext()?, |i| extrema.square_distance(i), tol)? else {
            return Ok(None);
        };
        Ok(Some(extrema.point(i_min)?.parameter()))
    }

    /// `GeomLib_Tool::Parameters(Surface, Point, MaxDist, U, V)`: the parameters of
    /// the nearest Extrema_ExtPS minimum (parameter tolerances 1e-9) within
    /// `max_dist`; `Tool::parameters(&surface, point, 1e-7)?`.
    pub fn parameters(
        surface: &Handle<dyn GeomSurface>,
        point: Point3,
        max_dist: f64,
    ) -> Result<Option<(f64, f64)>, Failure> {
        if surface.is_null() {
            return Ok(None);
        }
        let tol: f64 = max_dist * max_dist;
        let gas = GeomAdaptorSurface::with_surface(surface)?;
        let extrema = ExtPS::with_surface(
            point,
            &gas,
            PARTOLERANCE,
            PARTOLERANCE,
            ExtFlag::Min,
            ExtAlgo::Grad,
        )?;
        if !extrema.is_done() {
            return Ok(None);
        }
        let Some(i_min) = nearest(extrema.nb_ext()?, |i| extrema.square_distance(i), tol)? else {
            return Ok(None);
        };
        Ok(Some(extrema.point(i_min)?.parameter()))
    }

    /// `GeomLib_Tool::Parameter(Curve2d, Point2d, MaxDist, U)`: the 2D twin of
    /// [`Tool::parameter`] with Extrema_ExtPC2d; `Tool::parameter_2d(&curve, point, 1e-7)?`.
    pub fn parameter_2d(
        curve: &Handle<dyn Geom2dCurve>,
        point: Point2,
        max_dist: f64,
    ) -> Result<Option<f64>, Failure> {
        if curve.is_null() {
            return Ok(None);
        }
        let tol: f64 = max_dist * max_dist;
        let gac = Geom2dAdaptorCurve::with_curve(curve)?;
        let extrema = ExtPC2d::with_curve(point, &gac, 1.0e-10)?;
        if !extrema.is_done() {
            return Ok(None);
        }
        let Some(i_min) = nearest(extrema.nb_ext()?, |i| extrema.square_distance(i), tol)? else {
            return Ok(None);
        };
        Ok(Some(extrema.point(i_min)?.parameter()))
    }
}

// The index of the first smallest square distance of `n` extrema (starting from
// RealLast), when there is one within `tol`.
fn nearest(
    n: i32,
    square_distance: impl Fn(i32) -> Result<f64, Failure>,
    tol: f64,
) -> Result<Option<i32>, Failure> {
    if n <= 0 {
        return Ok(None);
    }
    let (mut i_min, mut dist2_min): (i32, f64) = (0, f64::MAX);
    for i in 1..=n {
        let d: f64 = square_distance(i)?;
        if d < dist2_min {
            i_min = i;
            dist2_min = d;
        }
    }
    if i_min != 0 && dist2_min <= tol {
        return Ok(Some(i_min));
    }
    Ok(None)
}
