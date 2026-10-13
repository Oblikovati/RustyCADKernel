// SPDX-License-Identifier: GPL-2.0-only
//! Sturm–Tarski sign determination without approximate root evaluation (ADR 0007).
mod error;
mod tarski;
use super::{RootIsolation, RootIsolationOptions, chain, isolate, polynomial::IntegerPolynomial};
pub use error::{RootSignError, RootSignStage};
use std::cmp::Ordering;

/// A complete source isolation and aligned query signs; inputs are not retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootSignEvaluation {
    isolation: RootIsolation,
    signs: Vec<Ordering>,
}
impl RootSignEvaluation {
    /// Borrow unchanged root brackets and multiplicities; example: `result.isolation().roots()`.
    pub fn isolation(&self) -> &RootIsolation {
        &self.isolation
    }
    /// Read query signs in source-root order: Less is negative, Equal zero, Greater positive.
    /// Example: `result.signs()[0] == std::cmp::Ordering::Equal` certifies a common root.
    pub fn signs(&self) -> &[Ordering] {
        &self.signs
    }
}
/// Determine the exact sign of an ascending query polynomial at every source root.
///
/// Both inputs are exact supplied binary64 coefficients. A zero query is valid;
/// a zero source is not. No bracket midpoint is used as a root approximation.
/// ```
/// use rustycad_tkmath::predicates::roots::{RootIsolationOptions, signs::*};
/// use std::cmp::Ordering::{Less, Greater};
/// let result = signs_at_polynomial_roots(&[-2., 0., 1.], &[0., 1.], [-2., 2.],
///     RootIsolationOptions::new(0.01), || false)?;
/// assert_eq!(result.signs(), &[Less, Greater]);
/// # Ok::<(), RootSignError>(())
/// ```
pub fn signs_at_polynomial_roots(
    source: &[f64],
    query: &[f64],
    interval: [f64; 2],
    options: RootIsolationOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<RootSignEvaluation, RootSignError> {
    let (bounds, width) = super::checked_policy_interval(interval, options)
        .map_err(|cause| RootSignStage::Policy.failure(cause))?;
    let source = IntegerPolynomial::from_binary64(source, options.max_degree)
        .map_err(|cause| RootSignStage::Source.failure(cause))?;
    let query = query_polynomial(query, options.max_degree)?;
    let tower = chain::multiplicity_tower(source.clone(), &mut cancelled)
        .map_err(|cause| RootSignStage::Source.failure(cause))?;
    let isolation = isolate::isolate(&tower, bounds, width, options, &mut cancelled)
        .map_err(|cause| RootSignStage::Source.failure(cause))?;
    evaluate_isolated(source, query, isolation, &mut cancelled)
}
pub(super) fn evaluate_isolated(
    source: IntegerPolynomial,
    query: IntegerPolynomial,
    isolation: RootIsolation,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootSignEvaluation, RootSignError> {
    let signs = tarski::evaluate(source, query, &isolation, cancelled)?;
    Ok(RootSignEvaluation { isolation, signs })
}
fn query_polynomial(coefficients: &[f64], limit: u16) -> Result<IntegerPolynomial, RootSignError> {
    IntegerPolynomial::accept_zero(IntegerPolynomial::from_binary64(coefficients, limit))
        .map_err(|cause| RootSignStage::Query.failure(cause))
}
