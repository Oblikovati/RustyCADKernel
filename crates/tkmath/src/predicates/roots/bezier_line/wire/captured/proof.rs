// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::super::{RootIsolation, RootIsolationOptions};
pub use super::super::super::winding::CapturedWindingEvidence;
use super::super::super::winding::WindingEvaluation;
use super::super::endpoint::Endpoint;
use super::CapturedBezierWireInput;
/// A completed capture; fields cannot be forged into successful evidence.
pub struct CapturedBezierWindingProof {
    pub(super) input: CapturedBezierWireInput,
    pub(super) endpoints: [Endpoint; 2],
    pub(super) evaluated: WindingEvaluation,
    pub(super) effective: RootIsolationOptions,
}
impl CapturedBezierWindingProof {
    /// Read the actual regular denominator isolation; example: `proof.denominator().roots().is_empty()`.
    pub fn denominator(&self) -> &RootIsolation {
        &self.evaluated.denominator
    }
    /// Read roots/signs/contributions from the executed branch; example: `proof.evidence()`.
    pub fn evidence(&self) -> &CapturedWindingEvidence {
        &self.evaluated.evidence
    }
    /// Read the effective remaining policy used on first capture; example: `proof.effective_policy()`.
    pub fn effective_policy(&self) -> RootIsolationOptions {
        self.effective
    }
    /// Read historical completed split count, not new replay work; example: `proof.bisections()`.
    pub fn bisections(&self) -> u16 {
        self.evaluated.bisections
    }
}
