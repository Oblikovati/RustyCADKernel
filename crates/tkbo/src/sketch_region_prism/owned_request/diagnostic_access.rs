// SPDX-License-Identifier: GPL-2.0-only
//! Borrowed local provenance is distinct from the later immutable diagnostic snapshot.
use super::{diagnostics::*, names::*, progress::*, request::*};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RetainedPreparationStage {
    ConstructedSource,
    BoundaryInputs,
    Census,
    Partition,
    ContactTransport,
    Prism,
}
/// Operation-local view; it can borrow mutable-owner containers and is not a published result.
pub(crate) struct PreparationDiagnosticView<'a> {
    retained: &'a PreparationRetained,
}
impl OwnedPrismPreparationFailure {
    /// Inspect actual failed invocation inputs; `failure.diagnostic_context().request()`.
    pub(crate) fn diagnostic_context(&self) -> PreparationDiagnosticView<'_> {
        // Only the actual local failure can supply its retained invocation state.
        PreparationDiagnosticView {
            retained: &self.retained,
        }
    }
}
impl PreparedPolynomialPrism {
    /// Inspect the actual prepared invocation; `prepared.diagnostic_context().plan()`.
    pub(crate) fn diagnostic_context(&self) -> PreparationDiagnosticView<'_> {
        // This grants no publication authority or independent identity correspondence.
        PreparationDiagnosticView {
            retained: &self.retained,
        }
    }
}
impl PreparationDiagnosticView<'_> {
    /// Borrow every original operand and policy; `view.request().sources[0].revision`.
    pub(crate) fn request(&self) -> &OwnedPolynomialPrismRequest {
        // No new request is synthesized from a successful subset of the operation.
        &self.retained.request
    }
    /// Preserve the actual last-entered phase; `view.phase()`.
    pub(crate) fn phase(&self) -> OwnedPrismPhase {
        // The cause or retained container count cannot substitute for the original phase.
        self.retained.phase
    }
    /// Borrow the validated dictionary if admission reached it; `view.plan().is_none()`.
    pub(crate) fn plan(&self) -> Option<&SingleBoundaryNamePlan> {
        // Failed admission must not manufacture a semantic dictionary.
        self.retained.plan.as_ref()
    }
    /// Identify local owners requiring diagnostic coverage; `view.retained_stages()`.
    pub(crate) fn retained_stages(&self) -> Vec<RetainedPreparationStage> {
        use RetainedPreparationStage::*;
        [
            (ConstructedSource, self.retained.source.is_some()),
            (BoundaryInputs, self.retained.inputs.is_some()),
            (Census, self.retained.census.is_some()),
            (Partition, self.retained.partition.is_some()),
            (ContactTransport, self.retained.bridge.is_some()),
            (Prism, self.retained.prism.is_some()),
        ]
        .into_iter()
        .filter_map(|(stage, present)| present.then_some(stage))
        .collect()
    }
}
