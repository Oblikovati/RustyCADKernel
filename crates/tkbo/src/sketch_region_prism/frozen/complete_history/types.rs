// SPDX-License-Identifier: GPL-2.0-only
//! Authored intermediate names add identity, never reconstruct it from original-parent sets.
use super::*;
use crate::sketch_region::{PartitionOccurrenceIdentity, RegionObservedContext};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PrismHistoryStage {
    Original,
    InputCopy,
    Partition,
    CandidateCopy,
    Selected,
    Complement,
    SweepBase,
}
/// Scalar lookup operands are not replay identities; `PrismHistoricalUseBinding { actual, authored_use }`.
pub(crate) struct PrismHistoricalUseBinding {
    pub actual: RegionObservedContext,
    pub authored_use: AuthoredPrismId,
}
pub(crate) struct PrismHistoricalStageBindings {
    pub stage: PrismHistoryStage,
    pub members: Vec<PrismSourceBinding>,
    pub uses: Vec<PrismHistoricalUseBinding>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismHistoryMemberKey {
    pub operation: AuthoredPrismId,
    pub stage: PrismHistoryStage,
    pub member: AuthoredPrismId,
    pub kind: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismHistoryUseKey(pub Vec<(PrismHistoryMemberKey, AuthoredPrismId)>);
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismHistoricalOccurrence {
    pub member: PrismHistoryMemberKey,
    pub orientation: u8,
    pub placement_bits: [u64; 13],
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismHistoricalContext {
    pub key: PrismHistoryUseKey,
    pub occurrence: PrismHistoricalOccurrence,
    pub children: Vec<PrismHistoryUseKey>,
}
pub(crate) enum PrismHistoryCause {
    AtUse {
        entity: PrismHistoryUseKey,
        cause: Box<PrismHistoryCause>,
    },
    Legacy(diagnostics::HistoricalLegacyCause),
    Lineage(Box<super::super::lineage::PrismLineageInspection>),
    LineageCancelled(FreezePhase),
    Count {
        required: &'static str,
        actual: usize,
        expected: usize,
    },
    Missing {
        required: &'static str,
    },
    Changed {
        operand: &'static str,
    },
    ParentsMismatch {
        stage: PrismHistoryStage,
        actual: [Vec<FrozenOriginalKey>; 3],
        expected: [Vec<FrozenOriginalKey>; 3],
    },
    Bounds(HistoryBoundCause),
    ContextRead {
        stage: PrismHistoryStage,
        cause: HistoricalContextReadCause,
    },
    Frozen(Box<PrismFreezeCause>),
    Native(std::sync::Arc<rustycad_tkernel::standard::Failure>),
    Geometry(std::sync::Arc<crate::sketch_region_geometry_capture::CapturedGeometryCause>),
    UnsupportedDiagnostic {
        stage: PrismHistoryStage,
        kind: HistoricalDiagnosticKind,
        observation: Option<usize>,
        retained_rows: usize,
    },
}
pub(super) type HistoryCheck<T> = Result<T, PrismHistoryCause>;

pub(super) struct BoundHistoricalContexts<'a> {
    pub actual: &'a [RegionObservedContext],
    pub owned: Vec<PrismHistoricalContext>,
}
pub(super) fn history_count(
    required: &'static str,
    actual: usize,
    expected: usize,
) -> HistoryCheck<()> {
    if actual != expected {
        return Err(PrismHistoryCause::Count {
            required,
            actual,
            expected,
        });
    }
    Ok(())
}
pub(super) fn same_definition(
    a: &PartitionOccurrenceIdentity,
    b: &PartitionOccurrenceIdentity,
) -> bool {
    super::super::identities::same_definition(a, b)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismHistoricalCopyQuery {
    pub source: PrismHistoryUseKey,
    pub native_images: Vec<PrismHistoricalOccurrence>,
    pub copied: PrismHistoryUseKey,
}

/// Explicit additional intermediate identities; `PrismHistoryBindings { complementary_region, intermediate }`.
pub(crate) struct PrismHistoryBindings {
    pub complementary_region: AuthoredPrismId,
    pub intermediate: Vec<PrismHistoricalStageBindings>,
}
pub(crate) struct PrismHistoryRequest<'a, 'join> {
    pub bridge: &'a crate::sketch_region_contact_transport::PairedContactTransaction,
    pub sources: &'a [crate::sketch_region::ArrangementSource<'a>],
    pub receipt: &'a crate::sketch_region_prism::PrismReceipt,
    pub publication: &'a PrismPublicationBindings,
    pub copied: &'a super::super::lineage::PrismCopyBindings,
    pub historical: &'a PrismHistoryBindings,
    pub joined: &'join super::super::proof_join::JoinedPrismProof,
    pub requested: PrismHistoryOptions,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PrismHistoryOptions {
    pub version: u32,
    pub limits: PrismHistoryLimits,
}
impl PrismHistoryOptions {
    /// Bounded one-Jordan history behavior; `PrismHistoryOptions::v1()` refuses unsupported domains.
    pub(crate) fn v1() -> Self {
        Self {
            version: 1,
            limits: PrismHistoryLimits::bounded_v1(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HistoricalContextReadCause {
    NotReached,
    RepeatedAncestor,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismHistoricalRelation {
    pub output: PrismHistoryUseKey,
    pub parents: [Vec<FrozenOriginalKey>; 3],
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismPartitionQuery {
    pub original: FrozenOriginalKey,
    pub copied: PrismHistoricalOccurrence,
    pub kind: u8,
    pub supported: bool,
    pub modified: Vec<PrismHistoricalOccurrence>,
    pub generated: Vec<PrismHistoricalOccurrence>,
    pub deleted: bool,
    pub continued: PrismHistoryUseKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HistoricalDiagnosticKind {
    GeneratedObservation,
    CandidateCompletion,
    CandidateObservation,
    PartitionRead,
    ArrangementCompletion,
}

impl std::fmt::Debug for PrismHistoryCause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AtUse { entity, cause } => formatter
                .debug_struct("AtUse")
                .field("entity", entity)
                .field("cause", cause)
                .finish(),
            Self::Legacy(value) => formatter
                .debug_tuple("UnsupportedLegacyDiagnostic")
                .field(value)
                .finish(),
            Self::Lineage(value) => formatter
                .debug_tuple("Lineage")
                .field(&value.findings())
                .finish(),
            Self::LineageCancelled(phase) => formatter
                .debug_tuple("LineageCancelled")
                .field(phase)
                .finish(),
            Self::Count {
                required,
                actual,
                expected,
            } => formatter
                .debug_struct("Count")
                .field("required", required)
                .field("actual", actual)
                .field("expected", expected)
                .finish(),
            Self::Missing { required } => formatter.debug_tuple("Missing").field(required).finish(),
            Self::Changed { operand } => formatter.debug_tuple("Changed").field(operand).finish(),
            Self::ParentsMismatch {
                stage,
                actual,
                expected,
            } => formatter
                .debug_struct("ParentsMismatch")
                .field("stage", stage)
                .field("actual", actual)
                .field("expected", expected)
                .finish(),
            Self::Bounds(value) => formatter.debug_tuple("Bounds").field(value).finish(),
            Self::ContextRead { stage, cause } => formatter
                .debug_struct("ContextRead")
                .field("stage", stage)
                .field("cause", cause)
                .finish(),
            Self::Frozen(value) => formatter.debug_tuple("Frozen").field(value).finish(),
            Self::Native(value) => formatter.debug_tuple("Native").field(value).finish(),
            Self::Geometry(value) => formatter.debug_tuple("Geometry").field(value).finish(),
            Self::UnsupportedDiagnostic {
                stage,
                kind,
                observation,
                retained_rows,
            } => formatter
                .debug_struct("UnsupportedDiagnostic")
                .field("stage", stage)
                .field("kind", kind)
                .field("observation", observation)
                .field("retained_rows", retained_rows)
                .finish(),
        }
    }
}

impl PrismHistoryCause {
    pub(super) fn at_use(self, entity: PrismHistoryUseKey) -> Self {
        Self::AtUse {
            entity,
            cause: Box::new(self),
        }
    }
    pub(super) fn primary_use(&self) -> Option<PrismHistoryUseKey> {
        match self {
            Self::AtUse { entity, .. } => Some(entity.clone()),
            _ => None,
        }
    }
    pub(super) fn failure_class(&self) -> PrismHistoryFailureClass {
        match self {
            Self::AtUse { cause, .. } => cause.failure_class(),
            Self::Native(_) | Self::Geometry(_) => PrismHistoryFailureClass::Failed,
            Self::Legacy(cause) if diagnostics::legacy_failed(cause) => {
                PrismHistoryFailureClass::Failed
            }
            _ => PrismHistoryFailureClass::Incomplete,
        }
    }
}
