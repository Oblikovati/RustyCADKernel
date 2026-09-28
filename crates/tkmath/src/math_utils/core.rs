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

use crate::math::Vector;
/// Native f64 machine epsilon; `EPSILON`.
pub const EPSILON: f64 = f64::EPSILON;
/// Native practical zero threshold; `ZERO_TOL`.
pub const ZERO_TOL: f64 = 1e-15;
/// Native pi; `PI`.
pub const PI: f64 = std::f64::consts::PI;
/// Native twice-pi; `TWO_PI`.
pub const TWO_PI: f64 = std::f64::consts::TAU;
/// Native Euler number; `EULER_NUMBER`.
pub const EULER_NUMBER: f64 = std::f64::consts::E;
/// Golden expansion ratio; `GOLDEN_RATIO`.
pub const GOLDEN_RATIO: f64 = 1.618033988749895;
/// Complementary golden section; `GOLDEN_SECTION`.
pub const GOLDEN_SECTION: f64 = 0.381966011250105;
/// Preserves native comparisons for NaN and reversed bounds; `clamp(x,low,high)`.
pub fn clamp(value: f64, lower: f64, upper: f64) -> f64 {
    if value < lower {
        lower
    } else if value > upper {
        upper
    } else {
        value
    }
}
/// Requires finite value and nonnegative finite tolerance; `is_zero(x,ZERO_TOL)`.
pub fn is_zero(value: f64, tolerance: f64) -> bool {
    value.is_finite() && tolerance.is_finite() && tolerance >= 0.0 && value.abs() <= tolerance
}
/// Uses scale-relative equality without overflowing tolerance products; `is_equal(a,b,ZERO_TOL)`.
pub fn is_equal(a: f64, b: f64, tolerance: f64) -> bool {
    if !a.is_finite() || !b.is_finite() || !tolerance.is_finite() || tolerance < 0.0 {
        return false;
    }
    let difference: f64 = (a - b).abs();
    if !difference.is_finite() {
        return false;
    }
    difference / 1.0_f64.max(a.abs()).max(b.abs()) <= tolerance
}
/// Combines absolute and relative tolerances in native order; `is_within_tolerance(a,b,1e-10,1e-10)`.
pub fn is_within_tolerance(a: f64, b: f64, absolute: f64, relative: f64) -> bool {
    if !a.is_finite()
        || !b.is_finite()
        || !absolute.is_finite()
        || !relative.is_finite()
        || absolute < 0.0
        || relative < 0.0
    {
        return false;
    }
    let difference: f64 = (a - b).abs();
    if !difference.is_finite() {
        return false;
    }
    let scale: f64 = a.abs().max(b.abs());
    scale == 0.0 || difference <= absolute || (difference - absolute) / scale <= relative
}
/// Returns a fallback for native near-zero divisors; `safe_div(a,b,0.0)`.
pub fn safe_div(numerator: f64, denominator: f64, fallback: f64) -> f64 {
    if is_zero(denominator, ZERO_TOL) {
        fallback
    } else {
        numerator / denominator
    }
}
/// Uses native practical-zero sign comparisons; `sign(x)`.
pub fn sign(value: f64) -> i32 {
    if value > ZERO_TOL {
        1
    } else if value < -ZERO_TOL {
        -1
    } else {
        0
    }
}
/// Transfers sign by B>=0, including native NaN behavior; `sign_transfer(a,b)`.
pub fn sign_transfer(a: f64, b: f64) -> f64 {
    if b >= 0.0 { a.abs() } else { -a.abs() }
}
/// Squares with one native multiplication; `square(x)`.
pub const fn square(value: f64) -> f64 {
    value * value
}
/// Cubes with two left-associated multiplications; `cube(x)`.
pub const fn cube(value: f64) -> f64 {
    value * value * value
}
/// Retains native signed cube-root dispatch through the oracle's glibc `cbrt` (Rust's
/// `f64::cbrt` is correctly rounded and can differ by one ulp); `cube_root(x)`.
pub fn cube_root(value: f64) -> f64 {
    if value >= 0.0 {
        super::glibc_cbrt::cbrt(value)
    } else {
        -super::glibc_cbrt::cbrt(-value)
    }
}
/// Tests finite values; `is_finite(x)`.
pub fn is_finite(value: f64) -> bool {
    value.is_finite()
}
/// Uses the pinned Linux oracle's compensated 80-bit accumulator; `dot_product(&a,&b)`.
pub fn dot_product(a: &Vector<'_>, b: &Vector<'_>) -> f64 {
    super::extended::dot_product(a.values(), b.values())
}
/// Computes a scaled norm with the native update order; `vector_norm(&v)`.
pub fn vector_norm(vector: &Vector<'_>) -> f64 {
    let mut scale: f64 = 0.0;
    let mut sum: f64 = 1.0;
    for value in vector.values().iter().map(|v| v.abs()) {
        if !value.is_finite() {
            return value;
        }
        if value == 0.0 {
            continue;
        }
        if scale < value {
            let ratio: f64 = scale / value;
            sum = 1.0 + sum * ratio * ratio;
            scale = value;
        } else {
            let ratio: f64 = value / scale;
            sum += ratio * ratio;
        }
    }
    if scale == 0.0 {
        0.0
    } else {
        scale * sum.sqrt()
    }
}
/// Native norm alias; `norm(&v)`.
pub fn norm(vector: &Vector<'_>) -> f64 {
    vector_norm(vector)
}
/// Squares the scaled norm, allowing native infinity overflow; `norm2(&v)`.
pub fn norm2(vector: &Vector<'_>) -> f64 {
    let result: f64 = vector_norm(vector);
    result * result
}
/// Native maximum ignores NaN entries through ordered comparison; `vector_inf_norm(&v)`.
pub fn vector_inf_norm(vector: &Vector<'_>) -> f64 {
    let mut maximum: f64 = 0.0;
    for &value in vector.values() {
        if maximum < value.abs() {
            maximum = value.abs();
        }
    }
    maximum
}
/// Computes the native power-of-two coefficient scale; `compute_scale_factor(&coefficients)`.
pub fn compute_scale_factor(coefficients: &[f64]) -> f64 {
    if coefficients.is_empty() || coefficients.iter().any(|x| !x.is_finite()) {
        return f64::NAN;
    }
    let maximum: f64 = coefficients.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
    if maximum == 0.0 || (ZERO_TOL..=1e15).contains(&maximum) {
        return 1.0;
    }
    power_of_two(1 - super::glibc_cbrt::frexp(maximum).1)
}
fn power_of_two(exponent: i32) -> f64 {
    if exponent > 1023 {
        return f64::INFINITY;
    }
    if exponent < -1074 {
        return 0.0;
    }
    if exponent < -1022 {
        return f64::from_bits(1_u64 << (exponent + 1074));
    }
    f64::from_bits(((exponent + 1023) as u64) << 52)
}
