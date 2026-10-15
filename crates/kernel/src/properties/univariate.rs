// SPDX-License-Identifier: GPL-2.0-only
//! Closed-form one-variable function algebra shared by the analytic face integrals.
//! Each implementation integrates exactly; the magnitude feeds the rounding bound.

/// A definite integral and the sum of the absolute values of the terms that formed it.
/// The magnitude scales the floating-point rounding bound; it is not an error itself.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TermSum {
    pub(crate) value: f64,
    pub(crate) magnitude: f64,
}

impl TermSum {
    /// One term `value`; `TermSum::term(2.0)`.
    pub(crate) fn term(value: f64) -> Self {
        Self {
            value,
            magnitude: value.abs(),
        }
    }

    /// Sum of two term sums; `a.plus(b)`.
    pub(crate) fn plus(self, other: Self) -> Self {
        Self {
            value: self.value + other.value,
            magnitude: self.magnitude + other.magnitude,
        }
    }

    /// Every term multiplied by `factor`; `sum.times(-1.0)`.
    pub(crate) fn times(self, factor: f64) -> Self {
        Self {
            value: self.value * factor,
            magnitude: self.magnitude * factor.abs(),
        }
    }
}

/// A one-variable function family closed under sums and products with exact antiderivatives.
/// Example: `Poly::constant(2.0).mul(&Poly::identity()).integral(0.0, 1.0).value == 1.0`.
pub(crate) trait Univariate: Clone + Sized {
    /// The constant function `value`.
    fn constant(value: f64) -> Self;
    /// The pointwise sum.
    fn add(&self, other: &Self) -> Self;
    /// The pointwise product.
    fn mul(&self, other: &Self) -> Self;
    /// The function multiplied by `factor`.
    fn scale(&self, factor: f64) -> Self;
    /// The derivative.
    fn derivative(&self) -> Self;
    /// The value at `t`.
    fn eval(&self, t: f64) -> f64;
    /// The exact definite integral over `[from, to]`, summed term by term.
    fn integral(&self, from: f64, to: f64) -> TermSum;
}
