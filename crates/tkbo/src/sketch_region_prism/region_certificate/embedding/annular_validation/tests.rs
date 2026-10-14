// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::frozen::annular_join::operation_tests::joined;
#[test]
fn complete_actual_annular_table_reuses_four_domains_and_ten_claims() {
    let join = joined();
    let mut checkpoints = Vec::new();
    check_bound_annular_table(join.embedding(), |point| {
        checkpoints.push(point);
        false
    })
    .unwrap();
    assert_eq!(checkpoints, (0..=12).collect::<Vec<_>>());
    assert!(!join.projection().is_valid_body());
}
#[test]
fn every_table_checkpoint_cancels_then_independent_reuse_succeeds() {
    let join = joined();
    for stop in 0..=12 {
        let failure =
            check_bound_annular_table(join.embedding(), |point| point == stop).unwrap_err();
        assert!(
            matches!(failure, AnnularTableFailure::Cancelled { completed } if completed == stop)
        );
    }
    check_bound_annular_table(join.embedding(), |_| false).unwrap();
}
#[test]
fn row_order_is_not_identity_but_duplicate_and_omitted_pairs_are_refused() {
    let join = joined();
    let evidence = join.embedding();
    let mut rows = evidence.incidences().to_vec();
    rows.reverse();
    checks::rows(evidence.domains(), &rows, &mut |_| false).unwrap();
    rows[0] = rows[1].clone();
    assert!(matches!(
        checks::rows(evidence.domains(), &rows, &mut |_| false),
        Err(AnnularTableFailure::Coverage(_))
    ));
    assert!(matches!(
        checks::rows(evidence.domains(), &rows[..9], &mut |_| false),
        Err(AnnularTableFailure::Coverage(_))
    ));
}
#[test]
fn a_mismatched_pair_or_changed_geometric_conclusion_is_never_an_exact_pass() {
    let join = joined();
    let evidence = join.embedding();
    let mut rows = evidence.incidences().to_vec();
    rows[0].conclusion = AnnularIncidenceConclusion::DisjointCaps;
    assert!(matches!(
        checks::rows(evidence.domains(), &rows, &mut |_| false),
        Err(AnnularTableFailure::Correspondence { .. })
    ));
    rows[0].first = rows[1].first.clone();
    assert!(matches!(
        checks::rows(evidence.domains(), &rows, &mut |_| false),
        Err(AnnularTableFailure::Coverage(_))
    ));
}
#[test]
fn duplicate_domain_and_changed_trim_are_refused_with_complete_operands() {
    let join = joined();
    let expected = join.embedding().domains();
    let mut actual = copy_domains(expected);
    actual.inner = actual.outer.clone();
    assert!(matches!(
        checks::domains(&actual, expected),
        Err(AnnularTableFailure::DomainMultiplicity { .. })
    ));
    let mut actual = copy_domains(expected);
    actual.first.trims.clear();
    assert!(matches!(
        checks::domains(&actual, expected),
        Err(AnnularTableFailure::DomainCorrespondence { .. })
    ));
}
fn copy_domains(value: &AnnularDomains) -> AnnularDomains {
    AnnularDomains {
        first: value.first.clone(),
        last: value.last.clone(),
        outer: value.outer.clone(),
        inner: value.inner.clone(),
    }
}
#[test]
fn table_failure_contains_only_owned_thread_safe_operands() {
    static_assertions::assert_impl_all!(AnnularTableFailure: Send, Sync);
}
