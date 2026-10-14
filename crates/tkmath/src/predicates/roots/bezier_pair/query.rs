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
pub(super) struct CapturedBezierPair {
    sources: [OwnedBezierSource; 2],
    domain: CapturedPairDomain,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PairQueryPolicy {
    pub version: u32,
    pub roots: RootIsolationOptions,
    pub signs: PairSignOptions,
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
    pub fn distinct(left: OwnedBezierSource, right: OwnedBezierSource) -> Self {
        Self {
            sources: [left, right],
            domain: CapturedPairDomain::Distinct,
        }
    }
    pub fn self_off_diagonal(
        controls: &[[f64; 2]],
        weights: Option<&[f64]>,
        interval: [f64; 2],
    ) -> Result<Self, PairSourceFailure> {
        let source = OwnedBezierSource::capture(controls, weights, interval)?;
        Ok(Self {
            sources: [source.clone(), source],
            domain: CapturedPairDomain::SelfOffDiagonal,
        })
    }
    pub fn sources(&self) -> &[OwnedBezierSource; 2] {
        let captured = &self.sources;
        // This returns the query's captured allocation, not caller-owned mutable controls.
        captured
    }
    pub fn evaluate(
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
