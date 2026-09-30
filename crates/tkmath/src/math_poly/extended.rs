//! Software x87 `long double` of the pinned Linux/GCC x86-64 oracle profile.
//!
//! MathPoly (and the modern solvers that reuse it) compute in `long double`, which GCC on
//! x86-64 maps to the 80-bit x87 extended format: 64-bit significand, 15-bit exponent,
//! round-to-nearest-even on every operation. Rust has no such type, so `LongDouble` wraps
//! rustc_apfloat's `X87DoubleExtended` and rounds each operation exactly as the x87 unit
//! does. glibc's `sqrtl` and `hypotl` are reproduced in software (`sqrt`, `hypot`).
use rustc_apfloat::ieee::{Double, X87DoubleExtended};
use rustc_apfloat::{Float, FloatConvert, Round};
use std::ops::{Add, Div, Mul, Neg, Sub};
/// An x87 80-bit extended value, the pinned profile's `long double`; `LongDouble::new(0.1)`.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct LongDouble(X87DoubleExtended);
#[cfg(test)]
mod complex_fixture;
mod hypot;
mod sqrt;
impl std::fmt::Debug for LongDouble {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "LongDouble({:#022x})", self.to_bits())
    }
}
impl From<f64> for LongDouble {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}
impl LongDouble {
    /// Positive zero; `LongDouble::ZERO`.
    pub const ZERO: Self = Self(X87DoubleExtended::ZERO);
    /// Exact widening `static_cast<long double>(double)`; `LongDouble::new(0.1)`.
    pub fn new(value: f64) -> Self {
        Self(
            Double::from_bits(value.to_bits() as u128)
                .convert(&mut false)
                .value,
        )
    }
    /// Builds a value from its 80-bit pattern (sign and exponent in bits 64..80, explicit
    /// integer bit at 63); `LongDouble::from_bits(0x3fff_8000000000000000)` is 1.
    pub fn from_bits(bits: u128) -> Self {
        Self(X87DoubleExtended::from_bits(bits))
    }
    /// Returns the 80-bit pattern; `LongDouble::new(1.0).to_bits()`.
    pub fn to_bits(self) -> u128 {
        self.0.to_bits()
    }
    /// `LDBL_EPSILON` = 2^-63; `LongDouble::epsilon()`.
    pub fn epsilon() -> Self {
        Self::new(1.0).scale(-63)
    }
    /// `LDBL_MAX`; `LongDouble::largest()`.
    pub fn largest() -> Self {
        Self(X87DoubleExtended::largest())
    }
    /// `LDBL_MIN`, the smallest normal value; `LongDouble::smallest_normal()`.
    pub fn smallest_normal() -> Self {
        Self(X87DoubleExtended::smallest_normalized())
    }
    /// Positive infinity; `LongDouble::infinity()`.
    pub fn infinity() -> Self {
        Self(X87DoubleExtended::INFINITY)
    }
    /// Exact conversion of an unsigned count (`static_cast<long double>(size_t)`);
    /// `LongDouble::from_usize(3)`.
    pub fn from_usize(value: usize) -> Self {
        Self(X87DoubleExtended::from_u128(value as u128).value)
    }
    /// Truncating conversion to an unsigned count; `LongDouble::new(3.7).to_usize()` is 3.
    pub fn to_usize(self) -> usize {
        self.0.to_u128(usize::BITS as usize).value as usize
    }
    /// Unchecked `static_cast<double>`: rounds to nearest, overflow yields an infinity as on
    /// x87; `LongDouble::new(0.5).narrow()`.
    pub fn narrow(self) -> f64 {
        let double: Double = self.0.convert(&mut false).value;
        f64::from_bits(double.to_bits() as u64)
    }
}
impl LongDouble {
    /// `std::abs`; `LongDouble::new(-2.0).abs()`.
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }
    /// `std::isfinite`; `LongDouble::new(1.0).is_finite()`.
    pub fn is_finite(self) -> bool {
        !self.0.is_infinite() && !self.0.is_nan()
    }
    /// `std::isnan`; `LongDouble::infinity().is_nan()` is false.
    pub fn is_nan(self) -> bool {
        self.0.is_nan()
    }
    /// `std::isinf`; `LongDouble::infinity().is_infinite()`.
    pub fn is_infinite(self) -> bool {
        self.0.is_infinite()
    }
    /// `std::max(self, other)`: returns `other` only when `self < other`, so a NaN `self`
    /// is kept; `LongDouble::new(1.0).max(LongDouble::new(2.0))`.
    pub fn max(self, other: Self) -> Self {
        if self < other { other } else { self }
    }
    /// `std::scalbn` (exact multiplication by 2^exponent, with x87 over/underflow);
    /// `LongDouble::new(1.0).scale(3)` is 8.
    pub fn scale(self, exponent: i32) -> Self {
        Self(self.0.scalbn(exponent))
    }
    /// `std::floor`; `LongDouble::new(-1.5).floor()` is -2.
    pub fn floor(self) -> Self {
        Self(self.0.round_to_integral(Round::TowardNegative).value)
    }
    /// `std::ceil`; `LongDouble::new(-1.5).ceil()` is -1.
    pub fn ceil(self) -> Self {
        Self(self.0.round_to_integral(Round::TowardPositive).value)
    }
    /// `static_cast<int>(std::ceil(x))` for values in the int range;
    /// `LongDouble::new(2.5).ceil_integer()` is 3.
    pub fn ceil_integer(self) -> i32 {
        self.ceil().0.to_i128(32).value as i32
    }
    /// `copysignl`: magnitude of `self`, sign bit of `sign`;
    /// `LongDouble::new(1.0).copysign(LongDouble::new(-0.0))` is -1.
    pub fn copysign(self, sign: Self) -> Self {
        Self(self.0.copy_sign(sign.0))
    }
}
impl Add for LongDouble {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self((self.0 + rhs.0).value)
    }
}
impl Sub for LongDouble {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self((self.0 - rhs.0).value)
    }
}
impl Mul for LongDouble {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self((self.0 * rhs.0).value)
    }
}
impl Div for LongDouble {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self((self.0 / rhs.0).value)
    }
}
impl Neg for LongDouble {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}
/// `std::ilogb` on a double, with rustc_apfloat's special results: `i32::MAX` for an
/// infinity, `i32::MIN` for a NaN and `i32::MIN + 1` for zero (glibc returns `INT_MIN`).
pub(crate) fn exponent(value: f64) -> i32 {
    Double::from_bits(value.to_bits() as u128).ilogb()
}
/// `std::scalbn` on a double; `scale_double(1.0, -3)` is 0.125.
pub(crate) fn scale_double(value: f64, power: i32) -> f64 {
    f64::from_bits(
        Double::from_bits(value.to_bits() as u128)
            .scalbn(power)
            .to_bits() as u64,
    )
}
