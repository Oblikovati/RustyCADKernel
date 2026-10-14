// SPDX-License-Identifier: GPL-2.0-only
//! Owned partial evidence preserves causes and never becomes an ordered stage completion.
use super::*;
use crate::sketch_region_prism::exact_tolerance::{
    StoredToleranceCap, failure_class::*, frames::ToleranceFrameFailure,
    scalar::ExactEndpointFailure,
};
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;
pub(in crate::sketch_region_prism) enum AnnularConformanceCause {
    Body(Box<JoinedConformanceCause>),
    SourceFrame(ToleranceFrameFailure),
    SourceEndpoint(Box<ExactEndpointFailure>),
    SourceReport {
        stage: crate::sketch_region_source_conformance::ConstructionStage,
        actual: ResidualDecision,
    },
}
pub(in crate::sketch_region_prism) struct AnnularConformanceFailure {
    pub operation: &'static str,
    pub phase: JoinedConformancePhase,
    pub entity: Option<FrozenPrismKey>,
    pub occurrence: Option<FrozenUseKey>,
    pub source_context: Option<AnnularSourceContext>,
    pub cause: AnnularConformanceCause,
    pub partial: AnnularConformanceEvidence,
    pub binding: Vec<u8>,
    pub limits: FrozenPrismLimits,
}
pub(in crate::sketch_region_prism) struct AnnularToleranceFailure {
    pub operation: &'static str,
    pub phase: JoinedTolerancePhase,
    pub entity: Option<FrozenPrismKey>,
    pub cause: JoinedToleranceCause,
    pub metadata: Vec<JoinedToleranceMetadata>,
    pub binding: Vec<u8>,
    pub limits: FrozenPrismLimits,
    pub cap: StoredToleranceCap,
}
impl From<JoinedConformanceCause> for AnnularConformanceCause {
    fn from(cause: JoinedConformanceCause) -> Self {
        Self::Body(Box::new(cause))
    }
}
impl AnnularConformanceFailure {
    /// Preserve exact discrepancy witnesses versus unsupported proof; `failure.classification()`.
    pub(in crate::sketch_region_prism) fn classification(&self) -> JoinedEvidenceClass {
        match &self.cause {
            AnnularConformanceCause::Body(cause) => classify_conformance_cause(cause),
            AnnularConformanceCause::SourceEndpoint(cause) => endpoint_class(cause),
            AnnularConformanceCause::SourceReport {
                actual: ResidualDecision::Exceeds { .. },
                ..
            } => JoinedEvidenceClass::Failed,
            _ => JoinedEvidenceClass::Incomplete,
        }
    }
}
impl AnnularToleranceFailure {
    /// Reuse the existing exact metadata cause classification; `failure.classification()`.
    pub(in crate::sketch_region_prism) fn classification(&self) -> JoinedEvidenceClass {
        classify_tolerance_cause(&self.cause)
    }
}
pub(super) type AnnularExactCheck<T = ()> = Result<T, AnnularConformanceCause>;
