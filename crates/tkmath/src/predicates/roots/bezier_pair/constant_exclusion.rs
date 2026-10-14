// SPDX-License-Identifier: GPL-2.0-only
//! A nonzero constant in either checked original equation excludes every common root.
use super::equations::PairedEquations;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ConstantEquationExclusion {
    pub equations: PairedEquations,
    pub axis: usize,
    pub divided_self_diagonal: bool,
}
impl ConstantEquationExclusion {
    pub fn from_checked_equations(
        equations: &PairedEquations,
        divided_self_diagonal: bool,
    ) -> Option<Self> {
        let axis = equations.0.iter().position(nonzero_constant_equation)?;
        Some(Self {
            equations: equations.clone(),
            axis,
            divided_self_diagonal,
        })
    }
}
fn nonzero_constant_equation(equation: &super::equations::BivariateEquation) -> bool {
    let powers = &equation.0;
    // Canonical exact-ring zero trimming makes total degree zero a nonzero witness.
    powers.len() == 1 && powers[0].degree() == Some(0)
}

#[cfg(test)]
mod tests {
    use super::super::{equations::BivariateEquation, polynomial::ExactCoefficientPolynomial};
    use super::*;
    fn equation(powers: &[&[i32]]) -> BivariateEquation {
        let coefficients = powers
            .iter()
            .map(|power| {
                ExactCoefficientPolynomial::new(power.iter().map(|value| (*value).into()).collect())
            })
            .collect();
        BivariateEquation::new(coefficients)
    }
    #[test]
    fn exact_nonzero_constant_excludes_arbitrary_other_equation() {
        let system = PairedEquations([equation(&[&[-7]]), equation(&[&[4, 8], &[0], &[9]])]);
        let proof = ConstantEquationExclusion::from_checked_equations(&system, false).unwrap();
        assert_eq!(proof.axis, 0);
        assert_eq!(proof.equations, system);
        assert!(!proof.divided_self_diagonal);
    }
    #[test]
    fn zero_first_axis_preserves_second_axis_constant_and_self_scope() {
        let system = PairedEquations([equation(&[&[0]]), equation(&[&[11]])]);
        let proof = ConstantEquationExclusion::from_checked_equations(&system, true).unwrap();
        assert_eq!(proof.axis, 1);
        assert_eq!(proof.equations.0[1].0[0].0, vec![11.into()]);
        assert!(proof.divided_self_diagonal);
    }
    #[test]
    fn zero_and_nonconstant_equations_cannot_forge_whole_domain_exclusion() {
        for other in [equation(&[]), equation(&[&[0, 1]]), equation(&[&[0], &[1]])] {
            let system = PairedEquations([equation(&[]), other]);
            assert!(ConstantEquationExclusion::from_checked_equations(&system, false).is_none());
        }
    }
    #[test]
    fn old_projection_failure_and_cancel_poll_precede_new_constant_evidence() {
        let system = PairedEquations([equation(&[&[3]]), equation(&[&[4], &[0], &[9]])]);
        let mut polls = 0;
        let old = super::super::projection::LinearFiberProjection::derive(&system, &mut || {
            polls += 1;
            false
        })
        .unwrap();
        assert!(old.is_none());
        assert_eq!(polls, 1);
        assert!(ConstantEquationExclusion::from_checked_equations(&system, false).is_some());
        assert!(matches!(
            super::super::projection::LinearFiberProjection::derive(&system, &mut || true),
            Err(super::super::super::RootIsolationError::Cancelled)
        ));
    }
}
