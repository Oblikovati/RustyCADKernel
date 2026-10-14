// SPDX-License-Identifier: GPL-2.0-only
//! The owned exact-ring dependency stays behind Martensite's borrowed coefficient view.
use num_bigint::BigInt;
/// Read-only exact ascending integer coefficients; construction remains provider-private.
#[derive(Clone, Copy)]
pub struct ExactPairPolynomial<'a> {
    coefficients: &'a [BigInt],
}
/// Read-only signed integer coefficient; decimal formatting is exact, never binary64 conversion.
pub struct ExactPairCoefficient<'a> {
    coefficient: &'a BigInt,
}
impl<'a> ExactPairPolynomial<'a> {
    pub(super) fn new(coefficients: &'a [BigInt]) -> Self {
        let retained = coefficients;
        // No normalization or coefficient copy occurs in this evidence view.
        Self {
            coefficients: retained,
        }
    }
    /// Read the exact reduced degree; `None` denotes zero. Example: `proof.graph().unwrap().candidates().0.polynomial_degree()`.
    pub fn polynomial_degree(&self) -> Option<usize> {
        let coefficients = self.coefficients;
        coefficients.len().checked_sub(1)
    }
    /// Read the number of retained coefficients. Example: `proof.graph().unwrap().projection().alpha.coefficient_count()`.
    pub fn coefficient_count(&self) -> usize {
        let coefficients = self.coefficients;
        coefficients.len()
    }
    /// Borrow an exact coefficient without replacing it with a rounded scalar. Example: `proof.graph().unwrap().projection().alpha.signed_coefficient(0).unwrap().to_string()`.
    pub fn signed_coefficient(&self, index: usize) -> Option<ExactPairCoefficient<'a>> {
        let coefficient = self.coefficients.get(index)?;
        Some(ExactPairCoefficient { coefficient })
    }
}
impl std::fmt::Display for ExactPairCoefficient<'_> {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let coefficient = self.coefficient;
        coefficient.fmt(output)
    }
}
