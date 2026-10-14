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
    Legacy(Box<diagnostics::HistoricalLegacyCause>),
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
        actual: Box<[Vec<FrozenOriginalKey>; 3]>,
        expected: Box<[Vec<FrozenOriginalKey>; 3]>,
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
pub(in crate::sketch_region_prism::frozen) type HistoryCheck<T> = Result<T, PrismHistoryCause>;

pub(in crate::sketch_region_prism::frozen) struct BoundHistoricalContexts<'a> {
    pub actual: &'a [RegionObservedContext],
    pub owned: Vec<PrismHistoricalContext>,
}
pub(in crate::sketch_region_prism::frozen) fn history_count(
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
pub(in crate::sketch_region_prism::frozen) fn same_definition(
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
            Self::AtUse { entity, cause } => {
                debug_fields(formatter, "AtUse", &[("entity", entity), ("cause", cause)])
            }
            Self::Legacy(value) => debug_value(formatter, "UnsupportedLegacyDiagnostic", value),
            Self::Lineage(value) => debug_value(formatter, "Lineage", &value.findings()),
            Self::LineageCancelled(phase) => debug_value(formatter, "LineageCancelled", phase),
            other => other.fmt_operands(formatter),
        }
    }
}
fn debug_fields(
    formatter: &mut std::fmt::Formatter<'_>,
    name: &str,
    fields: &[(&str, &dyn std::fmt::Debug)],
) -> std::fmt::Result {
    let mut result = formatter.debug_struct(name);
    for (name, value) in fields {
        result.field(name, value);
    }
    result.finish()
}
fn debug_value(
    formatter: &mut std::fmt::Formatter<'_>,
    name: &str,
    value: &dyn std::fmt::Debug,
) -> std::fmt::Result {
    formatter.debug_tuple(name).field(value).finish()
}
impl PrismHistoryCause {
    fn fmt_operands(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count {
                required,
                actual,
                expected,
            } => debug_count(formatter, required, actual, expected),
            Self::Missing { required } => debug_value(formatter, "Missing", required),
            Self::Changed { operand } => debug_value(formatter, "Changed", operand),
            Self::ParentsMismatch {
                stage,
                actual,
                expected,
            } => debug_fields(
                formatter,
                "ParentsMismatch",
                &[("stage", stage), ("actual", actual), ("expected", expected)],
            ),
            other => other.fmt_capture(formatter),
        }
    }
    fn fmt_capture(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bounds(value) => debug_value(formatter, "Bounds", value),
            Self::ContextRead { stage, cause } => debug_fields(
                formatter,
                "ContextRead",
                &[("stage", stage), ("cause", cause)],
            ),
            Self::Frozen(value) => debug_value(formatter, "Frozen", value),
            Self::Native(value) => debug_value(formatter, "Native", value),
            Self::Geometry(value) => debug_value(formatter, "Geometry", value),
            Self::UnsupportedDiagnostic {
                stage,
                kind,
                observation,
                retained_rows,
            } => debug_unsupported(formatter, stage, kind, observation, retained_rows),
            _ => unreachable!("capture-specific history diagnostic"),
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
    pub(in crate::sketch_region_prism) fn failure_class(&self) -> PrismHistoryFailureClass {
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

fn debug_count(
    formatter: &mut std::fmt::Formatter<'_>,
    required: &str,
    actual: &usize,
    expected: &usize,
) -> std::fmt::Result {
    debug_fields(
        formatter,
        "Count",
        &[
            ("required", &required),
            ("actual", actual),
            ("expected", expected),
        ],
    )
}
fn debug_unsupported(
    formatter: &mut std::fmt::Formatter<'_>,
    stage: &PrismHistoryStage,
    kind: &HistoricalDiagnosticKind,
    observation: &Option<usize>,
    retained_rows: &usize,
) -> std::fmt::Result {
    debug_fields(
        formatter,
        "UnsupportedDiagnostic",
        &[
            ("stage", stage),
            ("kind", kind),
            ("observation", observation),
            ("retained_rows", retained_rows),
        ],
    )
}
