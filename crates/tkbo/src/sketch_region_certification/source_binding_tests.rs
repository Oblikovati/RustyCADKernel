// SPDX-License-Identifier: GPL-2.0-only
use super::tests::{capture, policy};
use super::*;
use crate::sketch_region_candidates::tests::rectangle;
use crate::sketch_region_source_conformance::*;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
pub(super) fn source_policy() -> SourceConformancePolicy {
    SourceConformancePolicy {
        original_plane_limit: 0.,
        construction_limit: 0.,
        roots: RootIsolationOptions::new(0.002),
    }
}
pub(super) fn rectangle_inputs() -> BoundaryInputs {
    capture(&rectangle(0., 0., 4., 4.))
}
pub(super) fn report(inputs: &BoundaryInputs) -> SourceConformanceInspection {
    inspect_source_conformance(inputs, source_policy(), || false)
}
pub(super) fn bind(
    inputs: &BoundaryInputs,
    report: SourceConformanceInspection,
) -> BoundSourceConformance {
    bind_source_conformance(inputs, source_policy(), report)
}
#[test]
fn full_matrix_discharges_only_representation_and_retains_all_topology_obligations() {
    let census =
        enumerate_source_bound_census(rectangle_inputs(), policy(), source_policy(), || false)
            .unwrap_or_else(|_| panic!("expected diagnostic census"));
    assert_bound_representation(&census);
}
#[test]
fn legacy_census_keeps_its_poll_order_and_source_obligation_without_implicit_policy() {
    let census = super::tests::census(&rectangle(0., 0., 4., 4.));
    assert_representation_only(&census);
}
#[test]
fn missing_or_duplicate_parent_row_never_passes_as_complete_matrix() {
    let inputs = rectangle_inputs();
    let mut missing = report(&inputs);
    missing.entries.pop();
    assert!(!bind(&inputs, missing).representation_certified());
    let mut duplicate = report(&inputs);
    let mut other = report(&inputs);
    duplicate.entries.push(other.entries.remove(0));
    assert!(!bind(&inputs, duplicate).representation_certified());
}
#[test]
fn omitted_original_and_extra_unrepresented_source_are_not_empty_passes() {
    let mut inputs = rectangle_inputs();
    let proof = report(&inputs);
    inputs.originals.pop();
    assert!(!bind(&inputs, proof).representation_certified());
    let mut inputs = rectangle_inputs();
    let extra = inputs.originals[0].clone();
    inputs.originals.push(extra);
    assert!(!bind(&inputs, report(&inputs)).representation_certified());
}
#[test]
fn cancelled_source_phase_keeps_full_owned_failure_and_does_not_start_pairs() {
    let result =
        enumerate_source_bound_census(rectangle_inputs(), policy(), source_policy(), || true);
    let Err(CensusCause::SourceCancelled { completed }) = result else {
        panic!("expected source cancellation")
    };
    assert_source_cancel_prefix(&completed);
}
#[test]
fn actual_report_survives_failure_retry_and_source_topology_alias_changes() {
    let shapes = rectangle(0., 0., 4., 4.);
    let inputs = capture(&shapes);
    let proof = report(&inputs);
    shapes[0].set_closed(true).unwrap();
    inputs.segments[0].edge.set_closed(true).unwrap();
    assert!(bind(&inputs, proof).representation_certified());
    let cancelled = inspect_source_conformance(&inputs, source_policy(), || true);
    assert!(!bind(&inputs, cancelled).representation_certified());
    assert!(bind(&inputs, report(&inputs)).representation_certified());
}
#[test]
fn changed_policy_or_effective_stage_limit_is_not_rebound_to_a_more_lenient_request() {
    let inputs = rectangle_inputs();
    let proof = report(&inputs);
    let mut p = source_policy();
    p.construction_limit = 1.;
    assert!(!bind_source_conformance(&inputs, p, proof).representation_certified());
    let mut proof = report(&inputs);
    proof.entries[0].stages[0].effective_limit = 1.;
    assert!(!bind(&inputs, proof).representation_certified());
}
#[test]
fn work_receipt_and_effective_root_budget_must_match_actual_stage_prefix() {
    let inputs = rectangle_inputs();
    let mut proof = report(&inputs);
    proof.bisections += 1;
    assert!(!bind(&inputs, proof).representation_certified());
    let mut proof = report(&inputs);
    proof.entries[0].stages[0].effective_roots.max_bisections -= 1;
    assert!(!bind(&inputs, proof).representation_certified());
}

fn assert_representation_only(census: &BoundaryCensus) {
    assert!(census.source_binding.is_none());
    assert!(
        census
            .missing
            .contains(&CertificationObligation::SourceConversionAndPlane)
    );
}

fn assert_source_cancel_prefix(completed: &BoundaryCensus) {
    assert!(completed.domains.is_empty());
    assert!(completed.reports.is_empty());
    assert!(completed.source_binding.is_some());
    assert!(
        completed
            .missing
            .contains(&CertificationObligation::SourceConversionAndPlane)
    );
    assert!(
        !completed
            .source_binding
            .as_ref()
            .unwrap()
            .representation_certified()
    );
}

fn assert_bound_representation(census: &BoundaryCensus) {
    let source = census.source_binding.as_ref().unwrap();
    assert!(source.representation_certified());
    assert!(
        !census
            .missing
            .contains(&CertificationObligation::SourceConversionAndPlane)
    );
    assert!(
        census
            .missing
            .contains(&CertificationObligation::MappedSplitCorrespondence)
    );
    assert!(census.paired.is_some());
    assert!(
        !census
            .current_obligations()
            .contains(&CertificationObligation::FailedWorkAccounting)
    );
    assert_separate_root_policies(census, source);
}

fn assert_separate_root_policies(census: &BoundaryCensus, source: &BoundSourceConformance) {
    assert_eq!(source.inspection().requested.roots.max_width, 0.002);
    assert_eq!(census.requested.roots.max_width, 0.001);
}
