// SPDX-License-Identifier: GPL-2.0-only
//! ADR0065: one call from owned analytic coefficients to an owned named annular prism outcome.
//! It composes the existing stages unchanged: preparation, independent two-loop region proof,
//! binding, production names, embedding, publication (join, ordered validation, complete history,
//! measures) and physical volume. No stage is retried, widened or replaced by a fallback.
mod body;
mod outcome;
mod progress;
mod stages;
pub(in crate::sketch_region_prism) use outcome::*;
pub(in crate::sketch_region_prism) use progress::{
    AnnularOperationCheckpoint, AnnularOperationProgress, AnnularOperationStage,
    AnnularOperationStop,
};

use super::super::names::CallerBoundaryId;
use super::names::{AnnularNameDictionary, issue_annular_names};
use super::*;
use crate::sketch_region_prism::diagnostic_envelope::{
    AnnularPreparationRefusalProjection, OwnedAnnularPreparationDiagnostic,
    OwnedAnnularPublicationDiagnostic, OwnedDiagnosticCallbackCause, PrismDiagnosticClass,
    project_annular_preparation_refusal, project_annular_publication_refusal,
};
use crate::sketch_region_prism::frozen::annular_volume::{
    AnnularPrismVolume, AnnularVolumeFailure, certify_annular_prism_volume,
};
use progress::StageProgress;
use rustycad_tkernel::standard::Failure;

#[cfg(test)]
mod tests;

/// Run the whole annular region-to-prism operation on owned inputs.
/// Example: `match run_owned_annular_prism(request, &mut progress) { Published(p) => p.volume(), .. }`.
pub(in crate::sketch_region_prism) fn run_owned_annular_prism(
    request: OwnedAnnularPrismRequest,
    progress: &mut dyn AnnularOperationProgress,
) -> OwnedAnnularPrismOutcome {
    let mut stages = StageProgress::new(progress);
    let prepared = match prepare_owned_annular_prism(request.clone(), &mut stages) {
        Ok(prepared) => prepared,
        Err(failure) => return preparation_refusal(request, failure, stages.stop),
    };
    let outcome = prepared
        .with_region(|region, receipt| stages::construct(&prepared, region, receipt, &mut stages));
    match outcome {
        Ok(prism) => OwnedAnnularPrismOutcome::Published(prism),
        Err(refused) => {
            OwnedAnnularPrismOutcome::Refused(Box::new(refused.attach(request, stages.stop)))
        }
    }
}
fn preparation_refusal(
    request: OwnedAnnularPrismRequest,
    failure: Box<AnnularPreparationFailure>,
    stop: Option<AnnularOperationStop>,
) -> OwnedAnnularPrismOutcome {
    let diagnostic = match project_annular_preparation_refusal(failure) {
        AnnularPreparationRefusalProjection::Projected(diagnostic) => diagnostic,
        // The retained owners are local to this call; the owned diagnostic names them as gaps.
        AnnularPreparationRefusalProjection::Incomplete { diagnostic, .. } => diagnostic,
    };
    let stage = AnnularOperationStage::Preparation;
    let (class, complete) = (diagnostic.class(), diagnostic.gaps().is_empty());
    let diagnostic = AnnularOperationDiagnostic::Preparation(diagnostic);
    let refused = StageRefusal::owned(stage, class, diagnostic, complete);
    OwnedAnnularPrismOutcome::Refused(Box::new(refused.attach(request, stop)))
}
/// A refusal before the request and caller stop are attached.
pub(super) struct StageRefusal {
    stage: AnnularOperationStage,
    class: PrismDiagnosticClass,
    diagnostic: AnnularOperationDiagnostic,
    gaps: Vec<AnnularOperationGap>,
}
impl StageRefusal {
    /// An unprojected borrowed child failure: owned stage and class plus an explicit gap.
    fn new(stage: AnnularOperationStage, class: PrismDiagnosticClass) -> Self {
        Self {
            stage,
            class,
            diagnostic: AnnularOperationDiagnostic::Unprojected,
            gaps: vec![AnnularOperationGap::StageFailure(stage)],
        }
    }
    /// An owned diagnostic, with a gap only when the diagnostic itself records one.
    fn owned(
        stage: AnnularOperationStage,
        class: PrismDiagnosticClass,
        diagnostic: AnnularOperationDiagnostic,
        complete: bool,
    ) -> Self {
        let gaps = match complete {
            true => Vec::new(),
            false => vec![AnnularOperationGap::ChildDiagnostic(stage)],
        };
        Self {
            stage,
            class,
            diagnostic,
            gaps,
        }
    }
    fn attach(
        self,
        request: OwnedAnnularPrismRequest,
        stop: Option<AnnularOperationStop>,
    ) -> OwnedAnnularPrismRefusal {
        // ADR0056 wrapper rule: an actual caller stop classifies first; the child's own class
        // stays in its diagnostic (a cancel-only child API cannot report a native failure).
        let class = match stop.as_ref().map(|s| &s.cause) {
            Some(OwnedDiagnosticCallbackCause::Cancelled) => PrismDiagnosticClass::Cancelled,
            Some(OwnedDiagnosticCallbackCause::Failed(_)) => PrismDiagnosticClass::Failed,
            Some(OwnedDiagnosticCallbackCause::Exhausted) => PrismDiagnosticClass::Incomplete,
            None => self.class,
        };
        OwnedAnnularPrismRefusal {
            request,
            stage: self.stage,
            class,
            diagnostic: self.diagnostic,
            stop,
            gaps: self.gaps,
        }
    }
}
