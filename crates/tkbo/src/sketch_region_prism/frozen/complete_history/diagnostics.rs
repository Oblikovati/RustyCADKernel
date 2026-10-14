// SPDX-License-Identifier: GPL-2.0-only
//! Unsupported owner-bearing causes retain their full native payload in a separate local envelope.
use super::super::proof_join::*;
use super::*;
use crate::sketch_region::{ArrangementCompletion, PartitionObservation, PartitionReadCause};
use crate::sketch_region_candidates::{CandidateCause, CandidateGap};
#[derive(Debug)]
pub(crate) enum HistoricalLegacyCause {
    Join {
        phase: PrismProofJoinPhase,
        entity: Option<FrozenPrismKey>,
        variant: HistoryJoinVariant,
    },
    Partition {
        source: Option<i32>,
        vertex: Option<i32>,
        block: Option<usize>,
        parents: Vec<FrozenOriginalKey>,
        cause: HistoryPartitionReadCause,
    },
    Candidate {
        observation: Option<usize>,
        variant: HistoryCandidateVariant,
    },
    Arrangement {
        variant: HistoryArrangementVariant,
    },
}
#[derive(Debug)]
pub(crate) enum HistoryJoinVariant {
    MissingClosureQuery,
    ClosureQuery,
    ClosureRegion,
    Version {
        actual: u32,
        expected: u32,
    },
    Embedding {
        phase: crate::sketch_region_prism::region_certificate::embedding::PrismEmbeddingPhase,
    },
    IncompleteCapture {
        phase: FreezePhase,
    },
    FailedCapture {
        phase: FreezePhase,
    },
    Binding,
    Link,
    Cancelled {
        freeze_phase: Option<FreezePhase>,
    },
    Stale {
        representation: &'static str,
    },
}
#[derive(Debug)]
pub(crate) enum HistoryPartitionReadCause {
    Native(rustycad_tkernel::standard::Failure),
    MissingContext(&'static str),
    MissingSourceParents(i32),
    MissingCopiedSource(usize),
    PoisonedPaveBlock,
    MissingImageIndex(i32),
    MissingVertexIndex(i32),
}
#[derive(Debug)]
pub(crate) enum HistoryCandidateVariant {
    Native(rustycad_tkernel::standard::Failure),
    MappedReport,
    Cancelled(crate::sketch_region_candidates::CandidatePhase),
    Numerical,
    CapturedNumerical,
    CapturedOperation {
        retained_occurrences: usize,
        cause: Box<HistoryCandidateVariant>,
    },
    WindingOverflow(i64),
    UnsupportedOrientation(rustycad_tkg3d::top_abs::Orientation),
    UnclassifiedOccurrence,
    WorkBudget {
        requested: u16,
        used: u32,
    },
    UnattributedMember,
    UnsupportedCurve {
        occurrence: u64,
        dynamic_type: String,
    },
    MissingParents(u64),
    IncompleteWire {
        omitted: usize,
        unexpected: usize,
    },
}
#[derive(Debug)]
pub(crate) enum HistoryArrangementVariant {
    CompletedEmpty { deleted: usize },
    Declined,
    Failed,
    Cancelled(crate::sketch_region::ArrangementPhase),
    Incomplete,
}
pub(super) fn admit_observations<'a>(
    request: &PrismHistoryRequest<'a>,
) -> Result<(), PrismHistoryFailure<'a>> {
    admit_partition(request)?;
    admit_arrangement(request)?;
    let Some(copy) = &request.bridge.mapped.copied else {
        return Ok(());
    };
    if let Err(cause) = &copy.completion {
        return Err(candidate_failure(request, None, cause));
    }
    if let Some(cause) = copy.observation_failures.first() {
        return Err(candidate_failure(request, Some(0), cause));
    }
    Ok(())
}
fn admit_partition<'a>(request: &PrismHistoryRequest<'a>) -> Result<(), PrismHistoryFailure<'a>> {
    let PartitionObservation::Incomplete { failure, .. } =
        &request.bridge.mapped.arrangement.partition
    else {
        return Ok(());
    };
    let parents = parent_rows::original_keys(request, &failure.parents)
        .map_err(|cause| operation::failure(request, PrismHistoryPhase::Admission, cause))?;
    let cause = HistoricalLegacyCause::Partition {
        source: failure.source_index,
        vertex: failure.vertex_index,
        block: failure.block_position,
        parents,
        cause: partition_cause(&failure.cause),
    };
    let mut result = operation::failure(
        request,
        PrismHistoryPhase::Admission,
        PrismHistoryCause::Legacy(cause),
    );
    result.local_diagnostic = Some(PrismHistoryLocalDiagnostic::Partition(failure));
    Err(result)
}
fn partition_cause(cause: &PartitionReadCause) -> HistoryPartitionReadCause {
    match cause {
        PartitionReadCause::Native(v) => HistoryPartitionReadCause::Native((**v).clone()),
        PartitionReadCause::MissingContext(v) => HistoryPartitionReadCause::MissingContext(v),
        PartitionReadCause::MissingSourceParents(v) => {
            HistoryPartitionReadCause::MissingSourceParents(*v)
        }
        PartitionReadCause::MissingCopiedSource(v) => {
            HistoryPartitionReadCause::MissingCopiedSource(*v)
        }
        PartitionReadCause::PoisonedPaveBlock => HistoryPartitionReadCause::PoisonedPaveBlock,
        PartitionReadCause::MissingImageIndex(v) => {
            HistoryPartitionReadCause::MissingImageIndex(*v)
        }
        PartitionReadCause::MissingVertexIndex(v) => {
            HistoryPartitionReadCause::MissingVertexIndex(*v)
        }
    }
}
fn admit_arrangement<'a>(request: &PrismHistoryRequest<'a>) -> Result<(), PrismHistoryFailure<'a>> {
    let actual = &request.bridge.mapped.arrangement.completion;
    let variant = match actual {
        ArrangementCompletion::Arranged(_) => return Ok(()),
        ArrangementCompletion::CompletedEmpty { deleted } => {
            HistoryArrangementVariant::CompletedEmpty {
                deleted: deleted.len(),
            }
        }
        ArrangementCompletion::Declined(_) => HistoryArrangementVariant::Declined,
        ArrangementCompletion::Failed(_) => HistoryArrangementVariant::Failed,
        ArrangementCompletion::Cancelled(phase) => HistoryArrangementVariant::Cancelled(*phase),
        ArrangementCompletion::Incomplete(_) => HistoryArrangementVariant::Incomplete,
    };
    let mut error = operation::failure(
        request,
        PrismHistoryPhase::Admission,
        PrismHistoryCause::Legacy(HistoricalLegacyCause::Arrangement { variant }),
    );
    error.local_diagnostic = Some(PrismHistoryLocalDiagnostic::Arrangement(actual));
    Err(error)
}
fn candidate_failure<'a>(
    request: &PrismHistoryRequest<'a>,
    observation: Option<usize>,
    cause: &'a CandidateCause,
) -> PrismHistoryFailure<'a> {
    let variant = candidate_variant(cause);
    let mut error = operation::failure(
        request,
        PrismHistoryPhase::Admission,
        PrismHistoryCause::Legacy(HistoricalLegacyCause::Candidate {
            observation,
            variant,
        }),
    );
    error.local_diagnostic = Some(PrismHistoryLocalDiagnostic::Candidate(cause));
    error
}
fn candidate_variant(cause: &CandidateCause) -> HistoryCandidateVariant {
    match cause {
        CandidateCause::Mapped(cause) => HistoryCandidateVariant::Native(cause.clone()),
        CandidateCause::MappedReport(_) => HistoryCandidateVariant::MappedReport,
        CandidateCause::Incomplete(cause) => candidate_gap(cause),
        CandidateCause::Cancelled(phase) => HistoryCandidateVariant::Cancelled(*phase),
        CandidateCause::Numerical(_) => HistoryCandidateVariant::Numerical,
        CandidateCause::CapturedNumerical(_) => HistoryCandidateVariant::CapturedNumerical,
        CandidateCause::CapturedOperation(value) => HistoryCandidateVariant::CapturedOperation {
            retained_occurrences: value.retained.occurrences.len(),
            cause: Box::new(candidate_variant(&value.cause)),
        },
    }
}
fn candidate_gap(cause: &CandidateGap) -> HistoryCandidateVariant {
    match cause {
        CandidateGap::WindingOrientationOverflow { winding } => {
            HistoryCandidateVariant::WindingOverflow(*winding)
        }
        CandidateGap::UnsupportedFaceOrientation { orientation } => {
            HistoryCandidateVariant::UnsupportedOrientation(*orientation)
        }
        CandidateGap::UnclassifiedOccurrence { .. } => {
            HistoryCandidateVariant::UnclassifiedOccurrence
        }
        CandidateGap::WorkBudget { requested, used } => HistoryCandidateVariant::WorkBudget {
            requested: *requested,
            used: *used,
        },
        CandidateGap::UnattributedMember { .. } => HistoryCandidateVariant::UnattributedMember,
        CandidateGap::UnsupportedCurve {
            occurrence,
            dynamic_type,
        } => HistoryCandidateVariant::UnsupportedCurve {
            occurrence: *occurrence,
            dynamic_type: dynamic_type.clone(),
        },
        CandidateGap::MissingParents { occurrence } => {
            HistoryCandidateVariant::MissingParents(*occurrence)
        }
        CandidateGap::IncompleteWire {
            omitted,
            unexpected,
        } => HistoryCandidateVariant::IncompleteWire {
            omitted: omitted.len(),
            unexpected: unexpected.len(),
        },
    }
}
pub(super) fn join_failure<'a>(
    request: &PrismHistoryRequest<'a>,
    phase: PrismHistoryPhase,
    cause: PrismProofJoinFailure,
) -> PrismHistoryFailure<'a> {
    let description = HistoricalLegacyCause::Join {
        phase: cause.phase,
        entity: cause.entity,
        variant: join_variant(&cause.cause),
    };
    let mut error = operation::failure(request, phase, PrismHistoryCause::Legacy(description));
    error.local_diagnostic = Some(PrismHistoryLocalDiagnostic::Join(Box::new(cause)));
    error
}
fn join_variant(cause: &PrismProofJoinCause) -> HistoryJoinVariant {
    match cause {
        PrismProofJoinCause::MissingClosureQuery => HistoryJoinVariant::MissingClosureQuery,
        PrismProofJoinCause::ClosureQuery(_) => HistoryJoinVariant::ClosureQuery,
        PrismProofJoinCause::ClosureRegion(_) => HistoryJoinVariant::ClosureRegion,
        PrismProofJoinCause::Version { actual, expected } => HistoryJoinVariant::Version {
            actual: *actual,
            expected: *expected,
        },
        PrismProofJoinCause::Embedding(v) => HistoryJoinVariant::Embedding { phase: v.phase },
        PrismProofJoinCause::IncompleteCapture(v) => {
            HistoryJoinVariant::IncompleteCapture { phase: v.phase }
        }
        PrismProofJoinCause::FailedCapture(v) => {
            HistoryJoinVariant::FailedCapture { phase: v.phase }
        }
        PrismProofJoinCause::Binding(_) => HistoryJoinVariant::Binding,
        PrismProofJoinCause::Link(_) => HistoryJoinVariant::Link,
        PrismProofJoinCause::Cancelled { freeze_phase } => HistoryJoinVariant::Cancelled {
            freeze_phase: *freeze_phase,
        },
        PrismProofJoinCause::Stale { representation } => {
            HistoryJoinVariant::Stale { representation }
        }
    }
}
