// SPDX-License-Identifier: GPL-2.0-only
//! Derive both equations from actual homogeneous sources, retaining every coefficient.
use super::super::{RootIsolationError, error::poll};
use super::polynomial::ExactCoefficientPolynomial;
use super::source::HomogeneousPolynomials;
use num_bigint::BigInt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BivariateEquation(pub Vec<ExactCoefficientPolynomial>);
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PairedEquations(pub [BivariateEquation; 2]);
#[derive(Clone, Debug, PartialEq)]
/// Exact self-diagonal division failure; no approximate cancellation is used.
pub enum DiagonalDivisionFailure {
    /// Original coefficient-ring arithmetic or cancellation cause.
    Algebra(RootIsolationError),
    /// The supplied pair was not exactly divisible by the diagonal.
    NonzeroRemainder {
        /// Original coordinate equation index, expected zero or one.
        axis: usize,
        /// Number of remaining nonzero-axis polynomial coefficients.
        coefficient_count: usize,
    },
}
impl PairedEquations {
    pub fn derive(
        left: &HomogeneousPolynomials,
        right: &HomogeneousPolynomials,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self, RootIsolationError> {
        let x = coordinate_equation(left, right, 0, cancelled)?;
        let y = coordinate_equation(left, right, 1, cancelled)?;
        Ok(Self([x, y]))
    }
    pub fn exclude_diagonal(
        &self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self, DiagonalDivisionFailure> {
        let x = self.0[0].divide_diagonal(0, cancelled)?;
        let y = self.0[1].divide_diagonal(1, cancelled)?;
        Ok(Self([x, y]))
    }
}
fn coordinate_equation(
    left: &HomogeneousPolynomials,
    right: &HomogeneousPolynomials,
    axis: usize,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<BivariateEquation, RootIsolationError> {
    let count = right.components[2]
        .0
        .len()
        .max(right.components[axis].0.len());
    let mut powers = Vec::with_capacity(count);
    for power in 0..count {
        poll(cancelled)?;
        let left_part = left.components[axis].scaled(&right.components[2].coefficient(power));
        let right_part = left.components[2].scaled(&right.components[axis].coefficient(power));
        powers.push(left_part.subtract(&right_part));
    }
    Ok(BivariateEquation::new(powers))
}
impl BivariateEquation {
    pub fn new(mut powers: Vec<ExactCoefficientPolynomial>) -> Self {
        while powers.last().is_some_and(|p| p.degree().is_none()) {
            powers.pop();
        }
        Self(powers)
    }
    pub fn degree_u(&self) -> Option<usize> {
        let count = self.0.len();
        count.checked_sub(1)
    }
    fn divide_diagonal(
        &self,
        axis: usize,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self, DiagonalDivisionFailure> {
        let Some(degree) = self.degree_u() else {
            return Ok(Self(Vec::new()));
        };
        let mut quotient = vec![ExactCoefficientPolynomial::new(Vec::new()); degree];
        let remainder = self.diagonal_quotient(&mut quotient, cancelled)?;
        if remainder.degree().is_some() {
            return Err(DiagonalDivisionFailure::NonzeroRemainder {
                axis,
                coefficient_count: remainder.0.len(),
            });
        }
        Ok(Self::new(quotient))
    }
    fn diagonal_quotient(
        &self,
        quotient: &mut [ExactCoefficientPolynomial],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<ExactCoefficientPolynomial, DiagonalDivisionFailure> {
        let degree = quotient.len();
        let Some(last) = quotient.last_mut() else {
            return Ok(self.0[0].clone());
        };
        *last = self.0[degree].scaled(&BigInt::from(-1));
        for power in (1..quotient.len()).rev() {
            poll(cancelled).map_err(DiagonalDivisionFailure::Algebra)?;
            quotient[power - 1] = quotient[power].shifted().subtract(&self.0[power]);
        }
        Ok(self.0[0].subtract(&quotient[0].shifted()))
    }
}
#[cfg(test)]
mod tests {
    use super::super::source::OwnedBezierSource;
    use super::*;
    fn source(controls: &[[f64; 2]]) -> HomogeneousPolynomials {
        OwnedBezierSource::capture(controls, None, [0., 1.])
            .unwrap()
            .homogeneous(&mut || false)
            .unwrap()
    }
    fn polynomial(coefficients: &[i32]) -> ExactCoefficientPolynomial {
        let coefficients = coefficients
            .iter()
            .map(|value| BigInt::from(*value))
            .collect();
        ExactCoefficientPolynomial::new(coefficients)
    }
    #[test]
    fn genuine_curved_pair_equations_are_derived_from_bernstein_coefficients() {
        let a = source(&[[0., 0.], [0.5, 0.], [1., 1.]]);
        let b = source(&[[0., 0.5], [0.5, 0.5], [1., -0.5]]);
        let system = PairedEquations::derive(&a, &b, &mut || false).unwrap();
        assert_eq!(system.0[0].0, vec![polynomial(&[0, 2]), polynomial(&[-2])]);
        assert_eq!(
            system.0[1].0,
            vec![polynomial(&[-1, 0, 2]), polynomial(&[]), polynomial(&[2])]
        );
        assert_eq!(system.0[0].degree_u(), Some(1));
    }
    fn assert_coefficients(actual: &BivariateEquation, expected: &[&[i32]]) {
        let expected: Vec<ExactCoefficientPolynomial> =
            expected.iter().map(|row| polynomial(row)).collect();
        assert_eq!(actual.0, expected);
    }
    #[test]
    fn self_cubic_divides_exact_t_minus_u_factor_without_removing_off_diagonal_roots() {
        let curve = source(&[[3., -9.], [-1., 13.], [-1., -13.], [3., 9.]]);
        let system = PairedEquations::derive(&curve, &curve, &mut || false).unwrap();
        let quotient = system.exclude_diagonal(&mut || false).unwrap();
        assert_coefficients(&quotient.0[0], &[&[-12, 12], &[12]]);
        assert_coefficients(&quotient.0[1], &[&[66, -144, 96], &[-144, 96], &[96]]);
        assert_eq!(
            system.exclude_diagonal(&mut || true),
            Err(DiagonalDivisionFailure::Algebra(
                RootIsolationError::Cancelled
            ))
        );
    }
    #[test]
    fn diagonal_division_rejects_nondivisible_system_instead_of_inventing_zero() {
        let equation = BivariateEquation::new(vec![polynomial(&[1]), polynomial(&[-1])]);
        let system = PairedEquations([equation.clone(), equation]);
        assert!(matches!(
            system.exclude_diagonal(&mut || false),
            Err(DiagonalDivisionFailure::NonzeroRemainder { axis: 0, .. })
        ));
        let zero = PairedEquations([
            BivariateEquation::new(Vec::new()),
            BivariateEquation::new(Vec::new()),
        ]);
        assert_eq!(zero.exclude_diagonal(&mut || false).unwrap(), zero);
    }
}
