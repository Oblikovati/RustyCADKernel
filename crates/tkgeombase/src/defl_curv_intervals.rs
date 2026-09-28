// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of DeflCurvIntervals of ModelingData/TKGeomBase/Extrema/
// Extrema_CurveTool.cxx and Extrema_Curve2dTool.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_CurveTool::DeflCurvIntervals` and `Extrema_Curve2dTool::DeflCurvIntervals`:
//! the parameters of a GCPnts_TangentialDeflection sampling sized from the curve
//! length, which the ExtremaPC and ExtremaPC2d curve-aware partitions refine with.
//!
//! The classic Extrema package is not ported yet; this crate-private module ports
//! exactly these two functions for ExtremaPC, and the Extrema port will replace it.
use crate::gcpnts::{TCurveTypes, TangentialCriteria, TangentialDeflection};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use std::f64::consts::PI;

/// `Extrema_Curve2dTool::DeflCurvIntervals(C)`; the values of the 1-based native
/// array, in order.
pub(crate) fn defl_curv_intervals_2d(curve: &dyn Adaptor2dCurve2d) -> Result<Vec<f64>, Failure> {
    let (tf, tl) = (curve.first_parameter()?, curve.last_parameter()?);
    let mut length: f64 = 0.0;
    let origin = curve.value(tf)?;
    for i in 2..=23 {
        let t: f64 = sample_parameter(tf, tl, i);
        length += origin.distance(curve.value(t)?);
    }
    deflection_parameters(curve, (tf, tl), length)
}

/// `Extrema_CurveTool::DeflCurvIntervals(C)`; the values of the 1-based native array,
/// in order.
pub(crate) fn defl_curv_intervals_3d(curve: &dyn Adaptor3dCurve) -> Result<Vec<f64>, Failure> {
    let (tf, tl) = (curve.first_parameter()?, curve.last_parameter()?);
    let mut length: f64 = 0.0;
    let origin = curve.value(tf)?;
    for i in 2..=23 {
        let t: f64 = sample_parameter(tf, tl, i);
        length += origin.distance(&curve.value(t)?);
    }
    deflection_parameters(curve, (tf, tl), length)
}

// `(tf * (nbpnts - i) + (i - 1) * tl) / (nbpnts - 1)` with nbpnts = 23. The sum
// accumulates distances from the first point (not a polyline), as natively.
fn sample_parameter(tf: f64, tl: f64, i: i32) -> f64 {
    (tf * f64::from(23 - i) + f64::from(i - 1) * tl) / f64::from(23 - 1)
}

// The deflection-sized sampling of `[tf, tl]`, or its two bounds for a short, slow,
// long-range or strongly deflected curve.
fn deflection_parameters<C: TCurveTypes + ?Sized>(
    curve: &C,
    (tf, tl): (f64, f64),
    length: f64,
) -> Result<Vec<f64>, Failure> {
    let (epsd, maxdefl, mindefl) = (1.0e-3, 1.0e3, 1.0e-3);
    let dl_dt: f64 = length / (tl - tf);
    // (tl - tf) > 10000: to avoid problem with GCPnts
    if length <= precision::confusion() || dl_dt < epsd || (tl - tf) > 10000.0 {
        return Ok(vec![tf, tl]);
    }
    let defl: f64 = std_max(0.01 * length / (2.0 * PI), mindefl);
    if defl > maxdefl {
        return Ok(vec![tf, tl]);
    }
    let criteria = TangentialCriteria {
        angular_deflection: PI / 6.0,
        curvature_deflection: defl,
        minimum_of_points: 2,
        u_tol: std_max(0.00001 * (tl - tf), precision::p_confusion()),
        min_len: std_max(0.00001 * length, precision::confusion()),
    };
    let generator = TangentialDeflection::with_curve(curve, criteria)?;
    (1..=generator.nb_points())
        .map(|i| generator.parameter(i))
        .collect()
}
