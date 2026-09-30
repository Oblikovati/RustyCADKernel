//! glibc 2.39 `sysdeps/ieee754/ldbl-96/e_hypotl.c` (Borges MyHypot3) on software x87.
//!
//! `std::abs(std::complex<long double>)` lowers to `cabsl`, which glibc forwards to this
//! `hypotl`. The pinned oracle profile runs the C kernel, not an x87 `fsqrt(x*x+y*y)`.
use super::LongDouble;
/// `SCALE = 0x8p-8257L`.
const SCALE_EXPONENT: i32 = -8254;
/// `TINY_VAL = 0x8p-8194L`.
const TINY_EXPONENT: i32 = -8191;
/// `EPS = 0x8p-68L`.
const EPS_EXPONENT: i32 = -65;
/// `LARGE_VAL = 0xb.504f333f9de6484p+8188L`, i.e. the x87 rounding of sqrt(2)*2^8191.
fn large_value() -> LongDouble {
    LongDouble::new(2.0).sqrt().scale(8191)
}
fn power(exponent: i32) -> LongDouble {
    LongDouble::new(1.0).scale(exponent)
}
/// Requires `ax >= ay >= 0` with squares of `ax`, `ay` and `ax - ay` in range.
fn kernel(ax: LongDouble, ay: LongDouble) -> LongDouble {
    let two: LongDouble = LongDouble::new(2.0);
    let mut h: LongDouble = (ax * ax + ay * ay).sqrt();
    let (t1, t2): (LongDouble, LongDouble) = if h <= two * ay {
        let delta: LongDouble = h - ay;
        (ax * (two * delta - ax), (delta - two * (ax - ay)) * delta)
    } else {
        let delta: LongDouble = h - ax;
        (
            two * delta * (ax - two * ay),
            (LongDouble::new(4.0) * delta - ay) * ay + delta * delta,
        )
    };
    h = h - (t1 + t2) / (two * h);
    h
}
impl LongDouble {
    /// Native `hypotl`; `LongDouble::new(3.0).hypot(LongDouble::new(4.0))` is 5.
    pub(crate) fn hypot(self, other: Self) -> Self {
        if !self.is_finite() || !other.is_finite() {
            // Signalling NaNs never reach this port: arithmetic quiets them first.
            if self.is_infinite() || other.is_infinite() {
                return Self::infinity();
            }
            return self + other;
        }
        let (x, y): (Self, Self) = (self.abs(), other.abs());
        let (ax, ay): (Self, Self) = if x < y { (y, x) } else { (x, y) };
        if ax > large_value() {
            return scaled_down(ax, ay);
        }
        if ay < power(TINY_EXPONENT) {
            return scaled_up(ax, ay);
        }
        if ay <= ax * power(EPS_EXPONENT) {
            return ax + ay;
        }
        kernel(ax, ay)
    }
}
fn scaled_down(ax: LongDouble, ay: LongDouble) -> LongDouble {
    if ay <= ax * power(EPS_EXPONENT) {
        return ax + ay;
    }
    let scale: LongDouble = power(SCALE_EXPONENT);
    kernel(ax * scale, ay * scale) / scale
}
fn scaled_up(ax: LongDouble, ay: LongDouble) -> LongDouble {
    if ax >= ay / power(EPS_EXPONENT) {
        return ax + ay;
    }
    let scale: LongDouble = power(SCALE_EXPONENT);
    kernel(ax / scale, ay / scale) * scale
}
#[cfg(test)]
mod tests {
    use super::*;
    use rustc_apfloat::Float;
    #[test]
    fn large_value_matches_glibc_literal() {
        assert_eq!(large_value().0.to_bits(), 0x5ffe_b504f333f9de6484);
    }
    #[test]
    fn hypot_special_values() {
        let three: LongDouble = LongDouble::new(3.0);
        assert_eq!(three.hypot(LongDouble::new(-4.0)).narrow(), 5.0);
        assert!(
            LongDouble::new(f64::NAN)
                .hypot(LongDouble::infinity())
                .is_infinite()
        );
        assert!(LongDouble::new(f64::NAN).hypot(three).is_nan());
        assert_eq!(LongDouble::new(-2.0).hypot(LongDouble::ZERO).narrow(), 2.0);
    }
}
