// Rust translation of FoundationClasses/TKernel/Standard/Standard_Real.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The deprecated Standard_Real wrappers of `<cmath>` (ACos, Cosh, NextAfter, Max, ...).
//!
//! OCCT marks them deprecated but they are public API. Each calls the same C library
//! function as the header: Rust's float methods reach the platform libm for the
//! transcendental functions, and the inverse hyperbolic functions, which Rust computes
//! with its own formulas, go through `rustycad_cmath`. `Max`/`Min` keep the
//! `std::max`/`std::min` comparison, not IEEE `fmax`/`fmin`.

/// `ACos(double)`: `std::acos`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::acos(1.0), 0.0);
/// ```
pub fn acos(value: f64) -> f64 {
    value.acos()
}

/// `ASin(double)`: `std::asin`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::asin(0.0), 0.0);
/// ```
pub fn asin(value: f64) -> f64 {
    value.asin()
}

/// `ATan(double)`: `std::atan`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::atan(0.0), 0.0);
/// ```
pub fn atan(value: f64) -> f64 {
    value.atan()
}

/// `ATan2(double, double)`: `std::atan2(y, x)`; OCCT names the first argument `theX`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::atan2(0.0, 1.0), 0.0);
/// ```
pub fn atan2(first: f64, second: f64) -> f64 {
    first.atan2(second)
}

/// `ATanh(double)`: the C library `atanh` (NaN outside [-1, 1], infinities at the ends).
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::atanh(0.0), 0.0);
/// ```
pub fn atanh(value: f64) -> f64 {
    rustycad_cmath::atanh(value)
}

/// `ACosh(double)`: the C library `acosh` (NaN below 1).
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::acosh(1.0), 0.0);
/// ```
pub fn acosh(value: f64) -> f64 {
    rustycad_cmath::acosh(value)
}

/// `ASinh(double)`: the C library `asinh`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::asinh(0.0), 0.0);
/// ```
pub fn asinh(value: f64) -> f64 {
    rustycad_cmath::asinh(value)
}

/// `Cosh(double)`: `std::cosh`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::cosh(0.0), 1.0);
/// ```
pub fn cosh(value: f64) -> f64 {
    value.cosh()
}

/// `Sinh(double)`: `std::sinh`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::sinh(0.0), 0.0);
/// ```
pub fn sinh(value: f64) -> f64 {
    value.sinh()
}

/// `Tanh(double)`: `std::tanh`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::tanh(0.0), 0.0);
/// ```
pub fn tanh(value: f64) -> f64 {
    value.tanh()
}

/// `Cos(double)`: `std::cos`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::cos(0.0), 1.0);
/// ```
pub fn cos(value: f64) -> f64 {
    value.cos()
}

/// `Sin(double)`: `std::sin`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::sin(0.0), 0.0);
/// ```
pub fn sin(value: f64) -> f64 {
    value.sin()
}

/// `Tan(double)`: `std::tan`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::tan(0.0), 0.0);
/// ```
pub fn tan(value: f64) -> f64 {
    value.tan()
}

/// `Exp(double)`: `std::exp`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::exp(0.0), 1.0);
/// ```
pub fn exp(value: f64) -> f64 {
    value.exp()
}

/// `Log(double)`: the natural logarithm `std::log`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::log(1.0), 0.0);
/// ```
pub fn log(value: f64) -> f64 {
    value.ln()
}

/// `Log10(double)`: `std::log10`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::log10(100.0), 2.0);
/// ```
pub fn log10(value: f64) -> f64 {
    value.log10()
}

/// `Pow(double, double)`: `std::pow`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::pow(2.0, 10.0), 1024.0);
/// ```
pub fn pow(value: f64, power: f64) -> f64 {
    value.powf(power)
}

/// `Sqrt(double)`: the correctly rounded `std::sqrt`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::sqrt(4.0), 2.0);
/// ```
pub fn sqrt(value: f64) -> f64 {
    value.sqrt()
}

/// `NextAfter(double, double)`: the neighbour of `value` toward `direction`, as C `nextafter`.
///
/// Equal arguments return `direction` (so `next_after(0.0, -0.0)` is `-0.0`), and a NaN
/// argument returns the sum of both, which propagates the NaN as the C library does.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::next_after(1.0, 2.0), 1.0 + f64::EPSILON);
/// ```
pub fn next_after(value: f64, direction: f64) -> f64 {
    if value.is_nan() || direction.is_nan() {
        // glibc returns `x + y`, compiled with `y` as the SSE destination operand: when both
        // are NaN the payload of `direction` wins (probe: nextafter(NaN(0x123), -NaN) = -NaN).
        return direction + value;
    }
    if value == direction {
        return direction;
    }
    if direction > value {
        value.next_up()
    } else {
        value.next_down()
    }
}

/// `Sign(double, double)`: `std::copysign`, the magnitude of the first with the sign of the second.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::sign(2.0, -0.0), -2.0);
/// ```
pub fn sign(magnitude: f64, sign_source: f64) -> f64 {
    magnitude.copysign(sign_source)
}

/// `Abs(double)`: `std::abs`, which clears the sign bit (also of `-0.0` and NaN).
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::abs(-2.0), 2.0);
/// ```
pub fn abs(value: f64) -> f64 {
    value.abs()
}

/// `Ceiling(double)`: `std::ceil`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::ceiling(1.2), 2.0);
/// ```
pub fn ceiling(value: f64) -> f64 {
    value.ceil()
}

/// `Floor(double)`: `std::floor`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::floor(-1.2), -2.0);
/// ```
pub fn floor(value: f64) -> f64 {
    value.floor()
}

/// `IntegerPart(double)`: `std::trunc`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::integer_part(-1.7), -1.0);
/// ```
pub fn integer_part(value: f64) -> f64 {
    value.trunc()
}

/// `Round(double)`: `std::round`, halfway cases away from zero.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::real::round(-2.5), -3.0);
/// ```
pub fn round(value: f64) -> f64 {
    value.round()
}

/// `Max(double, double)`: `std::max`, i.e. `first < second ? second : first`.
///
/// Unlike `f64::max`, a NaN in either position is kept when the comparison selects it,
/// and `max(-0.0, 0.0)` returns the first argument, `-0.0`.
///
/// ```
/// assert!(rustycad_tkernel::standard::real::max(f64::NAN, 1.0).is_nan());
/// ```
pub fn max(first: f64, second: f64) -> f64 {
    if first < second { second } else { first }
}

/// `Min(double, double)`: `std::min`, i.e. `second < first ? second : first`.
///
/// ```
/// assert!(rustycad_tkernel::standard::real::min(f64::NAN, 1.0).is_nan());
/// ```
pub fn min(first: f64, second: f64) -> f64 {
    if second < first { second } else { first }
}
