// Rust translation of Standard_Real.hxx, ElCLib::InPeriod and std::min/std::max semantics used by BSplCLib at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Scalar helpers with the exact native semantics BSplCLib and BSplSLib rely on.
use rustycad_tkernel::precision;

/// `std::max(a, b)`: returns `b` only when `a < b`, so NaN and signed-zero ties keep `a`;
/// `std_max(0.0, -0.0).is_sign_positive()`.
pub(crate) fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// `std::min(a, b)`: returns `b` only when `b < a`; `std_min(1.0, 2.0) == 1.0`.
pub(crate) fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// `Epsilon(x)` from Standard_Real.hxx: the gap to the next representable value away
/// from zero; `epsilon(1.0) == f64::EPSILON`.
pub(crate) fn epsilon(value: f64) -> f64 {
    crate::math::real_epsilon(value)
}

/// `std::nextafter(x, RealLast())`; `next_toward_real_last(1.0) > 1.0`.
pub(crate) fn next_toward_real_last(value: f64) -> f64 {
    if value == f64::MAX {
        return value;
    }
    if value == f64::INFINITY {
        return f64::MAX;
    }
    value.next_up()
}

/// `ElCLib::InPeriod`: moves `u` into `[first, last)` without ever returning `last`
/// exactly unless rounding forces it; `in_period(7.0, 0.0, 5.0) == 2.0`.
pub(crate) fn in_period(u: f64, first: f64, last: f64) -> f64 {
    if precision::is_infinite(u) || precision::is_infinite(first) || precision::is_infinite(last) {
        return u;
    }
    let period: f64 = last - first;
    if period < epsilon(last) {
        return u;
    }
    std_max(first, u + period * ((first - u) / period).ceil())
}

/// `RealSmall()`, the smallest positive normal double; `real_small() == f64::MIN_POSITIVE`.
pub(crate) const fn real_small() -> f64 {
    f64::MIN_POSITIVE
}
