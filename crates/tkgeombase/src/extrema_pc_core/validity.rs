// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of the constants and validity helpers of
// ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d.hxx and ExtremaPC/ExtremaPC.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The tolerances, sampling constants and input checks that ExtremaPC2d.hxx and
//! ExtremaPC.hxx both declare with the same values (the neighbor step ratio, which
//! differs, stays in each package).
use rustycad_tkernel::precision;
use rustycad_tkmath::math_utils::Domain1D;

/// Default tolerance for root finding and distance comparison (`THE_DEFAULT_TOLERANCE`).
pub const THE_DEFAULT_TOLERANCE: f64 = precision::confusion();

/// Tolerance for parameter domain comparison (`THE_PARAM_TOLERANCE`).
pub const THE_PARAM_TOLERANCE: f64 = precision::p_confusion();

/// Minimum number of samples for Bezier curves (`THE_BEZIER_MIN_SAMPLES`).
pub const THE_BEZIER_MIN_SAMPLES: usize = 24;

/// Degree multiplier of the Bezier samples: max(min, multiplier * (degree + 1))
/// (`THE_BEZIER_DEGREE_MULTIPLIER`).
pub const THE_BEZIER_DEGREE_MULTIPLIER: usize = 3;

/// Default number of samples for general (other) curves (`THE_OTHER_CURVE_NB_SAMPLES`).
pub const THE_OTHER_CURVE_NB_SAMPLES: usize = 64;

/// Fallback number of samples for BSpline curves when the curve is null
/// (`THE_BSPLINE_FALLBACK_SAMPLES`).
pub const THE_BSPLINE_FALLBACK_SAMPLES: usize = 32;

/// BSpline samples per knot span: multiplier * (degree + 1) (`THE_BSPLINE_SPAN_MULTIPLIER`).
pub const THE_BSPLINE_SPAN_MULTIPLIER: usize = 2;

/// True for a finite, positive tolerance (`IsValidTolerance`); `is_valid_tolerance(1.0e-9)`.
pub fn is_valid_tolerance(tolerance: f64) -> bool {
    tolerance.is_finite() && tolerance > 0.0
}

/// True for a parameter usable as a finite endpoint: finite and not beyond
/// Precision::Infinite() (`IsFiniteParameter`); `is_finite_parameter(1.0)`.
pub fn is_finite_parameter(parameter: f64) -> bool {
    parameter.is_finite() && !precision::is_infinite(parameter)
}

/// True when a valid domain spans a whole number (at least one) of periods, within
/// `tolerance` (`IsClosedPeriodicDomain`);
/// `is_closed_periodic_domain(&Domain1D::new(0.0, 2.0 * PI), 2.0 * PI, 1e-10)`.
pub fn is_closed_periodic_domain(domain: &Domain1D, period: f64, tolerance: f64) -> bool {
    if !domain.is_valid()
        || !period.is_finite()
        || period <= 0.0
        || !tolerance.is_finite()
        || tolerance < 0.0
    {
        return false;
    }
    let nb_periods: f64 = (domain.length() / period).round();
    nb_periods >= 1.0 && (domain.length() - nb_periods * period).abs() <= tolerance
}
