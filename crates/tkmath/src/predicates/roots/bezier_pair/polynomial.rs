// SPDX-License-Identifier: GPL-2.0-only
//! Raw coefficients preserve magnitude; primitive normalization belongs at solver boundaries.
use super::super::{
    RootIsolationError, certificate_arithmetic::convolution::raw_coefficient_product,
    polynomial::IntegerPolynomial,
};
use num_bigint::{BigInt, Sign};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ExactCoefficientPolynomial(pub Vec<BigInt>);
impl ExactCoefficientPolynomial {
    pub fn new(mut coefficients: Vec<BigInt>) -> Self {
        while coefficients
            .last()
            .is_some_and(|term| term.sign() == Sign::NoSign)
        {
            coefficients.pop();
        }
        Self(coefficients)
    }
    #[cfg(test)]
    pub fn constant(value: BigInt) -> Self {
        let coefficients = vec![value];
        Self::new(coefficients)
    }
    pub fn coefficient(&self, power: usize) -> BigInt {
        // Missing powers are the additive identity, not a discarded geometric failure.
        self.0
            .get(power)
            .cloned()
            .unwrap_or_else(|| BigInt::from(0))
    }
    pub fn degree(&self) -> Option<usize> {
        let length = self.0.len();
        length.checked_sub(1)
    }
    pub fn add(&self, other: &Self) -> Self {
        let length = self.0.len().max(other.0.len());
        let coefficients = (0..length)
            .map(|i| self.coefficient(i) + other.coefficient(i))
            .collect();
        Self::new(coefficients)
    }
    pub fn subtract(&self, other: &Self) -> Self {
        let length = self.0.len().max(other.0.len());
        let coefficients = (0..length)
            .map(|i| self.coefficient(i) - other.coefficient(i))
            .collect();
        Self::new(coefficients)
    }
    pub fn scaled(&self, factor: &BigInt) -> Self {
        let coefficients = self.0.iter().map(|term| term * factor).collect();
        Self::new(coefficients)
    }
    pub fn shifted(&self) -> Self {
        let mut coefficients = vec![BigInt::from(0)];
        coefficients.extend(self.0.iter().cloned());
        Self::new(coefficients)
    }
    pub fn multiply(
        &self,
        other: &Self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self, RootIsolationError> {
        let coefficients = raw_coefficient_product(&self.0, &other.0, cancelled)?;
        Ok(Self::new(coefficients))
    }
    pub fn primitive(&self, limit: u16) -> Result<IntegerPolynomial, RootIsolationError> {
        // Applying content reduction earlier would corrupt sums in Fx/Fy substitution.
        IntegerPolynomial::from_integers(self.0.clone(), limit)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn coefficients(values: &[i32]) -> ExactCoefficientPolynomial {
        let values = values.iter().map(|value| BigInt::from(*value)).collect();
        ExactCoefficientPolynomial::new(values)
    }
    #[test]
    fn ring_preserves_content_before_addition_and_only_normalizes_solver_copy() {
        let a = coefficients(&[2, 2]);
        let b = coefficients(&[3, -3]);
        let product = a.multiply(&b, &mut || false).unwrap();
        assert_eq!(product, coefficients(&[6, 0, -6]));
        assert_eq!(product.add(&coefficients(&[3])), coefficients(&[9, 0, -6]));
        assert_eq!(product.primitive(2).unwrap().0, coefficients(&[1, 0, -1]).0);
        assert_eq!(product, coefficients(&[6, 0, -6]));
    }
    #[test]
    fn zero_degree_shift_scale_subtraction_and_cancel_are_exact() {
        let p = coefficients(&[1, 2, 0]);
        assert_eq!(p.degree(), Some(1));
        assert_eq!(p.subtract(&p).degree(), None);
        assert_eq!(p.scaled(&BigInt::from(-3)), coefficients(&[-3, -6]));
        assert_eq!(p.shifted(), coefficients(&[0, 1, 2]));
        assert_eq!(
            ExactCoefficientPolynomial::constant(0.into()).degree(),
            None
        );
        assert_eq!(
            p.multiply(&p, &mut || true),
            Err(RootIsolationError::Cancelled)
        );
    }
}
