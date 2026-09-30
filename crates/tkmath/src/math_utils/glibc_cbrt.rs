//! `std::cbrt` of the pinned Linux oracle (glibc 2.39, sysdeps/ieee754/dbl-64/s_cbrt.c).
//!
//! Rust's `f64::cbrt` — and even an `extern "C" cbrt` — resolves to the compiler-builtins
//! libm, which is correctly rounded, while glibc's polynomial-plus-Halley kernel is not: for
//! `2e100` the two differ by one ulp. MathUtils::CubeRoot must reproduce the native bits.
// The literals are glibc's source text verbatim; rustc rounds them exactly as GCC does.
#![allow(clippy::excessive_precision)]

const CBRT2: f64 = 1.2599210498948731648;
const SQR_CBRT2: f64 = 1.5874010519681994748;
const FACTOR: [f64; 5] = [1.0 / SQR_CBRT2, 1.0 / CBRT2, 1.0, CBRT2, SQR_CBRT2];

/// Splits a finite nonzero `x` into a mantissa in [0.5, 1) and a power of two, like `frexp`;
/// zero, infinity and NaN come back unchanged with exponent 0 (glibc's convention).
pub(crate) fn frexp(x: f64) -> (f64, i32) {
    let bits: u64 = x.to_bits();
    let field: i32 = ((bits >> 52) & 0x7ff) as i32;
    if x == 0.0 || field == 0x7ff {
        return (x, 0);
    }
    if field == 0 {
        // Subnormal: scale into the normal range first (2^54 is exact).
        let (mantissa, exponent): (f64, i32) = frexp(x * f64::from_bits((1023 + 54) << 52));
        return (mantissa, exponent - 54);
    }
    let mantissa: f64 = f64::from_bits((bits & !(0x7ff << 52)) | (1022 << 52));
    (mantissa, field - 1022)
}

/// glibc `__cbrt`: polynomial estimate on the reduced mantissa, one Halley step, then the
/// exponent's cube-root factor. `cbrt(8.0)` is 2.
pub(crate) fn cbrt(x: f64) -> f64 {
    let (xm, xe): (f64, i32) = frexp(x.abs());
    if xe == 0 && (x.is_nan() || x.is_infinite() || x == 0.0) {
        return x + x;
    }
    let u: f64 = 0.354895765043919860
        + ((1.50819193781584896
            + ((-2.11499494167371287
                + ((2.44693122563534430
                    + ((-1.83469277483613086
                        + (0.784932344976639262 - 0.145263899385486377 * xm) * xm)
                        * xm))
                    * xm))
                * xm))
            * xm);
    let t2: f64 = u * u * u;
    let ym: f64 = u * (t2 + 2.0 * xm) / (2.0 * t2 + xm) * FACTOR[(2 + xe % 3) as usize];
    // __ldexp by an exponent in [-358, 341]: the power of two is a normal double, so exact.
    let scale: f64 = f64::from_bits(((xe / 3 + 1023) as u64) << 52);
    (if x > 0.0 { ym } else { -ym }) * scale
}
