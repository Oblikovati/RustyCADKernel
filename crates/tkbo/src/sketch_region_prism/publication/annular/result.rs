// SPDX-License-Identifier: GPL-2.0-only
//! Only the complete assembler constructs an accepted annulus; refusals keep actual child causes.
use super::*;

/// One accepted annular prism: every child certificate binds the same sealed join bytes.
pub(in crate::sketch_region_prism) struct PublishedAnnularPrism {
    pub(super) join: AnnularFrozenDomain,
    pub(super) validation: AnnularValidationCertificate,
    pub(super) history: Box<AnnularCompleteHistory>,
    pub(super) measures: AnnularPrismMeasures,
    pub(super) requested: AnnularPublicationOptions,
    pub(super) replay: Vec<u8>,
}
pub(in crate::sketch_region_prism) struct AnnularPublicationFailure<'a> {
    pub operation: AuthoredPrismId,
    pub requested: AnnularPublicationOptions,
    pub phase: PrismPublicationPhase,
    pub class: PrismPublicationClass,
    pub cause: AnnularPublicationCause<'a>,
}
/// Child refusals are retained whole; a callback stop keeps the child it interrupted.
pub(in crate::sketch_region_prism) enum AnnularPublicationCause<'a> {
    UnsupportedVersion {
        actual: u32,
        expected: u32,
    },
    Join(Box<AnnularFrozenFailure<'a>>),
    Validation(Box<AnnularValidationFailure>),
    History(Box<AnnularHistoryFailure<'a>>),
    HistoryCancelled {
        operation: AuthoredPrismId,
        phase: PrismHistoryPhase,
    },
    Measures(Box<AnnularMeasureFailure>),
    Binding {
        proof: &'static str,
    },
    Callback {
        stop: Box<PublicationCallbackStop>,
        downstream: Option<Box<AnnularPublicationCause<'a>>>,
    },
}
pub(super) type AnnularPublicationCheck<'a, T> = Result<T, Box<AnnularPublicationFailure<'a>>>;
impl PublishedAnnularPrism {
    /// Borrow the sealed four-face join; `result.join().faces().len() == 4`.
    pub(in crate::sketch_region_prism) fn join(&self) -> &AnnularFrozenDomain {
        &self.join
    }
    /// Read ordered topology/geometry/intersection/tolerance acceptance; `result.validation()`.
    pub(in crate::sketch_region_prism) fn validation(&self) -> &AnnularValidationCertificate {
        &self.validation
    }
    /// Read complete source/candidate-scoped history; `result.history().continued()`.
    pub(in crate::sketch_region_prism) fn history(&self) -> &AnnularCompleteHistory {
        &self.history
    }
    /// Read per-face certified areas and the signed sweep; `result.measures().faces()`.
    pub(in crate::sketch_region_prism) fn measures(&self) -> &AnnularPrismMeasures {
        &self.measures
    }
    /// Read original policies; `result.requested().history`.
    pub(in crate::sketch_region_prism) fn requested(&self) -> AnnularPublicationOptions {
        self.requested
    }
    /// Replay every child operand and conclusion; `result.replay_bytes()` holds no addresses.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
}
