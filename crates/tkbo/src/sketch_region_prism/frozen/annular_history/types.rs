// SPDX-License-Identifier: GPL-2.0-only
//! ADR0058 scoped rosters: source and candidate scopes are authored lookups, never recovered order.
use super::*;

/// One semantic history scope; `AnnularHistoryScope::InputCopy(RegionSourceId(101))`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AnnularHistoryScope {
    Original(RegionSourceId),
    InputCopy(RegionSourceId),
    Partition,
    CandidateCopy,
    Selected(u64),
    Rejected(u64),
    SweepBase,
}
impl AnnularHistoryScope {
    /// The shared V1 stage carried by scalar history keys; `scope.stage()`.
    pub(crate) fn stage(self) -> PrismHistoryStage {
        match self {
            Self::Original(_) => PrismHistoryStage::Original,
            Self::InputCopy(_) => PrismHistoryStage::InputCopy,
            Self::Partition => PrismHistoryStage::Partition,
            Self::CandidateCopy => PrismHistoryStage::CandidateCopy,
            Self::Selected(_) => PrismHistoryStage::Selected,
            Self::Rejected(_) => PrismHistoryStage::Complement,
            Self::SweepBase => PrismHistoryStage::SweepBase,
        }
    }
}
/// Explicit intermediate names; `AnnularScopeBindings { scope, names }` with `names.stage == scope.stage()`.
pub(crate) struct AnnularScopeBindings {
    pub scope: AnnularHistoryScope,
    pub names: PrismHistoricalStageBindings,
}
/// Authored InputCopy, Partition, CandidateCopy and Rejected scopes; Original/Selected/SweepBase reuse
/// publication and copy bindings. Example: `AnnularHistoryBindings { scopes }`.
pub(crate) struct AnnularHistoryBindings {
    pub scopes: Vec<AnnularScopeBindings>,
}
/// Same retained two-loop request, sweep receipt, authored names and sealed annular join.
/// Example: `AnnularHistoryRequest { region, receipt, publication, copied, historical, joined, requested }`.
pub(crate) struct AnnularHistoryRequest<'a, 'r, 's, 'join> {
    pub region: &'a TwoLoopRegionRequest<'r, 's>,
    pub receipt: &'a PrismReceipt,
    pub publication: &'a PrismPublicationBindings,
    pub copied: &'a PrismCopyBindings,
    pub historical: &'a AnnularHistoryBindings,
    pub joined: &'join AnnularFrozenDomain,
    pub requested: PrismHistoryOptions,
}
impl PrismHistoryOptions {
    /// Bounded two-source annular history; `PrismHistoryOptions::annular_v2()` keeps V1 limits.
    pub(crate) fn annular_v2() -> Self {
        Self {
            version: 2,
            limits: PrismHistoryLimits::bounded_v1(),
        }
    }
}
impl<'a> AnnularHistoryRequest<'a, '_, '_, '_> {
    pub(super) fn native_inputs(&self) -> NativeHistoryInputs<'a> {
        NativeHistoryInputs {
            bridge: self.region.bridge,
            receipt: self.receipt,
            publication: self.publication,
        }
    }
    pub(super) fn binding(&self) -> &AnnularPrismBinding {
        self.joined.embedding().binding()
    }
}
/// One scope's actual rows bound to authored keys; lookup indices remain operation-local.
pub(super) struct AnnularBoundScope<'a> {
    pub scope: AnnularHistoryScope,
    pub contexts: BoundHistoricalContexts<'a>,
}
pub(super) struct AnnularScopeOperands {
    pub scope: AnnularHistoryScope,
    pub rows: Vec<RegionObservedContext>,
}
/// Require one named condition; `holds("same source role", a == b)?` refuses as an exact count.
pub(super) fn holds(required: &'static str, condition: bool) -> HistoryCheck<()> {
    history_count(required, usize::from(condition), 1)
}
