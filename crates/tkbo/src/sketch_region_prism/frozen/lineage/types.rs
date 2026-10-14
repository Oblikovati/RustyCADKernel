// SPDX-License-Identifier: GPL-2.0-only
use super::super::*;

/// Additional authored identities; example: `PrismCopyBindings { selected_members, selected_uses, copied_uses }`.
/// Actual contexts are lookup operands; authored bindings alone never prove correspondence.
pub(crate) struct PrismCopyBindings {
    pub selected_members: Vec<PrismSourceBinding>,
    pub selected_uses: Vec<PrismDirectedBinding>,
    pub copied_uses: Vec<PrismDirectedBinding>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CopyLineageStage {
    Selected,
    Copied,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CopyMemberKey {
    pub operation: AuthoredPrismId,
    pub stage: CopyLineageStage,
    pub member: AuthoredPrismId,
    pub kind: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CopyUseKey(pub Vec<(CopyMemberKey, AuthoredPrismId)>);
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CopyOccurrence {
    pub member: CopyMemberKey,
    pub orientation: u8,
    pub placement_bits: [u64; 13],
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CopyMemberObservation {
    pub selected: CopyOccurrence,
    pub native_image: CopyOccurrence,
    pub copied: CopyOccurrence,
    pub selected_parents: [Vec<FrozenOriginalKey>; 3],
    pub copied_parents: [Vec<FrozenOriginalKey>; 3],
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CopyContextObservation {
    pub key: CopyUseKey,
    pub occurrence: CopyOccurrence,
    pub children: Vec<CopyUseKey>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CopyDirectedObservation {
    pub selected: CopyUseKey,
    pub copied: CopyUseKey,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CopySweepObservation {
    pub copied: CopyUseKey,
    pub output: FrozenUseKey,
    pub role: FrozenSweepRole,
    pub parents: [Vec<FrozenOriginalKey>; 3],
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CopyRetirementObservation {
    pub copied: CopyOccurrence,
    pub parents: [Vec<FrozenOriginalKey>; 3],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CopyLineageCoverage {
    Complete,
    Incomplete,
    Blocked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CopyLineagePhase {
    Members,
    Directed,
    Sweep,
    Frozen,
    Commit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CopyLineageLimitation {
    HistoricalSelectedGeometryUnproven,
    UpstreamPartitionProvenanceUnbound,
    OriginalDeletionCompositionIncomplete,
    WholeBodyValidationIncomplete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CopyParentClass {
    Preserved,
    Modified,
    Generated,
}
impl CopyParentClass {
    pub(super) const ALL: [Self; 3] = [Self::Preserved, Self::Modified, Self::Generated];
}
#[derive(Debug)]
pub(crate) enum CopyLineageCause {
    Count {
        required: &'static str,
        actual: usize,
        expected: usize,
    },
    Parents {
        class: CopyParentClass,
        actual: Vec<FrozenOriginalKey>,
        expected: Vec<FrozenOriginalKey>,
    },
    Changed {
        operand: &'static str,
    },
    Frozen(Box<PrismFreezeCause>),
    FreezeFailure(Box<PrismFreezeFailure>),
}
#[derive(Debug)]
pub(crate) struct CopyLineageFinding {
    pub operation: AuthoredPrismId,
    pub phase: CopyLineagePhase,
    pub entity: Option<CopyMemberKey>,
    pub cause: CopyLineageCause,
}
pub(crate) struct PrismLineageInspection {
    pub(super) operation: AuthoredPrismId,
    pub(super) member_coverage: CopyLineageCoverage,
    pub(super) directed_coverage: CopyLineageCoverage,
    pub(super) members: Vec<CopyMemberObservation>,
    pub(super) contexts: Vec<CopyContextObservation>,
    pub(super) directed: Vec<CopyDirectedObservation>,
    pub(super) sweep: Vec<CopySweepObservation>,
    pub(super) retired: Vec<CopyRetirementObservation>,
    pub(super) findings: Vec<CopyLineageFinding>,
    pub(super) projection: Option<Box<FrozenPrismProjection>>,
}
pub(crate) enum PrismLineageOutcome {
    Inspected(Box<PrismLineageInspection>),
    Incomplete(Box<PrismLineageInspection>),
    Failed(Box<PrismLineageInspection>),
    Cancelled {
        operation: AuthoredPrismId,
        phase: FreezePhase,
    },
}
pub(super) type LineageCheck<T> = Result<T, CopyLineageFinding>;
