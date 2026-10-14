// SPDX-License-Identifier: GPL-2.0-only
//! One isolated root of an exact integer equation. Every comparison is exact:
//! brackets are only ever split at exact dyadic points, rational queries are
//! substituted exactly, and equality of two roots is proved by a common factor.
use super::super::certificate_arithmetic::ExactRatio;
use super::super::chain::{SturmChain, root_count};
use super::super::{DyadicBound, RootBracket, polynomial::IntegerPolynomial};
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use num_bigint::{BigInt, Sign};
use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;

/// A nonzero integer equation with its Sturm chain, shared by all of its roots.
pub(super) struct RootEquation {
    pub polynomial: IntegerPolynomial,
    chain: SturmChain,
}
impl fmt::Debug for RootEquation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "RootEquation({:?})", self.polynomial.0)
    }
}
impl RootEquation {
    pub fn new(polynomial: IntegerPolynomial, work: &mut Work<'_>) -> Result<Arc<Self>, Cause> {
        let chain = SturmChain::new(polynomial.clone(), &mut || work.poll())?;
        Ok(Arc::new(Self { polynomial, chain }))
    }
}
/// A closed bracket containing exactly one distinct root; nonexact ends are never roots.
#[derive(Clone, Debug)]
pub(super) struct IsolatedRoot {
    pub equation: Arc<RootEquation>,
    pub lower: DyadicBound,
    pub upper: DyadicBound,
}
impl PartialEq for IsolatedRoot {
    fn eq(&self, other: &Self) -> bool {
        self.lower == other.lower
            && self.upper == other.upper
            && self.equation.polynomial.0 == other.equation.polynomial.0
    }
}
impl IsolatedRoot {
    pub fn from_bracket(equation: &Arc<RootEquation>, bracket: &RootBracket) -> Self {
        let (lower, upper) = bracket.bounds();
        Self {
            equation: Arc::clone(equation),
            lower: lower.clone(),
            upper: upper.clone(),
        }
    }
    pub fn exact(&self) -> Option<&DyadicBound> {
        (self.lower == self.upper).then_some(&self.lower)
    }
    /// Halve the bracket exactly; one work unit.
    pub fn bisect(&mut self, work: &mut Work<'_>) -> Result<(), Cause> {
        if self.exact().is_some() {
            return Ok(());
        }
        let middle = self.lower.midpoint(&self.upper);
        self.split_at(&middle, work).map(|_| ())
    }
    /// Exact `root.cmp(value)`, narrowing the bracket to the side that holds the root.
    pub fn split_at(
        &mut self,
        value: &DyadicBound,
        work: &mut Work<'_>,
    ) -> Result<Ordering, Cause> {
        if let Some(root) = self.exact() {
            return Ok(root.cmp(value));
        }
        if *value <= self.lower {
            return Ok(Ordering::Greater);
        }
        if *value >= self.upper {
            return Ok(Ordering::Less);
        }
        work.charge()?;
        self.narrow(value, work)
    }
    fn narrow(&mut self, value: &DyadicBound, work: &mut Work<'_>) -> Result<Ordering, Cause> {
        let chain = &self.equation.chain;
        let middle = chain.endpoint(value.clone(), &mut || work.poll())?;
        if middle.root {
            self.lower = value.clone();
            self.upper = value.clone();
            return Ok(Ordering::Equal);
        }
        let low = chain.endpoint(self.lower.clone(), &mut || work.poll())?;
        match root_count(&low, &middle)? {
            1 => self.upper = value.clone(),
            0 => self.lower = value.clone(),
            _ => return Err(Cause::Invariant("isolating bracket held several roots")),
        }
        Ok(if self.upper == *value {
            Ordering::Less
        } else {
            Ordering::Greater
        })
    }
    /// Exact `root.cmp(value)` for a rational value that need not be dyadic.
    pub fn compare_ratio(
        &mut self,
        value: &ExactRatio,
        work: &mut Work<'_>,
    ) -> Result<Ordering, Cause> {
        loop {
            if let Some(root) = self.exact() {
                return Ok(ExactRatio::from_dyadic(root).cmp(value));
            }
            if *value <= ExactRatio::from_dyadic(&self.lower) {
                return Ok(Ordering::Greater);
            }
            if *value >= ExactRatio::from_dyadic(&self.upper) {
                return Ok(Ordering::Less);
            }
            if sign_at_ratio(&self.equation.polynomial, value) == Sign::NoSign {
                return Ok(Ordering::Equal);
            }
            self.bisect(work)?;
        }
    }
    /// Lower and upper bounds as exact rationals.
    pub fn ratio_bounds(&self) -> [ExactRatio; 2] {
        [
            ExactRatio::from_dyadic(&self.lower),
            ExactRatio::from_dyadic(&self.upper),
        ]
    }
}
/// Exact order of two isolated roots of possibly different equations.
pub(super) fn compare_roots(
    first: &mut IsolatedRoot,
    second: &mut IsolatedRoot,
    work: &mut Work<'_>,
) -> Result<Ordering, Cause> {
    let mut common_checked = false;
    loop {
        if let Some(order) = separated_order(first, second, work)? {
            return Ok(order);
        }
        if !common_checked {
            common_checked = true;
            if share_root_in_overlap(first, second, work)? {
                return Ok(Ordering::Equal);
            }
        }
        first.bisect(work)?;
        second.bisect(work)?;
    }
}
fn separated_order(
    first: &mut IsolatedRoot,
    second: &mut IsolatedRoot,
    work: &mut Work<'_>,
) -> Result<Option<Ordering>, Cause> {
    match (first.exact().cloned(), second.exact().cloned()) {
        (Some(left), Some(right)) => return Ok(Some(left.cmp(&right))),
        (Some(left), None) => return second.split_at(&left, work).map(|o| Some(o.reverse())),
        (None, Some(right)) => return first.split_at(&right, work).map(Some),
        (None, None) => {}
    }
    if first.upper <= second.lower {
        return Ok(Some(Ordering::Less));
    }
    if second.upper <= first.lower {
        return Ok(Some(Ordering::Greater));
    }
    Ok(None)
}
// Each bracket isolates one root of its own equation. A root of the common factor
// inside both brackets is therefore both roots; its absence proves inequality.
fn share_root_in_overlap(
    first: &IsolatedRoot,
    second: &IsolatedRoot,
    work: &mut Work<'_>,
) -> Result<bool, Cause> {
    let pair = SturmChain::from_pair(
        first.equation.polynomial.clone(),
        second.equation.polynomial.clone(),
        &mut || work.poll(),
    )?;
    let Some(common) = pair.repeated_factor() else {
        return Ok(false);
    };
    let lower = first.lower.clone().max(second.lower.clone());
    let upper = first.upper.clone().min(second.upper.clone());
    has_root_in_closed(common, lower, upper, work)
}
fn has_root_in_closed(
    polynomial: IntegerPolynomial,
    lower: DyadicBound,
    upper: DyadicBound,
    work: &mut Work<'_>,
) -> Result<bool, Cause> {
    let chain = SturmChain::new(polynomial, &mut || work.poll())?;
    let low = chain.endpoint(lower, &mut || work.poll())?;
    let high = chain.endpoint(upper, &mut || work.poll())?;
    Ok(low.root || root_count(&low, &high)? > 0)
}
/// Exact sign of an integer polynomial at a rational point (positive denominator).
pub(super) fn sign_at_ratio(polynomial: &IntegerPolynomial, value: &ExactRatio) -> Sign {
    let numerator = value.numerator();
    let denominator = value.denominator();
    let mut total = BigInt::from(0);
    let mut power = BigInt::from(1);
    for coefficient in polynomial.0.iter().rev() {
        total = total * numerator + coefficient * &power;
        power *= denominator;
    }
    total.sign()
}
/// Sign immediately left of an exact point: the first nonzero derivative, with parity.
pub(super) fn left_sign(polynomial: &IntegerPolynomial, point: &DyadicBound) -> Sign {
    let mut derivative = polynomial.clone();
    let mut order = 0_usize;
    while !derivative.0.is_empty() {
        let sign = derivative.sign_at(point);
        if sign != Sign::NoSign {
            return if order.is_multiple_of(2) { sign } else { -sign };
        }
        derivative = derivative.derivative();
        order += 1;
    }
    Sign::NoSign
}
