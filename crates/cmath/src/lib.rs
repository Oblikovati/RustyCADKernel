//! Platform C math functions that Rust's standard library re-implements differently.
//!
//! OCCT calls the C library's inverse hyperbolic functions. Rust's `f64::asinh`,
//! `f64::acosh` and `f64::atanh` are portable formulas whose last bit can differ,
//! so bit-exact ports call the same libm entry points that the native build links.
//! Other Rust float methods (`ln`, `sin`, `sqrt`, ...) already reach the platform
//! libm or are correctly rounded, so they need no adapter. Rust has no IEEE
//! `remainder`, so it is also taken from libm.
//!
//! The only unsafe construct is the foreign declaration: each function is a pure
//! function of its argument with no preconditions, so it is declared `safe`.

mod ffi {
    unsafe extern "C" {
        pub safe fn asinh(value: f64) -> f64;
        pub safe fn acosh(value: f64) -> f64;
        pub safe fn atanh(value: f64) -> f64;
        pub safe fn remainder(numerator: f64, denominator: f64) -> f64;
    }
}

/// Inverse hyperbolic sine from the platform libm, as `asinh(x)` in C++.
///
/// ```
/// assert_eq!(rustycad_cmath::asinh(0.0), 0.0);
/// ```
pub fn asinh(value: f64) -> f64 {
    ffi::asinh(value)
}

/// Inverse hyperbolic cosine from the platform libm, as `acosh(x)` in C++.
///
/// ```
/// assert_eq!(rustycad_cmath::acosh(1.0), 0.0);
/// ```
pub fn acosh(value: f64) -> f64 {
    ffi::acosh(value)
}

/// Inverse hyperbolic tangent from the platform libm, as `atanh(x)` in C++.
///
/// ```
/// assert_eq!(rustycad_cmath::atanh(0.0), 0.0);
/// ```
pub fn atanh(value: f64) -> f64 {
    ffi::atanh(value)
}

/// IEEE 754 remainder from the platform libm, as `std::remainder(x, y)` in C++:
/// `x - n*y` with `n` the integer nearest to `x/y` (ties to even).
///
/// ```
/// assert_eq!(rustycad_cmath::remainder(7.0, 2.0), -1.0);
/// ```
pub fn remainder(numerator: f64, denominator: f64) -> f64 {
    ffi::remainder(numerator, denominator)
}
