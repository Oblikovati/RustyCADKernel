// SPDX-License-Identifier: GPL-2.0-only
//! Retained annular preparation state; owns every native shape so later borrows stay local.
use super::*;

/// Actual stage causes; general child causes reuse the single-boundary carrier unchanged.
pub(in crate::sketch_region_prism) enum AnnularPreparationCause {
    Admission(AnnularAdmissionCause),
    Stage(OwnedPreparationCause),
    Role {
        required: &'static str,
        actual: usize,
        expected: usize,
    },
}
pub(super) struct AnnularPreparationRetained {
    pub request: OwnedAnnularPrismRequest,
    pub sources: Vec<ConstructedPolynomialSource>,
    pub census: Option<BoundaryCensus>,
    pub partition: Option<PartitionBoundOutputs>,
    pub bridge: Option<PairedContactTransaction>,
    pub identities: Option<TwoLoopRegionIdentities>,
    pub prism: Option<PrismReceipt>,
    pub phase: OwnedPrismPhase,
}
/// Prepared annular diagnostics; a prepared stack is not a published body.
pub(in crate::sketch_region_prism) struct PreparedAnnularPrism {
    pub(super) retained: AnnularPreparationRetained,
    pub(super) bridge: PairedContactTransaction,
    pub(super) identities: TwoLoopRegionIdentities,
    pub(super) receipt: PrismReceipt,
}
/// Actual child causes and the retained partial state stay together for later projection.
pub(in crate::sketch_region_prism) struct AnnularPreparationFailure {
    pub class: OwnedPreparationClass,
    pub cause: AnnularPreparationCause,
    pub callback: Option<OwnedPrismCallbackStop>,
    pub phase: OwnedPrismPhase,
    pub(super) retained: AnnularPreparationRetained,
}
pub(super) type PreparationCheck<T> = Result<T, AnnularPreparationCause>;
impl AnnularPreparationRetained {
    pub(super) fn new(request: OwnedAnnularPrismRequest) -> Self {
        Self {
            request,
            sources: Vec::new(),
            census: None,
            partition: None,
            bridge: None,
            identities: None,
            prism: None,
            phase: OwnedPrismPhase::Admission,
        }
    }
    /// Borrow both constructed sources in authored outer/inner role order.
    pub(super) fn arrangement_sources(&self) -> Vec<ArrangementSource<'_>> {
        self.sources.iter().map(|s| s.source()).collect()
    }
}
/// Missing retained state is incomplete, never a silent default.
pub(super) fn local<T>(value: Option<T>, expected: &'static str) -> PreparationCheck<T> {
    let missing = OwnedPreparationCause::MissingLocalState { expected };
    value.ok_or(AnnularPreparationCause::Stage(missing))
}
impl From<OwnedPreparationCause> for AnnularPreparationCause {
    fn from(cause: OwnedPreparationCause) -> Self {
        Self::Stage(cause)
    }
}
