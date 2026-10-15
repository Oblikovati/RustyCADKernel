// SPDX-License-Identifier: GPL-2.0-only
//! Real polynomials: the profile family of cylinders, cones and straight planar trims.
use super::univariate::{TermSum, Univariate};

/// `coefficients[k]` multiplies `t^k`; `Poly::new(vec![1.0, 2.0])` is `1 + 2t`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Poly {
    coefficients: Vec<f64>,
}

impl Poly {
    /// The polynomial with these ascending coefficients; `Poly::new(vec![0.0, 1.0])`.
    pub(crate) fn new(coefficients: Vec<f64>) -> Self {
        Self { coefficients }
    }

    /// `offset + slope * t`; `Poly::linear(1.0, 2.0)`.
    pub(crate) fn linear(offset: f64, slope: f64) -> Self {
        Self::new(vec![offset, slope])
    }

    fn coefficient(&self, power: usize) -> f64 {
        self.coefficients.get(power).copied().unwrap_or(0.0)
    }
}

impl Univariate for Poly {
    fn constant(value: f64) -> Self {
        Self::new(vec![value])
    }

    fn add(&self, other: &Self) -> Self {
        let length = self.coefficients.len().max(other.coefficients.len());
        Self::new(
            (0..length)
                .map(|k| self.coefficient(k) + other.coefficient(k))
                .collect(),
        )
    }

    fn mul(&self, other: &Self) -> Self {
        let length = self.coefficients.len() + other.coefficients.len() - 1;
        let mut product = vec![0.0; length];
        for (i, a) in self.coefficients.iter().enumerate() {
            for (j, b) in other.coefficients.iter().enumerate() {
                product[i + j] += a * b;
            }
        }
        Self::new(product)
    }

    fn scale(&self, factor: f64) -> Self {
        Self::new(self.coefficients.iter().map(|c| c * factor).collect())
    }

    fn derivative(&self) -> Self {
        let terms = self.coefficients.iter().enumerate().skip(1);
        let derived: Vec<f64> = terms.map(|(k, c)| c * k as f64).collect();
        if derived.is_empty() {
            return Self::constant(0.0);
        }
        Self::new(derived)
    }

    fn eval(&self, t: f64) -> f64 {
        self.coefficients
            .iter()
            .rev()
            .fold(0.0, |sum, c| sum * t + c)
    }

    fn integral(&self, from: f64, to: f64) -> TermSum {
        let mut sum = TermSum::default();
        for (k, c) in self.coefficients.iter().enumerate() {
            let power = (k + 1) as i32;
            let upper = TermSum::term(c * to.powi(power) / power as f64);
            let lower = TermSum::term(c * from.powi(power) / power as f64);
            sum = sum.plus(upper).plus(lower.times(-1.0));
        }
        sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_derivative_and_integral_match_hand_expansion() {
        let square = Poly::linear(1.0, 2.0).mul(&Poly::linear(1.0, 2.0));
        assert_eq!(square, Poly::new(vec![1.0, 4.0, 4.0]));
        assert_eq!(square.derivative(), Poly::new(vec![4.0, 8.0]));
        assert_eq!(square.eval(0.5), 4.0);
        // Integral of (1 + 2t)^2 over [0, 1] is ((3^3) - 1) / 6 = 13/3.
        assert!((square.integral(0.0, 1.0).value - 13.0 / 3.0).abs() < 1e-15);
        assert_eq!(Poly::constant(3.0).derivative(), Poly::constant(0.0));
        assert_eq!(
            Poly::linear(1.0, 1.0)
                .add(&Poly::constant(1.0))
                .scale(2.0)
                .eval(1.0),
            6.0
        );
    }
}
