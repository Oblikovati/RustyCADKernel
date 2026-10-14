// SPDX-License-Identifier: GPL-2.0-only
//! The query owns coefficients and domain; self mode captures one genuine source twice.
use super::super::RootIsolationOptions;
use super::{
    operation::{self, AccountedPairOutcome},
    sign_policy::PairSignOptions,
    source::{OwnedBezierSource, PairSourceFailure},
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CapturedPairDomain {
    Distinct,
    SelfOffDiagonal,
}
#[derive(Clone, Debug, PartialEq)]
/// Immutable captured distinct curves or one source for canonical self contacts.
pub struct CapturedBezierPair {
    sources: [OwnedBezierSource; 2],
    domain: CapturedPairDomain,
}
#[derive(Clone, Copy, Debug, PartialEq)]
/// Versioned primary root, generated-sign and independent world-error policy.
pub struct PairQueryPolicy {
    /// Query policy version; expected one.
    pub version: u32,
    /// Primary polynomial admission, initial bracket width and global split limit.
    pub roots: RootIsolationOptions,
    /// Separate versioned admission for generated exact sign polynomials.
    pub signs: PairSignOptions,
    /// Finite positive Euclidean construction error; never widened.
    pub requested_world_error: f64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PairQueryPreparationFailure {
    Version(u32),
}
pub(super) struct CapturedPairOutcome {
    pub query: CapturedBezierPair,
    pub policy: PairQueryPolicy,
    pub outcome: Result<AccountedPairOutcome, PairQueryPreparationFailure>,
}
impl CapturedBezierPair {
    pub(super) fn distinct(left: OwnedBezierSource, right: OwnedBezierSource) -> Self {
        Self {
            sources: [left, right],
            domain: CapturedPairDomain::Distinct,
        }
    }
    /// Capture one actual source for canonical t<u contacts, excluding the entire diagonal. Example: `CapturedBezierPair::self_off_diagonal(&controls, None, [0., 1.])?`.
    pub fn self_off_diagonal(
        controls: &[[f64; 2]],
        weights: Option<&[f64]>,
        interval: [f64; 2],
    ) -> Result<Self, PairSourceFailure> {
        let source = OwnedBezierSource::capture(controls, weights, interval)
            .map_err(|cause| super::source::capture_context(0, cause))?;
        Ok(Self {
            sources: [source.clone(), source],
            domain: CapturedPairDomain::SelfOffDiagonal,
        })
    }
    pub(super) fn sources(&self) -> &[OwnedBezierSource; 2] {
        let captured = &self.sources;
        // This returns the query's captured allocation, not caller-owned mutable controls.
        captured
    }
    pub(super) fn evaluate(
        &self,
        policy: PairQueryPolicy,
        cancelled: &mut impl FnMut() -> bool,
    ) -> CapturedPairOutcome {
        let outcome = self.run_policy(policy, cancelled);
        CapturedPairOutcome {
            query: self.clone(),
            policy,
            outcome,
        }
    }
    fn run_policy(
        &self,
        policy: PairQueryPolicy,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<AccountedPairOutcome, PairQueryPreparationFailure> {
        if policy.version != 1 {
            return Err(PairQueryPreparationFailure::Version(policy.version));
        }
        Ok(operation::compute_with_signs(
            self.sources.clone(),
            self.is_self(),
            policy.roots,
            policy.signs,
            policy.requested_world_error,
            cancelled,
        ))
    }
    fn is_self(&self) -> bool {
        let domain = self.domain;
        // Self canonicalization is derived from capture mode, never supplied source identities.
        domain == CapturedPairDomain::SelfOffDiagonal
    }
}
