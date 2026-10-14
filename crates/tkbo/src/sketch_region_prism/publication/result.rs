// SPDX-License-Identifier: GPL-2.0-only
//! Only the complete assembler constructs accepted immutable results.
use super::{progress::PrismPublicationPhase, request::*, stage_error::*};
use crate::sketch_region_prism::frozen::{
    AuthoredPrismId, complete_history::*, joined_measures::*, ordered_validation::*, proof_join::*,
};
use rustycad_tkmath::predicates::roots::metric_bounds::CertifiedPolynomialMeasure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismPublicationCompletion {
    CompleteNamedPrismV1,
    CompleteNamedPrismV2,
}
pub(crate) struct PublishedNamedPrism {
    pub(super) join: JoinedPrismProof,
    pub(super) validation: Box<OrderedPrismValidationCertificate>,
    pub(super) history: Box<PrismCompleteHistory>,
    pub(super) measures: JoinedPrismMeasures,
    pub(super) requested: PrismPublicationOptions,
    pub(super) replay: Vec<u8>,
}
pub(crate) struct PrismPublicationFailure<'a> {
    pub operation: AuthoredPrismId,
    pub requested: PrismPublicationOptions,
    pub phase: PrismPublicationPhase,
    pub class: PrismPublicationClass,
    pub cause: PrismPublicationCause<'a>,
}
pub(crate) enum PrismPublicationCause<'a> {
    UnsupportedVersion {
        actual: u32,
        expected: u32,
    },
    Stage(Box<PublicationStageFailure>),
    History(Box<PrismHistoryFailure<'a>>),
    HistoryCancelled {
        operation: AuthoredPrismId,
        phase: PrismHistoryPhase,
    },
}
pub(super) type PublicationCheck<'a, T> = Result<T, Box<PrismPublicationFailure<'a>>>;
impl PublishedNamedPrism {
    /// Read authoritative completion; `result.completion()` does not rewrite historical receipt status.
    pub(crate) fn completion(&self) -> PrismPublicationCompletion {
        if self.requested.version == 2 {
            return PrismPublicationCompletion::CompleteNamedPrismV2;
        }
        PrismPublicationCompletion::CompleteNamedPrismV1
    }
    /// Borrow the immutable body and exact operands; `result.join()` contains no mutable Shape.
    pub(crate) fn join(&self) -> &JoinedPrismProof {
        &self.join
    }
    /// Read current ordered acceptance and historical findings; `result.validation().historical()`.
    pub(crate) fn validation(&self) -> &OrderedPrismValidationCertificate {
        &self.validation
    }
    /// Read composed original/intermediate history; `result.history().continued()`.
    pub(crate) fn history(&self) -> &PrismCompleteHistory {
        &self.history
    }
    /// Read per-face certified areas and achieved errors; `result.measures().total_area()`.
    pub(crate) fn measures(&self) -> &JoinedPrismMeasures {
        &self.measures
    }
    /// Read the unchanged signed integral; `result.signed_sweep_integral().enclosure()` may be negative.
    pub(crate) fn signed_sweep_integral(&self) -> &CertifiedPolynomialMeasure {
        self.measures.signed_volume()
    }
    /// Read original policies without widened budgets; `result.requested().history`.
    pub(crate) fn requested(&self) -> PrismPublicationOptions {
        self.requested
    }
    /// Replay every child operand and conclusion; `result.replay_bytes()` contains no addresses.
    pub(crate) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
}
