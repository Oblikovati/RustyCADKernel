// SPDX-License-Identifier: GPL-2.0-only
//! Incomplete computations expose causes and work, never a successful partial contact list.
use super::{
    BezierPairOperationFailure, BezierPairPolicy, BezierPairWork, CapturedBezierPair,
    public_certificate::BezierPairCertificate,
    query::{CapturedPairOutcome, PairQueryPreparationFailure},
};
/// Immutable source-bound outcome; complete empty and failed computations remain distinct.
pub struct BezierPairReport {
    outcome: CapturedPairOutcome,
}
/// The original typed cause or unsupported outer query policy.
#[derive(Debug)]
pub enum BezierPairFailure<'a> {
    /// Rejected policy version; expected version 1 or 2 (ADR 0067).
    PolicyVersion(u32),
    /// Source, exact algebra, domain, construction, cancellation or exhaustion cause.
    Operation(&'a BezierPairOperationFailure),
}
impl BezierPairReport {
    pub(super) fn new(outcome: CapturedPairOutcome) -> Self {
        let retained = outcome;
        // Construction stays private so caller-supplied certificates cannot forge completion.
        Self { outcome: retained }
    }
    /// Read explicit completion without interpreting an empty contact list. Example: `report.completion() == BezierPairCompletion::Complete`.
    pub fn completion(&self) -> super::BezierPairCompletion {
        use super::{BezierPairCompletion, completion::operation_completion};
        match self.failure() {
            None => BezierPairCompletion::Complete,
            Some(BezierPairFailure::PolicyVersion(_)) => BezierPairCompletion::Invalid,
            Some(BezierPairFailure::Operation(cause)) => operation_completion(cause),
        }
    }
    /// Read the actual owned source binding. Example: `report.source().captured_curves()`.
    pub fn source(&self) -> &CapturedBezierPair {
        let binding = &self.outcome.query;
        // The binding survives both source mutation and destruction of the original query.
        binding
    }
    /// Read the unchanged requested limits. Example: `report.policy().requested_world_error`.
    pub fn policy(&self) -> BezierPairPolicy {
        let requested = self.outcome.policy;
        // Effective generated-sign settings live in the completed certificate separately.
        requested
    }
    /// Read actual completed subdivisions, including failed and cancelled stages. Example: `report.work().completed`.
    pub fn work(&self) -> BezierPairWork {
        self.outcome.outcome.as_ref().map_or(
            BezierPairWork {
                requested: self.policy().roots.max_bisections,
                completed: 0,
            },
            |outcome| outcome.work,
        )
    }
    /// Borrow complete product-domain coverage only after every required check passed. Example: `report.complete().map(|proof| proof.contact_count())`.
    pub fn complete(&self) -> Option<BezierPairCertificate<'_>> {
        let outcome = self.outcome.outcome.as_ref().ok()?;
        let proof = outcome.completion.as_ref().ok()?;
        Some(BezierPairCertificate::new(proof))
    }
    /// Read noncompletion without converting it to completed empty. Example: `assert!(report.failure().is_some())`.
    pub fn failure(&self) -> Option<BezierPairFailure<'_>> {
        match &self.outcome.outcome {
            Err(PairQueryPreparationFailure::Version(value)) => {
                Some(BezierPairFailure::PolicyVersion(*value))
            }
            Ok(outcome) => outcome
                .completion
                .as_ref()
                .err()
                .map(BezierPairFailure::Operation),
        }
    }
}
