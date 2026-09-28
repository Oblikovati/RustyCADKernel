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

//! Overflow-safe mapping of CSLib_Class2d coordinates and tolerances to [0,1].
use crate::cslib::precision::is_infinite;

/// Ranges at or below this width are not normalized.
const MIN_NORMALIZATION_RANGE: f64 = 1.0e-10;

/// True when direct arithmetic cannot reach OCCT's geometric infinity range.
pub(super) fn is_safe_for_direct_arithmetic(value: f64) -> bool {
    !value.is_nan() && !is_infinite(value)
}

/// True for representable values, including OCCT's finite infinity sentinels.
pub(super) fn is_representable_value(value: f64) -> bool {
    (-f64::MAX..=f64::MAX).contains(&value)
}

// Written as the native `range > MIN` test, so a NaN range is not normalized.
fn is_normalizable(range: f64) -> bool {
    range > MIN_NORMALIZATION_RANGE
}

// std::max(a, b), which returns `a` unless `a < b` (unlike f64::max for NaN).
fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// Maps `u` from `[u_min, u_max]` to `[0, 1]`, rescaling first when the direct
/// difference would overflow; ranges of 1e-10 or less return `u` unchanged.
/// `transform_to_normalized(5.0, 0.0, 10.0) == 0.5`.
pub(super) fn transform_to_normalized(u: f64, u_min: f64, u_max: f64) -> f64 {
    let range: f64 = u_max - u_min;
    if !is_normalizable(range) {
        return u;
    }
    let difference: f64 = u - u_min;
    if is_safe_for_direct_arithmetic(difference) && is_safe_for_direct_arithmetic(range) {
        return difference / range;
    }
    let scale: f64 = std_max(u_min.abs(), u_max.abs());
    let scaled_min: f64 = u_min / scale;
    (u / scale - scaled_min) / (u_max / scale - scaled_min)
}

/// Converts an external tolerance to a finite, non-negative value.
/// `sanitize_tolerance(f64::NAN) == 0.0`.
pub(super) fn sanitize_tolerance(tolerance: f64) -> f64 {
    if tolerance.is_nan() || tolerance <= 0.0 {
        return 0.0;
    }
    if is_representable_value(tolerance) {
        tolerance
    } else {
        f64::MAX
    }
}

/// Normalizes a distance without overflowing when the finite bounds span more
/// than the representable range. `normalize_tolerance(0.5, 0.0, 100.0) == 0.005`.
pub(super) fn normalize_tolerance(tolerance: f64, min: f64, max: f64) -> f64 {
    let tolerance: f64 = sanitize_tolerance(tolerance);
    let range: f64 = max - min;
    if !is_normalizable(range) {
        return tolerance;
    }
    if is_safe_for_direct_arithmetic(range) {
        return sanitize_tolerance(tolerance / range);
    }
    let scale: f64 = std_max(min.abs(), max.abs());
    let scaled_range: f64 = max / scale - min / scale;
    sanitize_tolerance((tolerance / scale) / scaled_range)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_rescales_ranges_beyond_the_geometric_infinity() {
        assert_eq!(transform_to_normalized(5.0, 0.0, 10.0), 0.5);
        assert_eq!(transform_to_normalized(7.0, 1.0, 1.0), 7.0);
        let limit: f64 = 2.0e100;
        assert_eq!(transform_to_normalized(0.0, -limit, limit), 0.5);
        assert_eq!(sanitize_tolerance(-1.0), 0.0);
        assert_eq!(sanitize_tolerance(f64::INFINITY), f64::MAX);
        assert_eq!(normalize_tolerance(limit, -limit, limit), 0.5);
        assert_eq!(normalize_tolerance(3.0, 2.0, 2.0), 3.0);
    }
}
