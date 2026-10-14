// SPDX-License-Identifier: GPL-2.0-only
//! Report reuse is operation-local; occurrence history and actual spent work survive every noncompletion.
use super::paired_tests::*;
use super::source_binding_tests::source_policy;
use super::tests::{capture, policy};
use super::*;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::predicates::roots::bezier_pair::*;

#[test]
fn reversed_aliases_reuse_cold_report_and_retain_all_full_original_parent_keys() {
    let mut shapes = curved_pair();
    shapes.push(shapes[0].oriented(Orientation::Reversed));
    let census = paired(&shapes);
    assert_eq!(census.domains.len(), 6);
    let retained = census.paired.as_ref().unwrap();
    assert!(retained.reused_reports > 0);
    assert_eq!(retained.occurrences.len(), 3);
    assert_eq!(
        retained.occurrences[0].physical_source,
        retained.occurrences[2].physical_source
    );
    assert_alias_parent_keys(&census);
    assert!(super::paired_binding::domain_certificate(&census, &census.domains[4]).is_some());
    assert_exact_work(&census);
}
fn assert_exact_work(census: &BoundaryCensus) {
    let sum = census
        .paired
        .as_ref()
        .unwrap()
        .reports
        .iter()
        .map(|record| record.completed_work())
        .sum::<u64>();
    assert_eq!(census.bisections, sum);
    assert!(super::paired_binding::paired_work_exact(census));
    assert!(
        !census
            .current_obligations()
            .contains(&CertificationObligation::FailedWorkAccounting)
    );
}
#[test]
fn actual_source_and_policy_mutation_cannot_rebind_a_previous_complete_report() {
    let mut census = paired(&curved_pair());
    let domain = 1;
    assert!(super::paired_binding::domain_certificate(&census, &census.domains[domain]).is_some());
    census.inputs.segments[0].controls[1][0] += 0.5;
    assert!(super::paired_binding::domain_certificate(&census, &census.domains[domain]).is_none());
    let mut census = paired(&curved_pair());
    census.requested.max_world_error *= 2.;
    assert!(super::paired_binding::domain_certificate(&census, &census.domains[domain]).is_none());
}
#[test]
fn full_history_kind_and_occurrence_changes_reject_binding_without_rewriting_the_report() {
    let mut census = paired(&curved_pair());
    let old = distinct_record(&census)
        .outcome
        .as_ref()
        .unwrap()
        .source()
        .clone();
    census.inputs.relations[0].modified.clear();
    assert!(super::paired_binding::domain_certificate(&census, &census.domains[1]).is_none());
    assert_eq!(
        distinct_record(&census).outcome.as_ref().unwrap().source(),
        &old
    );
    let mut census = paired(&curved_pair());
    census.inputs.segments[0].occurrence += 100;
    assert!(super::paired_binding::domain_certificate(&census, &census.domains[1]).is_none());
}
#[test]
fn located_source_occurrences_keep_distinct_domains_and_actual_world_coefficients() {
    use rustycad_tkmath::{
        gp::{Transform3, Vector3},
        toploc::Location,
    };
    let source = curved_pair().remove(0);
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(10., 0., 0.));
    let located = source.located(&Location::new(transform), true).unwrap();
    let census = paired(&[source, located]);
    let retained = census.paired.as_ref().unwrap();
    assert_ne!(
        retained.occurrences[0].physical_source,
        retained.occurrences[1].physical_source
    );
    assert_eq!(retained.occurrences[1].source.controls[0], [10., 0.]);
    assert_eq!(census.domains.len(), 3);
}
#[test]
fn retained_owned_report_survives_later_caller_geometry_mutation_failure_and_cancel() {
    let shapes = curved_pair();
    let published = paired(&shapes);
    let old = retained_source(&published);
    mutate_caller_curve(&shapes[0]);
    let mut invalid = policy();
    invalid.max_world_error = -1.;
    assert!(
        enumerate_source_bound_census(capture(&shapes), invalid, source_policy(), || false)
            .is_err()
    );
    assert!(
        enumerate_source_bound_census(capture(&shapes), policy(), source_policy(), || true)
            .is_err()
    );
    assert_eq!(retained_source(&published), old);
    assert!(complete_distinct(&published).contact_count() > 0);
}
#[test]
fn global_failure_work_is_spent_and_not_hidden_as_complete_empty_or_refunded() {
    let shapes = irrational_pair();
    let mut requested = policy();
    requested.roots.max_bisections = 1;
    let census =
        enumerate_source_bound_census(capture(&shapes), requested, source_policy(), || false)
            .unwrap_or_else(|_| panic!("expected owned noncompletion records"));
    assert!(census.bisections > 0 && census.bisections <= 1);
    assert!(
        paired_reports(&census)
            .iter()
            .any(|record| record.completion() == BezierPairCompletion::Incomplete)
    );
    assert_exact_work(&census);
    assert!(has_obligation(
        &census,
        CertificationObligation::ExhaustivePairAndSelfDomains
    ));
    assert!(complete_distinct(&paired(&shapes)).contact_count() > 0);
}
fn irrational_pair() -> Vec<rustycad_tkbrep::topods::Shape> {
    vec![
        bezier_shape(&[[0., 0.], [0.5, 0.], [1., 1.]], None),
        bezier_shape(&[[0., 0.5], [1., 0.5]], None),
    ]
}
#[test]
fn exact_cold_hits_need_no_new_budget_and_accounted_failure_hits_do_not_retry() {
    let mut shapes = irrational_pair();
    shapes.push(shapes[0].reversed());
    let mut requested = policy();
    requested.roots.max_bisections = 1;
    let census =
        enumerate_source_bound_census(capture(&shapes), requested, source_policy(), || false)
            .unwrap_or_else(|_| panic!("expected exhausted receipt with reused domains"));
    assert!(census.paired.as_ref().unwrap().reused_reports > 0);
    assert_exact_work(&census);
    assert_eq!(census.bisections, 1);
}
#[test]
fn forged_gap_removal_or_modified_work_never_promotes_current_coverage() {
    let mut census = paired(&curved_pair());
    census.missing.clear();
    assert!(has_obligation(
        &census,
        CertificationObligation::MappedSplitCorrespondence
    ));
    assert!(has_obligation(
        &census,
        CertificationObligation::SelfIntersection
    ));
    census.bisections += 1;
    assert!(super::paired_binding::domain_certificate(&census, &census.domains[1]).is_none());
    assert!(has_obligation(
        &census,
        CertificationObligation::FailedWorkAccounting
    ));
    assert_removed_domain_incomplete();
}
fn assert_removed_domain_incomplete() {
    let mut census = paired(&curved_pair());
    census.domains.pop();
    census.missing.clear();
    assert!(has_obligation(
        &census,
        CertificationObligation::ExhaustivePairAndSelfDomains
    ));
}

