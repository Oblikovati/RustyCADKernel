// SPDX-License-Identifier: GPL-2.0-only
//! Pseudo-remainders retain leading factors: their zero fibers need original-system checks.
use super::super::{RootIsolationError, error::poll};
use super::{equations::BivariateEquation, polynomial::ExactCoefficientPolynomial};

pub(super) struct FiberRowReduction {
    pub linear: Option<BivariateEquation>,
    pub leading_factors: Vec<ExactCoefficientPolynomial>,
}
pub(super) fn linear_row(
    system: &[BivariateEquation; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<FiberRowReduction, RootIsolationError> {
    let mut rows = system.clone();
    let mut leading_factors = Vec::new();
    poll(cancelled)?;
    while rows
        .iter()
        .all(|row| row.degree_u().is_some_and(|degree| degree > 1))
    {
        reduce_rows(&mut rows, &mut leading_factors, cancelled)?;
        poll(cancelled)?;
    }
    let linear = rows.iter().find(|row| row.degree_u() == Some(1)).cloned();
    Ok(FiberRowReduction {
        linear,
        leading_factors,
    })
}
fn reduce_rows(
    rows: &mut [BivariateEquation; 2],
    factors: &mut Vec<ExactCoefficientPolynomial>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), RootIsolationError> {
    if rows[0].degree_u() < rows[1].degree_u() {
        rows.swap(0, 1);
    }
    rows[0] = pseudo_remainder(&rows[0], &rows[1], factors, cancelled)?;
    rows.swap(0, 1);
    Ok(())
}
fn pseudo_remainder(
    dividend: &BivariateEquation,
    divisor: &BivariateEquation,
    factors: &mut Vec<ExactCoefficientPolynomial>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<BivariateEquation, RootIsolationError> {
    let degree = divisor
        .degree_u()
        .expect("nonzero divisor established by row reduction");
    let mut remainder = dividend.clone();
    while remainder.degree_u().is_some_and(|power| power >= degree) {
        poll(cancelled)?;
        factors.push(divisor.0[degree].clone());
        remainder = eliminate_leading(&remainder, divisor, degree, cancelled)?;
    }
    Ok(remainder)
}
fn eliminate_leading(
    remainder: &BivariateEquation,
    divisor: &BivariateEquation,
    degree: usize,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<BivariateEquation, RootIsolationError> {
    let power = remainder.0.len() - 1;
    let lead = &remainder.0[power];
    let offset = power - degree;
    let mut coefficients = Vec::with_capacity(remainder.0.len());
    for (index, coefficient) in remainder.0.iter().enumerate() {
        let scaled = coefficient.multiply(&divisor.0[degree], cancelled)?;
        let term = shifted_divisor_term(divisor, index, offset, lead, cancelled)?;
        coefficients.push(scaled.subtract(&term));
    }
    Ok(BivariateEquation::new(coefficients))
}
fn shifted_divisor_term(
    divisor: &BivariateEquation,
    index: usize,
    offset: usize,
    factor: &ExactCoefficientPolynomial,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ExactCoefficientPolynomial, RootIsolationError> {
    let Some(power) = index
        .checked_sub(offset)
        .and_then(|power| divisor.0.get(power))
    else {
        return Ok(ExactCoefficientPolynomial::new(Vec::new()));
    };
    power.multiply(factor, cancelled)
}
