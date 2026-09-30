//! `std::complex<double>` as MathPoly uses it under the pinned GCC 13 / glibc 2.39 profile.
//!
//! Operator semantics follow GCC's complex lowering: complex-by-real operations are
//! componentwise, adding a real leaves the imaginary part untouched, and full complex
//! products and quotients follow libgcc (`gcc.rs`).
use std::ops::{Add, Div, Mul, Sub};
mod csqrt;
pub(crate) mod gcc;
pub(crate) mod long;
/// Kernel-owned complex storage; `Complex::new(1.0,2.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex {
    value: num_complex::Complex64,
}
impl Complex {
    /// Builds Cartesian coordinates; `Complex::new(1.0,2.0)`.
    pub const fn new(real: f64, imag: f64) -> Self {
        Self {
            value: num_complex::Complex64::new(real, imag),
        }
    }
    /// Returns the real coordinate; `Complex::new(1.0,2.0).real()`.
    pub fn real(self) -> f64 {
        self.value.re
    }
    /// Returns the imaginary coordinate; `Complex::new(1.0,2.0).imag()`.
    pub fn imag(self) -> f64 {
        self.value.im
    }
    /// `std::polar`: `(rho*cos(theta), rho*sin(theta))`; `Complex::polar(2.0, 0.0)`.
    pub fn polar(rho: f64, theta: f64) -> Self {
        Self::new(rho * theta.cos(), rho * theta.sin())
    }
    /// `std::abs` through glibc `cabs`/`hypot`; `Complex::new(3.0,4.0).abs()` is 5.
    pub fn abs(self) -> f64 {
        self.real().hypot(self.imag())
    }
    /// `std::norm` as libstdc++ 13 computes it, `x*x + y*y`; `Complex::new(3.0,4.0).norm()`.
    pub fn norm(self) -> f64 {
        self.real() * self.real() + self.imag() * self.imag()
    }
    /// `std::conj`; `Complex::new(1.0,2.0).conj()` is `(1,-2)`.
    pub fn conj(self) -> Self {
        Self::new(self.real(), -self.imag())
    }
    /// `std::sqrt` through glibc `csqrt`; `Complex::new(-4.0,0.0).sqrt()` is `(0,2)`.
    pub fn sqrt(self) -> Self {
        let (re, im) = csqrt::csqrt(self.real(), self.imag());
        Self::new(re, im)
    }
    /// Both coordinates finite (MathPoly `Utils::IsFinite`); `Complex::new(1.0,0.0).is_finite()`.
    pub fn is_finite(self) -> bool {
        self.real().is_finite() && self.imag().is_finite()
    }
}
impl Mul for Complex {
    type Output = Self;
    /// libgcc `__muldc3` semantics, including infinity recovery.
    fn mul(self, rhs: Self) -> Self {
        let (re, im) = gcc::multiply(self.real(), self.imag(), rhs.real(), rhs.imag());
        Self::new(re, im)
    }
}
impl Div for Complex {
    type Output = Self;
    /// libgcc `__divdc3` scaled Smith division.
    fn div(self, rhs: Self) -> Self {
        let (re, im) = gcc::divide(self.real(), self.imag(), rhs.real(), rhs.imag());
        Self::new(re, im)
    }
}
impl Add for Complex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.real() + rhs.real(), self.imag() + rhs.imag())
    }
}
impl Sub for Complex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.real() - rhs.real(), self.imag() - rhs.imag())
    }
}
impl Add<f64> for Complex {
    type Output = Self;
    /// A real addend never touches the imaginary part, preserving its signed zero.
    fn add(self, rhs: f64) -> Self {
        Self::new(self.real() + rhs, self.imag())
    }
}
impl Mul<f64> for Complex {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.real() * rhs, self.imag() * rhs)
    }
}
impl Mul<Complex> for f64 {
    type Output = Complex;
    fn mul(self, rhs: Complex) -> Complex {
        rhs * self
    }
}
