// SPDX-License-Identifier: GPL-2.0-only
use super::{
    RootIsolationError,
    error::poll,
    polynomial::{IntegerPolynomial, integer_coefficients},
};
use num_bigint::BigInt;

pub(super) fn from_controls(
    controls: &[f64],
    limit: u16,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<IntegerPolynomial, RootIsolationError> {
    poll(cancelled)?;
    let differences = integer_coefficients(controls)?;
    from_integer_controls(differences, limit, cancelled)
}
pub(super) fn from_integer_controls(
    differences: Vec<BigInt>,
    limit: u16,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<IntegerPolynomial, RootIsolationError> {
    IntegerPolynomial::from_integers(power_coefficients(differences, cancelled)?, limit)
}
fn power_coefficients(
    mut differences: Vec<BigInt>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<BigInt>, RootIsolationError> {
    let degree = differences.len().saturating_sub(1);
    let mut coefficients = Vec::with_capacity(differences.len());
    let mut binomial = BigInt::from(1);
    while let Some(first) = differences.first() {
        poll(cancelled)?;
        let power = coefficients.len();
        // ADR 0005: choose(n,k) * Δ^k b[0], all in the same exact integer scale.
        coefficients.push(first * &binomial);
        forward_difference(&mut differences, cancelled)?;
        binomial = binomial * (degree - power) / (power + 1);
    }
    Ok(coefficients)
}
fn forward_difference(
    differences: &mut Vec<BigInt>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), RootIsolationError> {
    for index in 0..differences.len().saturating_sub(1) {
        poll(cancelled)?;
        differences[index] = &differences[index + 1] - &differences[index];
    }
    differences.pop();
    Ok(())
}
