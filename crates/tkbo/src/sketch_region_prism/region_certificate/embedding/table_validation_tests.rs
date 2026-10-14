// SPDX-License-Identifier: GPL-2.0-only
//! Mutations target typed proof correspondence, not independent geometric claims.
use super::{
    table_validation::{check_domains, check_rows},
    *,
};
use static_assertions::assert_impl_all;

fn owned_theorem() -> PrismEmbeddingEvidence {
    let (bridge, shapes) = super::super::tests::fixture();
    let receipt = super::super::tests::run(&bridge, &shapes);
    super::tests::embedded(&bridge, &shapes, &receipt)
}

#[test]
fn embedding_table_validation_keeps_all_named_self_and_unordered_pair_rows() {
    let report = owned_theorem();
    check_prism_embedding_table(&report, |_| false).unwrap();
    let mut rows = report.incidences().clone();
    rows.reverse();
    for row in &mut rows {
        std::mem::swap(&mut row.first, &mut row.second);
    }
    check_rows(report.domains(), &rows, |_| false).unwrap();
}

#[test]
fn embedding_table_validation_refuses_missing_and_extra_pair_rows() {
    let report = owned_theorem();
    let mut rows = report.incidences().to_vec();
    rows.pop();
    assert_row_count_refused(&report, &rows, 5);
    rows.extend([rows[0].clone(), rows[0].clone()]);
    assert_row_count_refused(&report, &rows, 7);
}

fn assert_row_count_refused(
    report: &PrismEmbeddingEvidence,
    rows: &[PrismFaceIncidence],
    count: usize,
) {
    assert!(matches!(check_rows(report.domains(), rows, |_| false),
        Err(PrismEmbeddingTableFailure::RowCount { actual, expected: 6 }) if actual == count));
    assert_eq!(rows.len(), count);
}

#[test]
fn embedding_table_validation_refuses_duplicate_pair_rows() {
    let report = owned_theorem();
    let mut rows = report.incidences().clone();
    rows[5] = rows[0].clone();
    assert!(matches!(
        check_rows(report.domains(), &rows, |_| false),
        Err(PrismEmbeddingTableFailure::PairMultiplicity {
            actual: 2,
            expected: 1,
            ..
        })
    ));
}

#[test]
fn embedding_table_validation_refuses_wrong_domain_and_wrong_conclusion() {
    let report = owned_theorem();
    let mut rows = report.incidences().clone();
    rows[0].first.face.topology_address += 1;
    assert_correspondence_refused(&report, &rows);
    let mut rows = report.incidences().clone();
    rows[0].conclusion = PrismIncidenceConclusion::Disjoint;
    assert_correspondence_refused(&report, &rows);
}

fn assert_correspondence_refused(report: &PrismEmbeddingEvidence, rows: &[PrismFaceIncidence]) {
    let before = format!("{rows:?}");
    assert!(matches!(
        check_rows(report.domains(), rows, |_| false),
        Err(PrismEmbeddingTableFailure::Correspondence { .. })
    ));
    assert_eq!(format!("{rows:?}"), before);
}

#[test]
fn embedding_table_validation_keeps_seam_roles() {
    let report = owned_theorem();
    let mut rows = report.incidences().clone();
    let PrismIncidenceConclusion::InjectiveModuloSeam { start, end, .. } = &mut rows[2].conclusion
    else {
        panic!("expected owned seam")
    };
    std::mem::swap(start, end);
    assert_correspondence_refused(&report, &rows);
}

#[test]
fn embedding_table_validation_keeps_seam_range_and_parent_multiplicity() {
    let report = owned_theorem();
    let mut rows = report.incidences().clone();
    let PrismIncidenceConclusion::InjectiveModuloSeam {
        normalized_sweep, ..
    } = &mut rows[2].conclusion
    else {
        panic!("expected owned seam")
    };
    *normalized_sweep = [0., 2.];
    assert_correspondence_refused(&report, &rows);
    let mut rows = report.incidences().clone();
    let PrismIncidenceConclusion::InjectiveModuloSeam { start, .. } = &mut rows[2].conclusion
    else {
        panic!("expected owned seam")
    };
    start.parents.push(start.parents[0].clone());
    assert_correspondence_refused(&report, &rows);
}

#[test]
fn embedding_table_validation_keeps_rim_authored_trim_and_native_domain() {
    let report = owned_theorem();
    let mut rows = report.incidences().clone();
    let PrismIncidenceConclusion::ExactlyRim { cap, .. } = &mut rows[4].conclusion else {
        panic!("expected owned first rim")
    };
    cap.interval = [0., 0.5];
    assert_correspondence_refused(&report, &rows);
    let mut rows = report.incidences().clone();
    let PrismIncidenceConclusion::ExactlyRim { native, .. } = &mut rows[4].conclusion else {
        panic!("expected owned first rim")
    };
    *native = [0., 0.5];
    assert_correspondence_refused(&report, &rows);
}

#[test]
fn embedding_table_validation_rejects_ambiguous_domains_and_missing_named_trim() {
    let report = owned_theorem();
    let mut domains = report.domains().clone();
    domains[1].role = domains[0].role;
    assert!(matches!(
        check_rows(&domains, report.incidences(), |_| false),
        Err(PrismEmbeddingTableFailure::Reconstruction(_))
    ));
    let mut domains = report.domains().clone();
    domains[2]
        .trims
        .retain(|trim| trim.role != PrismTrimRole::Start);
    assert!(matches!(
        check_rows(&domains, report.incidences(), |_| false),
        Err(PrismEmbeddingTableFailure::Reconstruction(_))
    ));
}

#[test]
fn embedding_table_validation_can_cancel_at_every_observed_checkpoint() {
    let report = owned_theorem();
    let mut observed = Vec::new();
    check_prism_embedding_table(&report, |step| {
        observed.push(step);
        false
    })
    .unwrap();
    assert_eq!(observed, (0..9).collect::<Vec<_>>());
    for requested in observed {
        assert!(
            matches!(check_prism_embedding_table(&report, |step| step == requested),
            Err(PrismEmbeddingTableFailure::Cancelled { completed }) if completed == requested)
        );
    }
}

#[test]
fn embedding_table_failure_has_owned_send_sync_scalar_and_context_payloads() {
    // This assertion does not replace the operation's ordinary-thread or sanitizer gates.
    assert_impl_all!(PrismEmbeddingTableFailure: Send, Sync);
}

#[test]
fn embedding_table_validation_binds_canonical_domains_and_all_trim_multiplicities() {
    let report = owned_theorem();
    let mut actual = report.domains().clone();
    actual.reverse();
    check_domains(&actual, report.domains()).unwrap();
    let repeated = actual[0].trims[0].clone();
    actual[0].trims.push(repeated);
    assert!(matches!(
        check_domains(&actual, report.domains()),
        Err(PrismEmbeddingTableFailure::DomainCorrespondence { .. })
    ));
}

#[test]
fn embedding_table_validation_binds_unique_canonical_domain_roles() {
    let report = owned_theorem();
    let mut actual = report.domains().clone();
    actual[1].role = actual[0].role;
    assert!(matches!(
        check_domains(&actual, report.domains()),
        Err(PrismEmbeddingTableFailure::DomainMultiplicity {
            actual: 2,
            expected: 1,
            ..
        })
    ));
}
