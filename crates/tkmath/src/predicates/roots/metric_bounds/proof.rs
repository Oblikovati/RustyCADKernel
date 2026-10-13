// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    DyadicBound, RootIsolationOptions, chain, error::poll, isolate, polynomial::IntegerPolynomial,
    signs,
};
use super::{ResidualDecision, ResidualError, ResidualViolation};
use num_bigint::{BigInt, Sign};
use std::cmp::Ordering;
pub(super) struct MetricProofDomain {
    pub bounds: [DyadicBound; 2],
    pub width: DyadicBound,
    pub options: RootIsolationOptions,
    pub upper: DyadicBound,
}
pub(super) fn decide(
    coefficients: Vec<BigInt>,
    domain: MetricProofDomain,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ResidualDecision, ResidualError> {
    decide_accounted(coefficients, domain, cancelled, &mut || {})
}
pub(super) fn decide_accounted(
    coefficients: Vec<BigInt>,
    domain: MetricProofDomain,
    cancelled: &mut impl FnMut() -> bool,
    completed_split: &mut impl FnMut(),
) -> Result<ResidualDecision, ResidualError> {
    poll(cancelled)?;
    let polynomial = IntegerPolynomial::accept_zero(IntegerPolynomial::from_integers(
        coefficients,
        domain.options.max_degree,
    ))?;
    if let Some(witness) = endpoint_violation(&polynomial, &domain.bounds) {
        return Ok(ResidualDecision::Exceeds { witness });
    }
    if polynomial.derivative().0.is_empty() || domain.bounds[0] == domain.bounds[1] {
        return Ok(ResidualDecision::Within {
            proven_squared_upper: domain.upper,
        });
    }
    stationary_decision(polynomial, domain, cancelled, completed_split)
}
fn endpoint_violation(
    polynomial: &IntegerPolynomial,
    bounds: &[DyadicBound; 2],
) -> Option<ResidualViolation> {
    bounds
        .iter()
        .find(|endpoint| polynomial.sign_at(endpoint) == Sign::Minus)
        .map(|endpoint| ResidualViolation::Endpoint(endpoint.clone()))
}
fn stationary_decision(
    polynomial: IntegerPolynomial,
    domain: MetricProofDomain,
    cancelled: &mut impl FnMut() -> bool,
    completed_split: &mut impl FnMut(),
) -> Result<ResidualDecision, ResidualError> {
    let (evaluated, upper) = stationary_evaluation(polynomial, domain, cancelled, completed_split)?;
    if let Some(witness) = stationary_violation(&evaluated) {
        return Ok(ResidualDecision::Exceeds { witness });
    }
    Ok(ResidualDecision::Within {
        proven_squared_upper: upper,
    })
}
fn stationary_evaluation(
    polynomial: IntegerPolynomial,
    domain: MetricProofDomain,
    cancelled: &mut impl FnMut() -> bool,
    completed_split: &mut impl FnMut(),
) -> Result<(signs::RootSignEvaluation, DyadicBound), ResidualError> {
    let derivative = polynomial.derivative();
    let tower = chain::multiplicity_tower(derivative.clone(), cancelled)?;
    let isolation = isolate::isolate_accounted(
        &tower,
        domain.bounds,
        domain.width,
        domain.options,
        cancelled,
        completed_split,
    )?;
    let evaluated = signs::evaluate_isolated(derivative, polynomial, isolation, cancelled)?;
    Ok((evaluated, domain.upper))
}
fn stationary_violation(evaluated: &signs::RootSignEvaluation) -> Option<ResidualViolation> {
    evaluated
        .isolation()
        .roots()
        .iter()
        .zip(evaluated.signs())
        .find(|(_, sign)| **sign == Ordering::Less)
        .map(|(root, _)| ResidualViolation::Stationary(root.clone()))
}
