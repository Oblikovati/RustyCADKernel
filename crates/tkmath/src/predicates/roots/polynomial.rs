// SPDX-License-Identifier: GPL-2.0-only
use super::{DyadicBound, RootIsolationError, error::poll};
use num_bigint::{BigInt, Sign};

/// Ascending primitive integer coefficients; positive content reduction preserves signs.
#[derive(Clone, Debug)]
pub(super) struct IntegerPolynomial(pub Vec<BigInt>);
impl IntegerPolynomial {
    pub fn from_binary64(coefficients: &[f64], limit: u16) -> Result<Self, RootIsolationError> {
        let converted = integer_coefficients(coefficients)?;
        Self::from_integers(converted, limit)
    }
    pub fn from_integers(converted: Vec<BigInt>, limit: u16) -> Result<Self, RootIsolationError> {
        let supplied = converted.len();
        let mut polynomial = Self(converted);
        polynomial.reduce_content();
        if polynomial.0.is_empty() {
            return Err(RootIsolationError::ZeroPolynomial {
                coefficients: supplied,
            });
        }
        if polynomial.degree() > usize::from(limit) {
            return Err(RootIsolationError::Degree {
                actual: polynomial.degree(),
                limit,
            });
        }
        Ok(polynomial)
    }
    pub fn accept_zero(
        result: Result<Self, RootIsolationError>,
    ) -> Result<Self, RootIsolationError> {
        match result {
            Err(RootIsolationError::ZeroPolynomial { .. }) => Ok(Self(Vec::new())),
            other => other,
        }
    }
    pub fn degree(&self) -> usize {
        self.0.len().saturating_sub(1)
    }
    pub fn derivative(&self) -> Self {
        Self(
            self.0
                .iter()
                .enumerate()
                .skip(1)
                .map(|(power, coefficient)| coefficient * power)
                .collect(),
        )
    }
    pub fn sign_at(&self, point: &DyadicBound) -> Sign {
        let (numerator, denominator) = point.ratio();
        let mut value = BigInt::from(0);
        let mut power = BigInt::from(1);
        for coefficient in self.0.iter().rev() {
            value = value * &numerator + coefficient * &power;
            power *= &denominator;
        }
        value.sign()
    }
    pub fn endpoint_signs(&self, point: &DyadicBound) -> (bool, Sign) {
        let at = self.sign_at(point);
        if at == Sign::NoSign {
            return (true, self.derivative().right_sign(point));
        }
        (false, at)
    }
    pub fn right_sign(&self, point: &DyadicBound) -> Sign {
        let mut derivative = self.clone();
        while !derivative.0.is_empty() {
            let sign = derivative.sign_at(point);
            if sign != Sign::NoSign {
                return sign;
            }
            derivative = derivative.derivative();
        }
        Sign::NoSign
    }
    pub fn negative_remainder(
        &self,
        divisor: &Self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self, RootIsolationError> {
        let mut remainder = self.clone();
        while !remainder.0.is_empty() && remainder.degree() >= divisor.degree() {
            poll(cancelled)?;
            remainder.cancel_leading_term(divisor);
            remainder.reduce_content();
        }
        Ok(remainder.negated())
    }
    pub fn negated(mut self) -> Self {
        for coefficient in &mut self.0 {
            *coefficient = -coefficient.clone();
        }
        self
    }
    pub fn multiply(
        &self,
        other: &Self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self, RootIsolationError> {
        if self.0.is_empty() || other.0.is_empty() {
            return Ok(Self(Vec::new()));
        }
        let mut product = vec![BigInt::from(0); self.0.len() + other.0.len() - 1];
        for (left_power, left) in self.0.iter().enumerate() {
            for (right_power, right) in other.0.iter().enumerate() {
                poll(cancelled)?;
                product[left_power + right_power] += left * right;
            }
        }
        let mut polynomial = Self(product);
        polynomial.reduce_content();
        Ok(polynomial)
    }
    fn cancel_leading_term(&mut self, divisor: &Self) {
        let offset = self.degree() - divisor.degree();
        let leading = self.0[self.degree()].clone();
        let divisor_leading = &divisor.0[divisor.degree()];
        let scale = positive(divisor_leading.clone());
        let factor = if divisor_leading.sign() == Sign::Minus {
            -leading
        } else {
            leading
        };
        for coefficient in &mut self.0 {
            *coefficient *= &scale;
        }
        for (index, coefficient) in divisor.0.iter().enumerate() {
            self.0[index + offset] -= &factor * coefficient;
        }
    }
    fn reduce_content(&mut self) {
        while self
            .0
            .last()
            .is_some_and(|value| value.sign() == Sign::NoSign)
        {
            self.0.pop();
        }
        let content = self.0.iter().fold(BigInt::from(0), |gcd, value| {
            positive_gcd(gcd, positive(value.clone()))
        });
        if content <= BigInt::from(1) {
            return;
        }
        for coefficient in &mut self.0 {
            *coefficient /= &content;
        }
    }
}
fn positive(value: BigInt) -> BigInt {
    if value.sign() == Sign::Minus {
        return -value;
    }
    value
}
fn positive_gcd(mut left: BigInt, mut right: BigInt) -> BigInt {
    while right.sign() != Sign::NoSign {
        let remainder = left % &right;
        left = right;
        right = remainder;
    }
    left
}

pub(super) fn integer_coefficients(
    coefficients: &[f64],
) -> Result<Vec<BigInt>, RootIsolationError> {
    coefficients
        .iter()
        .enumerate()
        .map(|(index, value)| {
            super::super::exact_coordinate(*value, index, 0).map_err(|_| {
                RootIsolationError::Coefficient {
                    index,
                    value: *value,
                }
            })
        })
        .collect()
}
