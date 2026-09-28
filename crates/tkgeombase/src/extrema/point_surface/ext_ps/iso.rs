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
// Rust translation of IsoIsDeg of Extrema_ExtPS.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The degenerated boundary isos of Extrema_ExtPS.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;

/// `IsoIsDeg(S, Param, IT, TolMin, TolMax)`: the largest first derivative along the
/// U iso (`is_u`) or V iso at `param`, sampled at ten steps over a finite range, is
/// within `[tol_min, tol_max]` (true as well over an infinite range; false for a
/// step below PConfusion); `iso_is_deg(&surface, 0.0, true, 0., 1e-9)?`.
pub(super) fn iso_is_deg(
    surface: &dyn Adaptor3dSurface,
    param: f64,
    is_u: bool,
    tol_min: f64,
    tol_max: f64,
) -> Result<bool, Failure> {
    let (u1, u2) = (surface.first_u_parameter()?, surface.last_u_parameter()?);
    let (v1, v2) = (surface.first_v_parameter()?, surface.last_v_parameter()?);
    // GeomAbs_IsoV samples along U, GeomAbs_IsoU along V.
    let (first, last) = if is_u { (v1, v2) } else { (u1, u2) };
    if precision::is_infinite(first) || precision::is_infinite(last) {
        return Ok(true);
    }
    let step: f64 = (last - first) / 10.;
    if step < precision::p_confusion() {
        return Ok(false);
    }
    let mut d1_norm_max: f64 = 0.;
    let mut t: f64 = first;
    while t <= last {
        let d1 = if is_u {
            surface.d1(param, t)?
        } else {
            surface.d1(t, param)?
        };
        let derivative = if is_u { d1.d1v } else { d1.d1u };
        d1_norm_max = std_max(d1_norm_max, derivative.magnitude());
        t += step;
    }
    Ok(!(d1_norm_max > tol_max || d1_norm_max < tol_min))
}