#[test]
fn complete_closed_curve_alias_hit_is_valid_with_zero_new_split_allowance() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let shapes = [source.clone(), source.reversed()];
    let mut requested = policy();
    requested.roots.max_bisections = 0;
    let census =
        enumerate_source_bound_census(capture(&shapes), requested, source_policy(), || false)
            .unwrap_or_else(|_| panic!("expected completed proof reuse with no subdivisions"));
    let records = census.paired.as_ref().unwrap();
    assert_eq!(records.reports.len(), 1);
    assert_eq!(records.reused_reports, 1);
    assert_eq!(
        records.reports[0].completion(),
        BezierPairCompletion::Complete
    );
    assert_eq!(census.bisections, 0);
    assert_exact_work(&census);
}
struct StopAfterCensusPoll {
    calls: usize,
    limit: usize,
}
impl StopAfterCensusPoll {
    fn new(limit: usize) -> Self {
        let calls = 0;
        Self { calls, limit }
    }
    fn stopped(&mut self) -> bool {
        self.calls += 1;
        self.calls >= self.limit
    }
}
#[test]
fn cancellation_after_actual_pair_work_retains_failed_report_and_exact_spent_prefix() {
    let census = cancelled_after_pair_work();
    assert!(census.bisections > 0);
    assert!(super::paired_binding::paired_work_exact(&census));
    assert!(
        paired_reports(&census)
            .iter()
            .any(|record| record.completion() == BezierPairCompletion::Cancelled)
    );
    assert!(has_obligation(
        &census,
        CertificationObligation::ExhaustivePairAndSelfDomains
    ));
    assert!(complete_distinct(&paired(&irrational_pair())).contact_count() > 0);
}
fn cancelled_after_pair_work() -> BoundaryCensus {
    let total_polls = successful_operation_polls();
    let mut callback = StopAfterCensusPoll::new(total_polls);
    let result = enumerate_source_bound_census(
        capture(&irrational_pair()),
        policy(),
        source_policy(),
        || callback.stopped(),
    );
    cancelled_pair_prefix(result)
}
fn cancelled_pair_prefix(result: Result<BoundaryCensus, CensusCause>) -> BoundaryCensus {
    let Err(CensusCause::Cancelled {
        completed,
        during_domain: true,
        ..
    }) = result
    else {
        panic!(
            "expected actual final provider poll cancellation after completed earlier pair work"
        );
    };
    *completed
}
#[test]
fn unrelated_callback_failure_propagates_without_rewriting_previous_report() {
    let published = paired(&curved_pair());
    let old = retained_source(&published);
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        enumerate_source_bound_census(capture(&curved_pair()), policy(), source_policy(), || {
            std::panic::panic_any(73u32)
        })
    }));
    assert_eq!(*failure.err().unwrap().downcast::<u32>().unwrap(), 73);
    assert_eq!(retained_source(&published), old);
    assert!(complete_distinct(&paired(&curved_pair())).contact_count() > 0);
}

fn assert_alias_parent_keys(census: &BoundaryCensus) {
    let retained = census.paired.as_ref().unwrap();
    assert!(
        census
            .domains
            .iter()
            .any(|domain| domain.evidence
                == DomainEvidence::Unsupported(CensusGap::SamePhysicalCurve))
    );
    assert!(retained.occurrences[2].source.reversed);
    assert!(retained.occurrences[2].relations.iter().any(|row| {
        row.modified
            .iter()
            .any(|parent| parent.orientation == Orientation::Reversed)
    }));
}

fn mutate_caller_curve(shape: &rustycad_tkbrep::topods::Shape) {
    use rustycad_tkbrep::{brep::BRepTool, topods::edge};
    use rustycad_tkg3d::geom::GeomBezierCurve;
    let (curve, _, _) = BRepTool::curve(&edge(shape).unwrap()).unwrap();
    curve
        .cast::<GeomBezierCurve>()
        .set_pole(2, rustycad_tkmath::gp::Point3::new(9., 0., 0.))
        .unwrap();
}

fn successful_operation_polls() -> usize {
    let mut polls = 0;
    let baseline = enumerate_source_bound_census(
        capture(&irrational_pair()),
        policy(),
        source_policy(),
        || {
            polls += 1;
            false
        },
    )
    .unwrap_or_else(|_| panic!("expected deterministic completed census baseline"));
    assert!(baseline.bisections > 0);
    polls
}
