//! GCC 13.3 libgcc2.c `__mul?c3`/`__div?c3` complex arithmetic for DF and XF modes.
//!
//! GCC (`-fcx-method=2`, the C/C++ default) inlines `ac-bd, ad+bc` and calls `__muldc3`
//! only when both parts are NaN; `__muldc3` recomputes the same products first, so the
//! whole libgcc routine reproduces both. Full complex division always calls `__divdc3`.
use super::super::extended::LongDouble;
use std::ops::{Add, Div, Mul, Neg, Sub};
/// Scalar operations and mode limits used by the libgcc routines.
pub(crate) trait GccFloat:
    Copy
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    fn literal(value: f64) -> Self;
    fn magnitude(self) -> Self;
    fn nan(self) -> bool;
    fn inf(self) -> bool;
    fn with_sign_of(self, sign: Self) -> Self;
    /// `MAX / 2`.
    fn rbig() -> Self;
    /// Smallest normal value.
    fn rmin() -> Self;
    /// Machine epsilon.
    fn rmin2() -> Self;
    fn infinity() -> Self;
    fn fin(self) -> bool {
        !self.nan() && !self.inf()
    }
    /// `1 / EPSILON`, exact.
    fn rminscal() -> Self {
        Self::literal(1.0) / Self::rmin2()
    }
    /// `RBIG * RMIN2`, exact.
    fn rmax2() -> Self {
        Self::rbig() * Self::rmin2()
    }
    /// libgcc's `COPYSIGN (isinf (v) ? 1 : 0, v)` infinity boxing.
    fn boxed(self) -> Self {
        Self::literal(if self.inf() { 1.0 } else { 0.0 }).with_sign_of(self)
    }
    /// libgcc's `if (isnan (v)) v = COPYSIGN (0, v)`.
    fn unnan(self) -> Self {
        if self.nan() {
            Self::literal(0.0).with_sign_of(self)
        } else {
            self
        }
    }
}
impl GccFloat for f64 {
    fn literal(value: f64) -> Self {
        value
    }
    fn magnitude(self) -> Self {
        self.abs()
    }
    fn nan(self) -> bool {
        self.is_nan()
    }
    fn inf(self) -> bool {
        self.is_infinite()
    }
    fn with_sign_of(self, sign: Self) -> Self {
        self.copysign(sign)
    }
    fn rbig() -> Self {
        f64::MAX / 2.0
    }
    fn rmin() -> Self {
        f64::MIN_POSITIVE
    }
    fn rmin2() -> Self {
        f64::EPSILON
    }
    fn infinity() -> Self {
        f64::INFINITY
    }
}
impl GccFloat for LongDouble {
    fn literal(value: f64) -> Self {
        LongDouble::new(value)
    }
    fn magnitude(self) -> Self {
        self.abs()
    }
    fn nan(self) -> bool {
        self.is_nan()
    }
    fn inf(self) -> bool {
        self.is_infinite()
    }
    fn with_sign_of(self, sign: Self) -> Self {
        self.copysign(sign)
    }
    fn rbig() -> Self {
        LongDouble::largest() / LongDouble::new(2.0)
    }
    fn rmin() -> Self {
        LongDouble::smallest_normal()
    }
    fn rmin2() -> Self {
        // LDBL_EPSILON of the 64-bit x87 significand.
        LongDouble::new(1.0).scale(-63)
    }
    fn infinity() -> Self {
        LongDouble::infinity()
    }
}
/// `__mul?c3(a, b, c, d)` = (a + ib)(c + id).
pub(crate) fn multiply<T: GccFloat>(a: T, b: T, c: T, d: T) -> (T, T) {
    let (ac, bd, ad, bc): (T, T, T, T) = (a * c, b * d, a * d, b * c);
    let (x, y): (T, T) = (ac - bd, ad + bc);
    if !(x.nan() && y.nan()) {
        return (x, y);
    }
    let left_infinite: bool = a.inf() || b.inf();
    let right_infinite: bool = c.inf() || d.inf();
    let product_overflow: bool = ac.inf() || bd.inf() || ad.inf() || bc.inf();
    if !(left_infinite || right_infinite || product_overflow) {
        return (x, y);
    }
    let (a, b, c, d): (T, T, T, T) = boxed_factors(a, b, c, d, left_infinite, right_infinite);
    (
        T::infinity() * (a * c - b * d),
        T::infinity() * (a * d + b * c),
    )
}
/// Recovery of infinities that computed as NaN + iNaN, in libgcc's sequential order.
fn boxed_factors<T: GccFloat>(
    mut a: T,
    mut b: T,
    mut c: T,
    mut d: T,
    left_infinite: bool,
    right_infinite: bool,
) -> (T, T, T, T) {
    if left_infinite {
        (a, b, c, d) = (a.boxed(), b.boxed(), c.unnan(), d.unnan());
    }
    // libgcc re-tests the (possibly boxed) right factor here.
    if c.inf() || d.inf() {
        (a, b, c, d) = (a.unnan(), b.unnan(), c.boxed(), d.boxed());
    }
    if !left_infinite && !right_infinite {
        (a, b, c, d) = (a.unnan(), b.unnan(), c.unnan(), d.unnan());
    }
    (a, b, c, d)
}
/// `__div?c3(a, b, c, d)` = (a + ib)/(c + id) using GCC 12+'s scaled Smith algorithm.
pub(crate) fn divide<T: GccFloat>(a: T, b: T, c: T, d: T) -> (T, T) {
    let imaginary_major: bool = c.magnitude() < d.magnitude();
    // libgcc scales its parameters in place, so the recovery below sees scaled operands.
    let (a, b, c, d) = prescaled([a, b, c, d], if imaginary_major { 3 } else { 2 });
    let (x, y): (T, T) = if imaginary_major {
        imaginary_major_quotient(a, b, c, d)
    } else {
        real_major_quotient(a, b, c, d)
    };
    if x.nan() && y.nan() {
        return recovered_quotient(a, b, c, d, (x, y));
    }
    (x, y)
}
/// Applies libgcc's overflow halving and underflow scaling keyed on the larger divisor part.
fn prescaled<T: GccFloat>(mut v: [T; 4], major: usize) -> (T, T, T, T) {
    if v[major].magnitude() >= T::rbig() {
        v = v.map(|x| x / T::literal(2.0));
    }
    let (a, b, m): (T, T, T) = (v[0].magnitude(), v[1].magnitude(), v[major].magnitude());
    let small_numerator: bool = (a < T::rmin() && b < T::rmax2() && m < T::rmax2())
        || (b < T::rmin() && a < T::rmax2() && m < T::rmax2());
    if m < T::rmin2() || small_numerator {
        v = v.map(|x| x * T::rminscal());
    }
    (v[0], v[1], v[2], v[3])
}
fn imaginary_major_quotient<T: GccFloat>(a: T, b: T, c: T, d: T) -> (T, T) {
    let ratio: T = c / d;
    let denom: T = (c * ratio) + d;
    if ratio.magnitude() > T::rmin() {
        return (((a * ratio) + b) / denom, ((b * ratio) - a) / denom);
    }
    (((c * (a / d)) + b) / denom, ((c * (b / d)) - a) / denom)
}
fn real_major_quotient<T: GccFloat>(a: T, b: T, c: T, d: T) -> (T, T) {
    let ratio: T = d / c;
    let denom: T = (d * ratio) + c;
    if ratio.magnitude() > T::rmin() {
        return (((b * ratio) + a) / denom, (b - (a * ratio)) / denom);
    }
    ((a + (d * (b / c))) / denom, (b - (d * (a / c))) / denom)
}
/// Recovers nonzero/zero, infinite/finite and finite/infinite quotients.
fn recovered_quotient<T: GccFloat>(a: T, b: T, c: T, d: T, nan: (T, T)) -> (T, T) {
    let zero: T = T::literal(0.0);
    if c == zero && d == zero && (!a.nan() || !b.nan()) {
        let signed: T = T::infinity().with_sign_of(c);
        return (signed * a, signed * b);
    }
    if (a.inf() || b.inf()) && c.fin() && d.fin() {
        let (a, b): (T, T) = (a.boxed(), b.boxed());
        return (
            T::infinity() * (a * c + b * d),
            T::infinity() * (b * c - a * d),
        );
    }
    if (c.inf() || d.inf()) && a.fin() && b.fin() {
        let (c, d): (T, T) = (c.boxed(), d.boxed());
        return (zero * (a * c + b * d), zero * (b * c - a * d));
    }
    nan
}
