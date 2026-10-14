// SPDX-License-Identifier: GPL-2.0-only
//! Boundary checkpoints are explicit; mapped copy/sweep calls are not interruptible.
use super::{copying::*, history::observe_sweep, outcome::*};
use crate::sketch_region_candidates::*;
use crate::sketch_region_validation::{
    BodyKind, StageCoverage, ValidationStage, source_relations, validate_owned_candidate,
};
use rustycad_tkernel::message::Report;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// Retain diagnostic output and all non-success causes; `retain_selected_prism(batch,request,&mut progress)`.
pub(crate) fn retain_selected_prism(
    batch: CandidateBatch,
    request: PrismRequest,
    progress: &mut dyn PrismProgress,
) -> PrismReceipt {
    let mut receipt = initial_receipt(&batch, request);
    if let Err(completion) = execute(batch, &mut receipt, progress) {
        receipt.completion = completion;
    }
    receipt
}
fn initial_receipt(batch: &CandidateBatch, requested: PrismRequest) -> PrismReceipt {
    PrismReceipt {
        completion: PrismCompletion::Incomplete,
        phase: PrismPhase::Admission,
        limits: PrismLimits {
            requested,
            effective: PrismAppliedSettings::default(),
            achieved_error: None,
        },
        classification: None,
        cancelled_query: None,
        copied: None,
        input: initial_history(batch),
        diagnostic: None,
        gaps: vec![
            PrismGap::PrismInjectivityAndSurfaceCharts,
            PrismGap::AchievedErrorUnknown,
        ],
    }
}
fn initial_history(batch: &CandidateBatch) -> PrismInputHistory {
    let report = std::sync::Arc::new(Report::new());
    report.merge(&batch.mapped_report);
    PrismInputHistory {
        rejected: batch
            .candidates
            .iter()
            .map(|candidate| candidate.relations.clone())
            .collect(),
        input_deleted: batch.input_deleted.clone(),
        unrepresented: batch.unrepresented.clone(),
        report,
        selected: None,
        declared_plane: batch.plane,
    }
}
fn execute(
    batch: CandidateBatch,
    receipt: &mut PrismReceipt,
    progress: &mut dyn PrismProgress,
) -> Result<(), PrismCompletion> {
    checkpoint(receipt, progress, PrismPhase::Admission)?;
    admit_request(&receipt.limits.requested).map_err(PrismCompletion::Failed)?;
    checkpoint(receipt, progress, PrismPhase::Selection)?;
    select_batch(batch, receipt, progress)?;
    checkpoint(receipt, progress, PrismPhase::Copy)?;
    retain_copy(receipt)?;
    sweep(receipt, progress)
}
fn select_batch(
    mut batch: CandidateBatch,
    receipt: &mut PrismReceipt,
    progress: &mut dyn PrismProgress,
) -> Result<(), PrismCompletion> {
    receipt.limits.effective.seed = Some(receipt.limits.requested.seed);
    receipt.limits.effective.roots = Some(receipt.limits.requested.roots);
    let classified = classify_progress(&batch, receipt, progress);
    receipt.input.unrepresented = std::mem::take(&mut batch.unrepresented);
    if !receipt.input.unrepresented.is_empty() {
        receipt.gaps.push(PrismGap::UnrepresentedInput);
    }
    let index = classified.and_then(|result| retain_classification(result, receipt));
    let selected = index.map(|index| {
        receipt.input.selected = Some(batch.candidates.remove(index));
    });
    retain_rejected(batch, receipt);
    selected
}
fn classify_progress(
    batch: &CandidateBatch,
    receipt: &mut PrismReceipt,
    progress: &mut dyn PrismProgress,
) -> Result<CandidateClassification, PrismCompletion> {
    let request = receipt.limits.requested;
    let mut failure = None;
    let mut stopped = false;
    let result = classify_candidates(batch, request.seed, request.roots, || {
        poll_selection(progress, &mut failure, &mut stopped)
    });
    result.map_err(|query| classification_failure(query, failure, stopped, receipt))
}
fn classification_failure(
    query: CandidateCause,
    failure: Option<rustycad_tkernel::standard::Failure>,
    stopped: bool,
    receipt: &mut PrismReceipt,
) -> PrismCompletion {
    if let Some(failure) = failure {
        return PrismCompletion::Failed(PrismCause::ClassificationProgress {
            failure: Box::new(failure),
            query: Box::new(query),
        });
    }
    if stopped {
        receipt.cancelled_query = Some(Box::new(query));
        return PrismCompletion::Cancelled(PrismPhase::Selection);
    }
    PrismCompletion::Failed(PrismCause::Classification(Box::new(query)))
}
fn retain_copy(receipt: &mut PrismReceipt) -> Result<(), PrismCompletion> {
    let selected = receipt
        .input
        .selected
        .as_ref()
        .expect("unique selected candidate retained");
    let (base, relations) = copied_selection(selected).map_err(PrismCompletion::Failed)?;
    receipt.copied = Some(PrismCopiedSelection { base, relations });
    Ok(())
}
fn retain_rejected(batch: CandidateBatch, receipt: &mut PrismReceipt) {
    receipt.input.rejected = batch
        .candidates
        .into_iter()
        .map(|candidate| candidate.relations)
        .collect();
}
fn retain_classification(
    classification: CandidateClassification,
    receipt: &mut PrismReceipt,
) -> Result<usize, PrismCompletion> {
    let CandidateClassification::Incomplete {
        selection, missing, ..
    } = &classification;
    receipt
        .gaps
        .extend(missing.iter().copied().map(PrismGap::Candidate));
    let selection = selection.clone();
    receipt.classification = Some(classification);
    match selection {
        SeedSelection::Unique { candidate } => Ok(candidate),
        disposition => Err(PrismCompletion::NoBody(disposition)),
    }
}
fn sweep(
    receipt: &mut PrismReceipt,
    progress: &mut dyn PrismProgress,
) -> Result<(), PrismCompletion> {
    checkpoint(receipt, progress, PrismPhase::Sweep)?;
    receipt.limits.effective.extrusion = Some(receipt.limits.requested.extrusion);
    let copied = receipt
        .copied
        .as_ref()
        .expect("completed deep copy retained");
    // Owned base precedes sweep; copy=false avoids an unobserved second geometry copy.
    let mut maker = construct_prism(&copied.base, receipt.limits.requested.extrusion)?;
    retain_body(&mut maker, receipt)?;
    checkpoint(receipt, progress, PrismPhase::History)?;
    retain_history(&mut maker, receipt)?;
    capture_coverage_gaps(receipt);
    checkpoint(receipt, progress, PrismPhase::Validation)?;
    inspect_output(receipt);
    Ok(())
}
fn retain_body(
    maker: &mut BRepPrimAPIMakePrism,
    receipt: &mut PrismReceipt,
) -> Result<(), PrismCompletion> {
    let body = maker
        .shape()
        .map_err(|failure| PrismCompletion::Failed(mapped(PrismPhase::Sweep, failure)))?
        .clone();
    receipt.diagnostic = Some(PrismDiagnostic {
        body,
        bindings: Vec::new(),
        transport_gaps: Vec::new(),
        relations: Vec::new(),
        native_retired: Vec::new(),
        coverage: None,
    });
    Ok(())
}
fn capture_coverage_gaps(receipt: &mut PrismReceipt) {
    let Some(diagnostic) = &receipt.diagnostic else {
        return;
    };
    if !diagnostic.native_retired.is_empty() {
        receipt.gaps.push(PrismGap::ComposedDeletionCoverage);
    }
    for row in &diagnostic.relations {
        if row.preserved.is_empty() && row.modified.is_empty() && row.generated.is_empty() {
            receipt
                .gaps
                .push(PrismGap::UnattributedSweepOccurrence(row.output.clone()));
        }
    }
}
/// Retain ordered validation without success promotion; `inspect_output(&mut receipt)`.
pub(super) fn inspect_output(receipt: &mut PrismReceipt) {
    let Some(diagnostic) = receipt.diagnostic.as_mut() else {
        return;
    };
    let relations = source_relations(&diagnostic.relations);
    let coverage = validate_owned_candidate(&diagnostic.body, BodyKind::Solid, &relations);
    if let Some(stage) = failed_stage(&coverage) {
        receipt.completion = PrismCompletion::Failed(PrismCause::ValidationRejected(stage));
    }
    diagnostic.coverage = Some(coverage);
}
fn checkpoint(
    receipt: &mut PrismReceipt,
    progress: &mut dyn PrismProgress,
    phase: PrismPhase,
) -> Result<(), PrismCompletion> {
    receipt.phase = phase;
    match progress.checkpoint(phase) {
        Ok(false) => Ok(()),
        Ok(true) => Err(PrismCompletion::Cancelled(phase)),
        Err(failure) => Err(PrismCompletion::Failed(mapped(phase, failure))),
    }
}
fn admit_request(request: &PrismRequest) -> Result<(), PrismCause> {
    let vector = [
        request.extrusion.x(),
        request.extrusion.y(),
        request.extrusion.z(),
    ];
    if vector.iter().any(|value| !value.is_finite()) || vector.iter().all(|value| *value == 0.) {
        return Err(PrismCause::InvalidRequest {
            quantity: "finite nonzero extrusion",
            actual: vector.to_vec(),
        });
    }
    admit_limit("nonnegative spatial limit", request.spatial_limit)?;
    admit_limit("nonnegative parameter limit", request.parameter_limit)
}
fn admit_limit(quantity: &'static str, actual: f64) -> Result<(), PrismCause> {
    if actual.is_finite() && actual >= 0. {
        return Ok(());
    }
    Err(PrismCause::InvalidRequest {
        quantity,
        actual: vec![actual],
    })
}

