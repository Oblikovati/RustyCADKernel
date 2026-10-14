// SPDX-License-Identifier: GPL-2.0-only
//! Exact rational power-basis polynomials for derived incidence equations.
use super::super::certificate_arithmetic::{ExactRatio, RatioInterval};
use super::super::{RootIsolationError, polynomial::IntegerPolynomial};
use num_bigint::{BigInt, Sign};

/// Ascending exact coefficients; trailing zeros are allowed and ignored by conversion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RationalPolynomial(pub Vec<ExactRatio>);
pub(super) fn ratio(value: f64) -> ExactRatio {
    ExactRatio::from_finite(value)
}
pub(super) fn zero() -> ExactRatio {
    ratio(0.)
}
impl RationalPolynomial {
    pub fn constant(value: ExactRatio) -> Self {
        Self(vec![value])
    }
    pub fn linear(constant: ExactRatio, slope: ExactRatio) -> Self {
        Self(vec![constant, slope])
    }
    pub fn add(&self, other: &Self) -> Self {
        let length = self.0.len().max(other.0.len());
        Self(
            (0..length)
                .map(|index| self.coefficient(index).add(&other.coefficient(index)))
                .collect(),
        )
    }
    pub fn scale(&self, factor: &ExactRatio) -> Self {
        Self(self.0.iter().map(|value| value.multiply(factor)).collect())
    }
    pub fn multiply(&self, other: &Self) -> Self {
        if self.0.is_empty() || other.0.is_empty() {
            return Self(Vec::new());
        }
        let mut product = vec![zero(); self.0.len() + other.0.len() - 1];
        for (left_power, left) in self.0.iter().enumerate() {
            for (right_power, right) in other.0.iter().enumerate() {
                product[left_power + right_power] =
                    product[left_power + right_power].add(&left.multiply(right));
            }
        }
        Self(product)
    }
    pub fn derivative(&self) -> Self {
        Self(
            self.0
                .iter()
                .enumerate()
                .skip(1)
                .map(|(power, value)| value.multiply(&ExactRatio::new(power.into(), 1.into())))
                .collect(),
        )
    }
    pub fn coefficient(&self, index: usize) -> ExactRatio {
        self.0.get(index).cloned().unwrap_or_else(zero)
    }
    /// Degree after removing exact trailing zeros; None for the zero polynomial.
    pub fn degree(&self) -> Option<usize> {
        self.0
            .iter()
            .rposition(|value| value.sign() != std::cmp::Ordering::Equal)
    }
    pub fn evaluate(&self, point: &ExactRatio) -> ExactRatio {
        self.0
            .iter()
            .rev()
            .fold(zero(), |total, value| total.multiply(point).add(value))
    }
    /// Conservative Horner enclosure over a closed rational interval.
    pub fn enclose(&self, parameter: &RatioInterval) -> RatioInterval {
        self.0
            .iter()
            .rev()
            .fold(RatioInterval::point(zero()), |total, value| {
                total
                    .multiply(parameter)
                    .add(&RatioInterval::point(value.clone()))
            })
    }
    /// Scale by the positive least common denominator; None for the zero polynomial.
    pub fn to_integer(&self, limit: u16) -> Result<Option<IntegerPolynomial>, RootIsolationError> {
        if self.degree().is_none() {
            return Ok(None);
        }
        let common = self.0.iter().fold(BigInt::from(1), |lcm, value| {
            least_common_multiple(lcm, value.denominator())
        });
        let integers = self
            .0
            .iter()
            .map(|value| value.numerator() * (&common / value.denominator()))
            .collect();
        IntegerPolynomial::from_integers(integers, limit).map(Some)
    }
}
fn least_common_multiple(left: BigInt, right: &BigInt) -> BigInt {
    let divisor = greatest_common_divisor(left.clone(), right.clone());
    left / divisor * right
}
fn greatest_common_divisor(mut left: BigInt, mut right: BigInt) -> BigInt {
    while right.sign() != Sign::NoSign {
        let remainder = &left % &right;
        left = right;
        right = remainder;
    }
    if left.sign() == Sign::Minus {
        -left
    } else {
        left
    }
}
