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
#[derive(Debug)]
pub(crate) enum PrismHistoryCause {
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
    Bounds(HistoryBoundCause),
    Frozen(Box<PrismFreezeCause>),
    Native(std::sync::Arc<rustycad_tkernel::standard::Failure>),
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