fn failed_stage(
    coverage: &crate::sketch_region_validation::OrderedCoverage,
) -> Option<ValidationStage> {
    [
        (coverage.topology, ValidationStage::Topology),
        (coverage.geometry, ValidationStage::Geometry),
        (
            coverage.self_intersection,
            ValidationStage::SelfIntersection,
        ),
    ]
    .into_iter()
    .find_map(|(status, stage)| (status == StageCoverage::Failed).then_some(stage))
}

fn construct_prism(
    base: &rustycad_tkbrep::topods::Shape,
    extrusion: rustycad_tkmath::gp::Vector3,
) -> Result<BRepPrimAPIMakePrism, PrismCompletion> {
    BRepPrimAPIMakePrism::new(base, extrusion, false, true)
        .map_err(|failure| PrismCompletion::Failed(mapped(PrismPhase::Sweep, failure)))
}
fn retain_history(
    maker: &mut BRepPrimAPIMakePrism,
    receipt: &mut PrismReceipt,
) -> Result<(), PrismCompletion> {
    let parents = &receipt
        .copied
        .as_ref()
        .expect("completed copy retained")
        .relations;
    receipt.diagnostic = Some(observe_sweep(maker, parents).map_err(PrismCompletion::Failed)?);
    Ok(())
}

fn poll_selection(
    progress: &mut dyn PrismProgress,
    failure: &mut Option<rustycad_tkernel::standard::Failure>,
    stopped: &mut bool,
) -> bool {
    match progress.checkpoint(PrismPhase::Selection) {
        Ok(stop) => {
            *stopped |= stop;
            stop
        }
        Err(cause) => {
            *failure = Some(cause);
            true
        }
    }
}
