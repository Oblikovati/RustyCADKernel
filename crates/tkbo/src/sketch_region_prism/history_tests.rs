// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
use crate::sketch_region::{ArrangementRelation, equal_occurrences};
use crate::sketch_region_candidates::{
    CandidateClassification, MissingCertificate,
    tests::{candidates, rectangle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
#[test]
fn rectangle_sweep_retains_every_actual_occurrence_and_source_parent() {
    let batch = candidates(&rectangle(0., 0., 4., 3.));
    let result = retain_selected_prism(batch, request([2., 1.]), &mut progress(None, None));
    let diagnostic = result.diagnostic.unwrap();
    let actual = equal_occurrences(&diagnostic.body).unwrap();
    assert_eq!(actual.len(), diagnostic.relations.len());
    for output in actual {
        let row = diagnostic
            .relations
            .iter()
            .find(|row| row.output.is_equal(&output))
            .unwrap();
        assert!(!row.modified.is_empty() || !row.generated.is_empty());
    }
    assert_rectangle_sources(&diagnostic.relations);
}
#[test]
fn copied_generated_parent_kind_survives_caps_and_generates_sides() {
    let mut batch = candidates(&rectangle(0., 0., 4., 3.));
    for candidate in &mut batch.candidates {
        for relation in &mut candidate.relations {
            relation.generated.append(&mut relation.modified);
        }
    }
    let result = retain_selected_prism(batch, request([2., 1.]), &mut progress(None, None));
    let diagnostic = result.diagnostic.unwrap();
    assert_generated_only(&diagnostic.relations);
}
#[test]
fn missing_history_remains_unknown_and_is_not_fabricated_deletion() {
    let mut batch = candidates(&rectangle(0., 0., 4., 3.));
    for candidate in &mut batch.candidates {
        candidate.relations.clear();
    }
    let result = retain_selected_prism(batch, request([2., 1.]), &mut progress(None, None));
    assert!(matches!(result.completion, PrismCompletion::Incomplete));
    assert!(
        result
            .gaps
            .iter()
            .any(|gap| matches!(gap, PrismGap::UnattributedSweepOccurrence(_)))
    );
    assert!(result.input.input_deleted.is_empty());
}
#[test]
fn input_unknown_and_deleted_metadata_survive_even_early_failure() {
    let mut batch = candidates(&rectangle(0., 0., 4., 3.));
    let row = batch.candidates[0].relations[0].clone();
    let key = row
        .generated
        .first()
        .or_else(|| row.modified.first())
        .unwrap()
        .clone();
    batch.unrepresented.push(row);
    batch.input_deleted.push(key.clone());
    let result = retain_selected_prism(
        batch,
        request([2., 1.]),
        &mut progress(None, Some(PrismPhase::Admission)),
    );
    assert_eq!(result.input.input_deleted, vec![key]);
    assert_eq!(result.input.unrepresented.len(), 1);
    assert_eq!(result.input.rejected.len(), 2);
}
#[test]
fn upstream_missing_history_certificate_survives_classification_and_sweep() {
    let mut batch = candidates(&rectangle(0., 0., 4., 3.));
    batch
        .unrepresented
        .push(batch.candidates[0].relations[0].clone());
    let result = retain_selected_prism(batch, request([2., 1.]), &mut progress(None, None));
    let CandidateClassification::Incomplete { missing, .. } = result.classification.unwrap();
    assert!(missing.contains(&MissingCertificate::CandidateHistoryCoverage));
    assert!(
        result
            .gaps
            .iter()
            .any(|gap| matches!(gap, PrismGap::UnrepresentedInput))
    );
}
#[test]
fn history_phase_cancel_keeps_constructed_output_without_accepted_history() {
    let result = cubic_receipt([1., 1.], &mut progress(Some(PrismPhase::History), None));
    let diagnostic = result.diagnostic.unwrap();
    assert_eq!(diagnostic.body.shape_type().unwrap(), ShapeEnum::Solid);
    assert!(diagnostic.relations.is_empty());
    assert!(diagnostic.coverage.is_none());
}
#[test]
fn missing_copy_uses_actual_unrelated_entity_and_retains_failure_scope() {
    let mut batch = candidates(&rectangle(0., 0., 4., 3.));
    let unrelated = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    for candidate in &mut batch.candidates {
        candidate.relations.push(ArrangementRelation {
            output: unrelated.clone(),
            preserved: Vec::new(),
            modified: Vec::new(),
            generated: Vec::new(),
        });
    }
    let result = retain_selected_prism(batch, request([2., 1.]), &mut progress(None, None));
    assert!(matches!(
        result.completion,
        PrismCompletion::Failed(PrismCause::Mapped {
            phase: PrismPhase::Copy,
            ..
        })
    ));
}

#[test]
fn native_wire_retirement_is_not_flattened_into_deleted_original_edges() {
    let batch = candidates(&rectangle(0., 0., 4., 3.));
    let result = retain_selected_prism(batch, request([2., 1.]), &mut progress(None, None));
    let diagnostic = result.diagnostic.unwrap();
    assert!(!diagnostic.native_retired.is_empty());
    assert!(
        diagnostic
            .native_retired
            .iter()
            .all(|row| row.output.shape_type().unwrap() == ShapeEnum::Wire)
    );
    assert!(result.input.input_deleted.is_empty());
    assert!(
        result
            .gaps
            .iter()
            .any(|gap| matches!(gap, PrismGap::ComposedDeletionCoverage))
    );
}

fn assert_rectangle_sources(relations: &[ArrangementRelation]) {
    let sources: std::collections::BTreeSet<_> = relations
        .iter()
        .flat_map(|r| r.modified.iter().chain(&r.generated))
        .map(|p| p.source.0)
        .collect();
    assert_eq!(sources, [1, 2, 3, 4].into_iter().collect());
}

fn assert_generated_only(relations: &[ArrangementRelation]) {
    assert!(relations.iter().all(|row| row.modified.is_empty()));
    assert!(relations.iter().all(|row| !row.generated.is_empty()));
    assert!(!relations.is_empty());
}

#[test]
fn actual_wrong_body_kind_blocks_later_validation_and_retains_typed_finding() {
    use crate::sketch_region_validation::ValidationStage;
    let mut result = cubic_receipt([1., 1.], &mut progress(None, None));
    let actual = candidates(&rectangle(0., 0., 4., 3.))
        .candidates
        .remove(0)
        .face
        .into_shape();
    result.diagnostic.as_mut().unwrap().body = actual;
    super::transaction::inspect_output(&mut result);
    assert!(matches!(
        result.completion,
        PrismCompletion::Failed(PrismCause::ValidationRejected(ValidationStage::Topology))
    ));
    let report = result.diagnostic.unwrap().coverage.unwrap();
    assert_blocked_after_topology(&report);
}

fn assert_blocked_after_topology(report: &crate::sketch_region_validation::OrderedCoverage) {
    use crate::sketch_region_validation::StageCoverage;
    assert_eq!(report.topology, StageCoverage::Failed);
    assert_eq!(report.geometry, StageCoverage::Blocked);
    assert_eq!(report.self_intersection, StageCoverage::Blocked);
    assert_eq!(report.tolerance, StageCoverage::Blocked);
    assert_eq!(report.findings.len(), 1);
}

#[test]
fn every_postselection_stop_retains_selected_and_completed_copy_history() {
    for phase in [
        PrismPhase::Copy,
        PrismPhase::Sweep,
        PrismPhase::History,
        PrismPhase::Validation,
    ] {
        let result = cubic_receipt([1., 1.], &mut progress(Some(phase), None));
        assert_postselection_history(&result, phase);
        let failed = cubic_receipt([1., 1.], &mut progress(None, Some(phase)));
        assert_postselection_history(&failed, phase);
    }
}
fn assert_postselection_history(result: &PrismReceipt, phase: PrismPhase) {
    let selected = result
        .input
        .selected
        .as_ref()
        .expect("selected occurrence retained before copy");
    assert!(!selected.relations.is_empty());
    assert_eq!(result.input.rejected.len(), 1);
    if phase == PrismPhase::Copy {
        assert!(result.copied.is_none());
        return;
    }
    let copied = result
        .copied
        .as_ref()
        .expect("completed isolated copy retained before sweep");
    assert!(!copied.base.is_same(&selected.face));
    assert_eq!(copied.relations.len(), selected.relations.len());
    assert_copy_parent_kinds(&selected.relations, &copied.relations);
}

fn assert_copy_parent_kinds(before: &[ArrangementRelation], after: &[ArrangementRelation]) {
    for (original, copied) in before.iter().zip(after) {
        assert_eq!(copied.generated, original.generated);
        assert_eq!(
            copied.modified,
            original
                .preserved
                .iter()
                .chain(&original.modified)
                .cloned()
                .collect::<Vec<_>>()
        );
    }
}
