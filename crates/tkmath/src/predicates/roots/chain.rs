// SPDX-License-Identifier: GPL-2.0-only
use super::{DyadicBound, RootIsolationError, error::poll, polynomial::IntegerPolynomial};
use num_bigint::Sign;

pub(super) struct SturmChain(Vec<IntegerPolynomial>);
#[derive(Clone)]
pub(super) struct Endpoint {
    pub bound: DyadicBound,
    pub variation: usize,
    pub root: bool,
}
impl SturmChain {
    pub fn new(
        polynomial: IntegerPolynomial,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self, RootIsolationError> {
        poll(cancelled)?;
        let derivative = polynomial.derivative();
        Self::from_pair(polynomial, derivative, cancelled)
    }
    pub fn from_pair(
        polynomial: IntegerPolynomial,
        second: IntegerPolynomial,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self, RootIsolationError> {
        if second.0.is_empty() {
            return Ok(Self(vec![polynomial]));
        }
        let mut members = vec![polynomial, second];
        while members.last().is_some_and(|last| last.degree() > 0) {
            let last = members.len() - 1;
            let remainder = members[last - 1].negative_remainder(&members[last], cancelled)?;
            if remainder.0.is_empty() {
                break;
            }
            members.push(remainder);
        }
        Ok(Self(members))
    }
    pub fn endpoint(
        &self,
        bound: DyadicBound,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Endpoint, RootIsolationError> {
        poll(cancelled)?;
        let (root, mut previous) = self.0[0].endpoint_signs(&bound);
        let mut variation = 0;
        for polynomial in &self.0[1..] {
            poll(cancelled)?;
            let (_, right) = polynomial.endpoint_signs(&bound);
            variation += usize::from(previous != Sign::NoSign && right != previous);
            previous = right;
        }
        Ok(Endpoint {
            bound,
            variation,
            root,
        })
    }
    pub fn repeated_factor(&self) -> Option<IntegerPolynomial> {
        self.0.last().filter(|last| last.degree() > 0).cloned()
    }
}
pub(super) fn multiplicity_tower(
    polynomial: IntegerPolynomial,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<SturmChain>, RootIsolationError> {
    let mut tower = vec![SturmChain::new(polynomial, cancelled)?];
    while let Some(factor) = tower.last().and_then(SturmChain::repeated_factor) {
        tower.push(SturmChain::new(factor, cancelled)?);
    }
    Ok(tower)
}
pub(super) fn root_count(lower: &Endpoint, upper: &Endpoint) -> Result<usize, RootIsolationError> {
    lower
        .variation
        .checked_sub(upper.variation)
        .ok_or(RootIsolationError::Variation {
            lower: lower.variation,
            upper: upper.variation,
        })
}
