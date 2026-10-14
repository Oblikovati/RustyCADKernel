// SPDX-License-Identifier: GPL-2.0-only
//! x=12t(1-t)^2,y=12t²(1-t): y/x=t/(1-t) proves interior injectivity independently.
use super::super::tests::{progress, request};
use super::*;
use crate::sketch_region_contact_transport::test_fixture::{
    actual_inputs_for, finish_fixture_request,
};
use crate::sketch_region_partition_bindings::binding_sources;
use rustycad_tkbrep::topods::Shape;

pub(in crate::sketch_region_prism) fn fixture() -> (PairedContactTransaction, Vec<Shape>) {
    fixture_for(vec![crate::closed_analytic_cubic::closed_analytic_cubic(
        1., false,
    )])
}
pub(super) fn fixture_for(shapes: Vec<Shape>) -> (PairedContactTransaction, Vec<Shape>) {
    let (census, mapped, shapes) = actual_inputs_for(shapes);
    let bridge = finish_fixture_request(census, mapped, &binding_sources(&shapes));
    (bridge, shapes)
}
pub(in crate::sketch_region_prism) fn run(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
) -> PrismReceipt {
    retain_source_bound_prism(
        bridge,
        &binding_sources(shapes),
        request([1., 1.]),
        &mut progress(None, None),
    )
    .ok()
    .expect("actual completed candidate batch")
}
pub(super) fn proof(receipt: &PrismReceipt) -> &RegionCertificate {
    let inspection = receipt.region.as_ref().expect("certification attempted");
    assert_eq!(inspection.version, 1);
    inspection.decision.as_ref().unwrap()
}
pub(super) fn assert_four_present(receipt: &PrismReceipt) {
    for required in obligations() {
        assert!(
            receipt
                .gaps
                .iter()
                .any(|gap| matches!(gap, PrismGap::Candidate(value) if *value==required))
        );
    }
    assert!(
        receipt
            .region
            .as_ref()
            .is_none_or(|row| row.decision.is_err())
    );
}
#[test]
fn source_bound_region_certifies_single_jordan_component_without_body_success() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    assert_eq!(proof(&receipt).selected_winding, 1);
    assert!(!proof(&receipt).boundary_reversed);
    assert_eq!(
        proof(&receipt).source_candidate.face.orientation,
        rustycad_tkg3d::top_abs::Orientation::Forward
    );
    assert!(
        !receipt
            .gaps
            .iter()
            .any(|gap| matches!(gap,PrismGap::Candidate(value) if obligations().contains(value)))
    );
    assert_unpublished(&receipt);
}
fn assert_unpublished(receipt: &PrismReceipt) {
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    assert!(
        receipt
            .diagnostic
            .as_ref()
            .unwrap()
            .coverage
            .as_ref()
            .is_some_and(|coverage| !coverage.complete())
    );
}

#[test]
fn borrowed_original_outcome_and_history_survive_repeated_certification() {
    let (bridge, shapes) = fixture();
    let before = bridge_signature(&bridge);
    let first = run(&bridge, &shapes);
    let second = run(&bridge, &shapes);
    assert_eq!(bridge_signature(&bridge), before);
    assert_eq!(proof(&first).original, proof(&second).original);
    assert_ne!(proof(&first).copied.face, proof(&second).copied.face);
    assert_eq!(proof(&first).contact, proof(&second).contact);
}
fn bridge_signature(bridge: &PairedContactTransaction) -> (usize, usize, usize, u64) {
    let batch = borrowed_batch(bridge).unwrap();
    assert_eq!(
        bridge.inspection.completion,
        crate::sketch_region_contact_transport::ContactTransportCompletion::Incomplete
    );
    (
        batch.candidates.len(),
        batch.candidates.iter().map(|c| c.relations.len()).sum(),
        bridge.inspection.remaining_operation_proofs.len(),
        bridge.inspection.completed_bisections,
    )
}
#[test]
fn legacy_batch_entry_retains_previous_candidate_gap_behavior() {
    let (bridge, _) = fixture();
    let legacy = retain_selected_prism(
        temporary_view(borrowed_batch(&bridge).unwrap()),
        request([1., 1.]),
        &mut progress(None, None),
    );
    assert!(legacy.region.is_none());
    assert!(
        legacy
            .gaps
            .iter()
            .any(|gap| matches!(gap, PrismGap::Candidate(MissingCertificate::Construction)))
    );
    assert!(legacy.gaps.iter().any(|gap| matches!(
        gap,
        PrismGap::Candidate(MissingCertificate::SearchCompleteness)
    )));
}
#[test]
fn unavailable_candidates_preserve_requested_policy_and_explicit_obligations() {
    let (mut bridge, shapes) = fixture();
    bridge.mapped.copied = None;
    let missing = retain_source_bound_prism(
        &bridge,
        &binding_sources(&shapes),
        request([1., 1.]),
        &mut progress(None, None),
    )
    .err()
    .unwrap();
    assert!(matches!(
        missing.cause,
        RegionCertificateCause::UnavailableCandidates
    ));
    assert_eq!(missing.missing, obligations());
    assert_eq!(missing.requested.seed, [1., 1.]);
}
