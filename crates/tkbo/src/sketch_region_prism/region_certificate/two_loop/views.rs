// SPDX-License-Identifier: GPL-2.0-only
//! Borrowed values expose this certified planar subset without a constructor for proof authority.
use super::{
    census_settings::TwoLoopCensusSettings, nesting::TwoLoopNestingProof, outcome::*,
    pair_types::TwoLoopPairProof, request::TwoLoopRegionOptions,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TwoLoopRegionLimitation {
    ComposedCopyAndDeletionHistory,
    SweptCapAndSideCorrespondence,
    OrderedBodyValidation,
    TolerancePolicyAcceptance,
    AreaAndVolumeAcceptance,
    ConcurrentSourceAliasMutation,
}
impl TwoLoopRegionCertificate {
    /// Read the caller's operation name; `proof.operation_id()`.
    pub(crate) fn operation_id(&self) -> u64 {
        self.operation
    }
    /// Read the proved selected candidate name; `proof.selected_candidate_id()`.
    pub(crate) fn selected_candidate_id(&self) -> u64 {
        self.selected
    }
    /// Borrow exact original/census geometry and native closure evidence; `proof.sources()`.
    pub(crate) fn sources(&self) -> &[TwoLoopSourceProof] {
        &self.sources
    }
    /// Borrow complete closure-only self and empty cross-domain evidence; `proof.domains()`.
    pub(crate) fn domains(&self) -> &[TwoLoopPairProof] {
        &self.domains
    }
    /// Borrow every actual selected/rejected region and directed member path; `proof.candidates()`.
    pub(crate) fn candidates(&self) -> &[TwoLoopCandidateProof] {
        &self.candidates
    }
    /// Borrow bound native closure, checked interior and material witness proofs; `proof.nesting()`.
    pub(crate) fn nesting(&self) -> &TwoLoopNestingProof {
        &self.nesting
    }
    /// Borrow provider requests, exact chart rank and actual historical work; `proof.census_settings()`.
    pub(crate) fn census_settings(&self) -> &TwoLoopCensusSettings {
        &self.census
    }
    /// Read the unmodified request; `proof.requested()` equals its effective top-level policy.
    pub(crate) fn requested(&self) -> TwoLoopRegionOptions {
        self.requested
    }
    /// Read actual aggregate captured structural units, including query temporaries; `proof.retained_units()`.
    pub(crate) fn retained_units(&self) -> usize {
        self.retained_units
    }
    /// Borrow all named operands and conclusions; `proof.replay_bytes()` has no owner addresses.
    pub(crate) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
    /// Read explicit excluded publication obligations; `proof.limitations()` never means a valid body.
    pub(crate) fn limitations(&self) -> &'static [TwoLoopRegionLimitation] {
        use TwoLoopRegionLimitation::*;
        &[
            ComposedCopyAndDeletionHistory,
            SweptCapAndSideCorrespondence,
            OrderedBodyValidation,
            TolerancePolicyAcceptance,
            AreaAndVolumeAcceptance,
            ConcurrentSourceAliasMutation,
        ]
    }
}
