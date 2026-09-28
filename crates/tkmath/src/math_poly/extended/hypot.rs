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

//! glibc 2.39 `sysdeps/ieee754/ldbl-96/e_hypotl.c` (Borges MyHypot3) on software x87.
//!
//! `std::abs(std::complex<long double>)` lowers to `cabsl`, which glibc forwards to this
//! `hypotl`. The pinned oracle profile runs the C kernel, not an x87 `fsqrt(x*x+y*y)`.
use super::Extended;
/// `SCALE = 0x8p-8257L`.
const SCALE_EXPONENT: i32 = -8254;
/// `TINY_VAL = 0x8p-8194L`.
const TINY_EXPONENT: i32 = -8191;
/// `EPS = 0x8p-68L`.
const EPS_EXPONENT: i32 = -65;
/// `LARGE_VAL = 0xb.504f333f9de6484p+8188L`, i.e. the x87 rounding of sqrt(2)*2^8191.
fn large_value() -> Extended {
    Extended::new(2.0).sqrt().scale(8191)
}
fn power(exponent: i32) -> Extended {
    Extended::new(1.0).scale(exponent)
}
/// Requires `ax >= ay >= 0` with squares of `ax`, `ay` and `ax - ay` in range.
fn kernel(ax: Extended, ay: Extended) -> Extended {
    let two: Extended = Extended::new(2.0);
    let mut h: Extended = (ax * ax + ay * ay).sqrt();
    let (t1, t2): (Extended, Extended) = if h <= two * ay {
        let delta: Extended = h - ay;
        (ax * (two * delta - ax), (delta - two * (ax - ay)) * delta)
    } else {
        let delta: Extended = h - ax;
        (
            two * delta * (ax - two * ay),
            (Extended::new(4.0) * delta - ay) * ay + delta * delta,
        )
    };
    h = h - (t1 + t2) / (two * h);
    h
}
impl Extended {
    /// Native `hypotl`; `Extended::new(3.0).hypot(Extended::new(4.0))` is 5.
    pub(crate) fn hypot(self, other: Self) -> Self {
        if !self.finite() || !other.finite() {
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
fn scaled_down(ax: Extended, ay: Extended) -> Extended {
    if ay <= ax * power(EPS_EXPONENT) {
        return ax + ay;
    }
    let scale: Extended = power(SCALE_EXPONENT);
    kernel(ax * scale, ay * scale) / scale
}
fn scaled_up(ax: Extended, ay: Extended) -> Extended {
    if ax >= ay / power(EPS_EXPONENT) {
        return ax + ay;
    }
    let scale: Extended = power(SCALE_EXPONENT);
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
        let three: Extended = Extended::new(3.0);
        assert_eq!(three.hypot(Extended::new(-4.0)).narrow(), 5.0);
        assert!(
            Extended::new(f64::NAN)
                .hypot(Extended::infinity())
                .is_infinite()
        );
        assert!(Extended::new(f64::NAN).hypot(three).is_nan());
        assert_eq!(Extended::new(-2.0).hypot(Extended::ZERO).narrow(), 2.0);
    }
}
