// SPDX-License-Identifier: GPL-2.0-only
//! A linear algebraic fiber is a general elimination condition, not a curve-kind recognizer.
use super::super::{RootIsolationError, error::poll};
use super::{
    equations::{BivariateEquation, PairedEquations},
    polynomial::ExactCoefficientPolynomial,
};
use num_bigint::BigInt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LinearFiberProjection {
    pub alpha: ExactCoefficientPolynomial,
    pub beta: ExactCoefficientPolynomial,
    pub cleared_originals: [ExactCoefficientPolynomial; 2],
    pub leading_factors: Vec<ExactCoefficientPolynomial>,
}
impl LinearFiberProjection {
    pub fn derive(
        system: &PairedEquations,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Option<Self>, RootIsolationError> {
        let reduction = super::fiber_row::linear_row(&system.0, cancelled)?;
        let Some(row) = reduction.linear else {
            return Ok(None);
        };
        Self::from_row(system, row, reduction.leading_factors, cancelled).map(Some)
    }
    fn from_row(
        system: &PairedEquations,
        row: BivariateEquation,
        leading_factors: Vec<ExactCoefficientPolynomial>,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self, RootIsolationError> {
        let alpha = row.0[1].clone();
        let beta = row.0[0].clone();
        let x = substitute_linear_fiber(&system.0[0], &alpha, &beta, cancelled)?;
        let y = substitute_linear_fiber(&system.0[1], &alpha, &beta, cancelled)?;
        Ok(Self {
            alpha,
            beta,
            cleared_originals: [x, y],
            leading_factors,
        })
    }
}
fn substitute_linear_fiber(
    equation: &BivariateEquation,
    alpha: &ExactCoefficientPolynomial,
    beta: &ExactCoefficientPolynomial,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ExactCoefficientPolynomial, RootIsolationError> {
    let Some(numerator) = equation.0.last().cloned() else {
        return Ok(ExactCoefficientPolynomial::new(Vec::new()));
    };
    let mut horner = FiberHorner::new(alpha, beta, numerator);
    for coefficient in equation.0[..equation.0.len() - 1].iter().rev() {
        horner.advance(alpha, coefficient, cancelled)?;
    }
    Ok(horner.numerator)
}
struct FiberHorner {
    numerator: ExactCoefficientPolynomial,
    minus_beta: ExactCoefficientPolynomial,
    alpha_power: ExactCoefficientPolynomial,
}
impl FiberHorner {
    fn new(
        alpha: &ExactCoefficientPolynomial,
        beta: &ExactCoefficientPolynomial,
        numerator: ExactCoefficientPolynomial,
    ) -> Self {
        Self {
            numerator,
            minus_beta: beta.scaled(&BigInt::from(-1)),
            alpha_power: alpha.clone(),
        }
    }
    fn advance(
        &mut self,
        alpha: &ExactCoefficientPolynomial,
        coefficient: &ExactCoefficientPolynomial,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), RootIsolationError> {
        poll(cancelled)?;
        let previous = self.numerator.multiply(&self.minus_beta, cancelled)?;
        let term = coefficient.multiply(&self.alpha_power, cancelled)?;
        self.numerator = previous.add(&term);
        self.alpha_power = self.alpha_power.multiply(alpha, cancelled)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::super::source::OwnedBezierSource;
    use super::*;
    fn source(controls: &[[f64; 2]]) -> super::super::source::HomogeneousPolynomials {
        OwnedBezierSource::capture(controls, None, [0., 1.])
            .unwrap()
            .homogeneous(&mut || false)
            .unwrap()
    }
    fn pair(a: &[[f64; 2]], b: &[[f64; 2]]) -> PairedEquations {
        PairedEquations::derive(&source(a), &source(b), &mut || false).unwrap()
    }
    #[test]
    fn actual_curved_pair_projects_only_after_source_derived_linear_correspondence() {
        let system = pair(
            &[[0., 0.], [0.5, 0.], [1., 1.]],
            &[[0., 0.5], [0.5, 0.5], [1., -0.5]],
        );
        let projection = LinearFiberProjection::derive(&system, &mut || false)
            .unwrap()
            .unwrap();
        verify_parabola_projection(&projection);
        let identity = substitute_linear_fiber(
            &system.0[0],
            &projection.alpha,
            &projection.beta,
            &mut || false,
        )
        .unwrap();
        assert!(identity.0.is_empty());
    }
    fn verify_parabola_projection(projection: &LinearFiberProjection) {
        assert_eq!(projection.alpha.0, vec![(-2).into()]);
        assert_eq!(projection.beta.0, vec![0.into(), 2.into()]);
        assert_eq!(
            projection.cleared_originals[1].primitive(2).unwrap().0,
            vec![(-1).into(), 0.into(), 4.into()]
        );
    }
    #[test]
    fn self_loop_and_cusp_use_same_correspondence_with_distinct_projection_multiplicity() {
        verify_self_projection(
            &[[3., -9.], [-1., 13.], [-1., -13.], [3., 9.]],
            &[3, -16, 16],
        );
        verify_self_projection(&[[3., -3.], [-1., 3.], [-1., -3.], [3., 3.]], &[1, -4, 4]);
    }
    fn verify_self_projection(controls: &[[f64; 2]], expected: &[i32]) {
        let system = pair(controls, controls)
            .exclude_diagonal(&mut || false)
            .unwrap();
        let projection = LinearFiberProjection::derive(&system, &mut || false)
            .unwrap()
            .unwrap();
        let expected: Vec<BigInt> = expected.iter().map(|value| BigInt::from(*value)).collect();
        assert_eq!(
            projection.cleared_originals[1].primitive(2).unwrap().0,
            expected
        );
        assert_eq!(projection.alpha.0, vec![12.into()]);
        assert_eq!(projection.beta.0, vec![(-12).into(), 12.into()]);
    }
    #[test]
    fn closed_cubic_uses_generic_row_elimination_and_checks_both_originals() {
        let controls = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
        let system = pair(&controls, &controls)
            .exclude_diagonal(&mut || false)
            .unwrap();
        assert_eq!(system.0[0].degree_u(), Some(2));
        assert_eq!(system.0[1].degree_u(), Some(2));
        let projection = LinearFiberProjection::derive(&system, &mut || false)
            .unwrap()
            .unwrap();
        assert_eq!(projection.leading_factors.len(), 1);
        for (original, expected) in projection.cleared_originals.iter().zip([
            vec![0.into(), (-1).into(), 1.into()],
            vec![0.into(), 1.into(), (-1).into()],
        ]) {
            assert_eq!(original.primitive(2).unwrap().0, expected);
        }
        assert_eq!(projection.alpha.0, vec![144.into()]);
        assert_eq!(projection.beta.0, vec![(-144).into(), 144.into()]);
    }
    #[test]
    fn zero_projection_is_not_empty_intersection_and_cancellation_stays_typed() {
        let controls = [[0., 0.], [0.5, 0.], [1., 1.]];
        let system = pair(&controls, &controls);
        let projected = LinearFiberProjection::derive(&system, &mut || false)
            .unwrap()
            .unwrap();
        assert!(matches!(
            projected.cleared_originals[1].primitive(32),
            Err(RootIsolationError::ZeroPolynomial { .. })
        ));
        assert_eq!(
            LinearFiberProjection::derive(&system, &mut || true),
            Err(RootIsolationError::Cancelled)
        );
    }
}
