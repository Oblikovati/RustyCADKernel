//! glibc 2.39 `math/s_csqrt_template.c` for `double`, reached by libstdc++'s
//! `std::sqrt(std::complex<double>)` through `__builtin_csqrt`.
use super::super::extended::scale_double;
/// `-((DBL_MANT_DIG + 1) / 2)`.
const SUBNORMAL_SCALE: i32 = -27;
/// Returns (real, imaginary) of the principal square root of `re + i*im`.
pub(super) fn csqrt(re: f64, im: f64) -> (f64, f64) {
    if !re.is_finite() || !im.is_finite() {
        return nonfinite(re, im);
    }
    if im == 0.0 {
        if re < 0.0 {
            return (0.0, (-re).sqrt().copysign(im));
        }
        return (re.sqrt().abs(), 0.0_f64.copysign(im));
    }
    if re == 0.0 {
        let r: f64 = if im.abs() >= 2.0 * f64::MIN_POSITIVE {
            (0.5 * im.abs()).sqrt()
        } else {
            0.5 * (2.0 * im.abs()).sqrt()
        };
        return (r, r.copysign(im));
    }
    general(re, im)
}
fn nonfinite(re: f64, im: f64) -> (f64, f64) {
    if im.is_infinite() {
        return (f64::INFINITY, im);
    }
    if !re.is_infinite() {
        return (f64::NAN, f64::NAN);
    }
    if re < 0.0 {
        let real: f64 = if im.is_nan() { f64::NAN } else { 0.0 };
        return (real, f64::INFINITY.copysign(im));
    }
    (
        re,
        if im.is_nan() {
            f64::NAN
        } else {
            0.0_f64.copysign(im)
        },
    )
}
/// Scales finite nonzero inputs away from overflow and subnormal ranges.
fn scaled_input(re: f64, im: f64) -> (f64, f64, i32) {
    if re.abs() > f64::MAX / 4.0 {
        return (scale_double(re, -2), scale_double(im, -2), 1);
    }
    if im.abs() > f64::MAX / 4.0 {
        let real: f64 = if re.abs() >= 4.0 * f64::MIN_POSITIVE {
            scale_double(re, -2)
        } else {
            0.0
        };
        return (real, scale_double(im, -2), 1);
    }
    if re.abs() < 2.0 * f64::MIN_POSITIVE && im.abs() < 2.0 * f64::MIN_POSITIVE {
        let factor: i32 = -2 * SUBNORMAL_SCALE;
        return (
            scale_double(re, factor),
            scale_double(im, factor),
            SUBNORMAL_SCALE,
        );
    }
    (re, im, 0)
}
fn general(re: f64, im: f64) -> (f64, f64) {
    let (re, im, scale) = scaled_input(re, im);
    let d: f64 = re.hypot(im);
    // 2 Re(res) Im(res) = Im(x) avoids cancellation in d +/- Re(x).
    let (mut r, mut s, scale): (f64, f64, i32) = if re > 0.0 {
        positive_real(re, im, d, scale)
    } else {
        nonpositive_real(re, im, d, scale)
    };
    if scale != 0 {
        r = scale_double(r, scale);
        s = scale_double(s, scale);
    }
    (r, s.copysign(im))
}
/// Returns (r, s, remaining scale); a small `im` rescales early to avoid underflow.
fn positive_real(re: f64, im: f64, d: f64, scale: i32) -> (f64, f64, i32) {
    let r: f64 = (0.5 * (d + re)).sqrt();
    if scale == 1 && im.abs() < 1.0 {
        return (scale_double(r, scale), im / r, 0);
    }
    (r, 0.5 * (im / r), scale)
}
fn nonpositive_real(re: f64, im: f64, d: f64, scale: i32) -> (f64, f64, i32) {
    let s: f64 = (0.5 * (d - re)).sqrt();
    if scale == 1 && im.abs() < 1.0 {
        return ((im / s).abs(), scale_double(s, scale), 0);
    }
    ((0.5 * (im / s)).abs(), s, scale)
}
