//! Platform C math functions that Rust's standard library re-implements differently.
//!
//! OCCT calls the C library's `asinh`; Rust's `f64::asinh` is a portable formula
//! (`ln_1p` of a hypot expression) whose last bit can differ, so bit-exact ports call the
//! same libm entry point that the native build links against.
// The only unsafe construct is the foreign declaration; `asinh` is a pure function of its
// argument, has no preconditions, and is declared `safe` so callers need no unsafe block.
#[allow(unsafe_code)]
mod ffi {
    unsafe extern "C" {
        pub safe fn asinh(value: f64) -> f64;
    }
}

/// Inverse hyperbolic sine from the platform libm, as `asinh(x)` in C++; `asinh(0.5)`.
pub(crate) fn asinh(value: f64) -> f64 {
    ffi::asinh(value)
}
