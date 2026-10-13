// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    RootBracket, RootIsolation, chain::SturmChain, error::poll, polynomial::IntegerPolynomial,
};
use super::{RootSignError, RootSignStage};
use num_bigint::Sign;
use std::cmp::Ordering;

pub(super) fn evaluate(
    source: IntegerPolynomial,
    query: IntegerPolynomial,
    isolation: &RootIsolation,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<Ordering>, RootSignError> {
    let chain = query_chain(source, &query, cancelled)
        .map_err(|cause| RootSignStage::Evaluation.failure(cause))?;
    isolation
        .roots()
        .iter()
        .map(|root| classify(&chain, &query, root, cancelled))
        .collect()
}
fn query_chain(
    source: IntegerPolynomial,
    query: &IntegerPolynomial,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<SturmChain, super::super::RootIsolationError> {
    poll(cancelled)?;
    let numerator = source.derivative().multiply(query, cancelled)?;
    // ADR 0007: removing the polynomial part preserves the Cauchy index.
    let remainder = numerator.negative_remainder(&source, cancelled)?.negated();
    SturmChain::from_pair(source, remainder, cancelled)
}
fn classify(
    chain: &SturmChain,
    query: &IntegerPolynomial,
    root: &RootBracket,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Ordering, RootSignError> {
    poll(cancelled).map_err(|cause| RootSignStage::Evaluation.failure(cause))?;
    let (lower, upper) = root.bounds();
    if lower == upper {
        return Ok(sign_order(query.sign_at(lower)));
    }
    let first = chain
        .endpoint(lower.clone(), cancelled)
        .map_err(|cause| RootSignStage::Evaluation.failure(cause))?;
    let last = chain
        .endpoint(upper.clone(), cancelled)
        .map_err(|cause| RootSignStage::Evaluation.failure(cause))?;
    certified_difference(first.variation, last.variation)
}
fn sign_order(sign: Sign) -> Ordering {
    match sign {
        Sign::Minus => Ordering::Less,
        Sign::NoSign => Ordering::Equal,
        Sign::Plus => Ordering::Greater,
    }
}
fn certified_difference(lower: usize, upper: usize) -> Result<Ordering, RootSignError> {
    if lower == upper {
        return Ok(Ordering::Equal);
    }
    if lower.checked_sub(upper) == Some(1) {
        return Ok(Ordering::Greater);
    }
    if upper.checked_sub(lower) == Some(1) {
        return Ok(Ordering::Less);
    }
    Err(RootSignError::Variation { lower, upper })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_tarski_sums_are_declines_in_both_directions() {
        for (lower, upper) in [(2, 0), (0, 2), (usize::MAX, 0)] {
            assert_eq!(
                certified_difference(lower, upper),
                Err(RootSignError::Variation { lower, upper })
            );
        }
    }
}
