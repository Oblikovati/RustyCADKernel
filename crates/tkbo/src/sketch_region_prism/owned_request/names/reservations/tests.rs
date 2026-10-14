// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::owned_request::{resolver::fixture::*, test_fixture};

#[test]
fn actual_history_and_output_apply_73_reservations_and_explicitly_exclude_28() {
    let invoked = invoked_prism_fixture(test_fixture::request());
    let names = body_map::issue_resolved_prism_names(&invoked).unwrap();
    let coverage = inspect_reservations(&invoked, &names).unwrap();
    assert_eq!(coverage.rows().len(), 101);
    assert_eq!(
        coverage
            .rows()
            .iter()
            .filter(|r| matches!(r.status, PrismReservationStatus::Applied { .. }))
            .count(),
        73
    );
    assert!(coverage.binds(&names));
    assert_literal_counts(coverage.rows());
    assert_literal_inapplicable(coverage.rows());
}
fn assert_literal_counts(rows: &[PrismReservationRow]) {
    for (token, expected) in [
        (515, 1),
        (516, 2),
        (1058, 1),
        (1059, 1),
        (2332, 3),
        (2333, 3),
        (2309, 6),
        (2310, 6),
    ] {
        assert_named_count(rows, token, expected);
    }
}
fn assert_literal_inapplicable(rows: &[PrismReservationRow]) {
    let actual: Vec<_> = rows
        .iter()
        .filter(|r| matches!(r.status, PrismReservationStatus::Inapplicable(_)))
        .map(|r| r.slot.token().value())
        .collect();
    let expected = [
        1032, 1033, 1034, 1035, 1036, 1037, 1038, 1039, 1288, 1289, 1290, 1291, 1292, 1293, 1294,
        1295, 1545, 1547, 1549, 1551, 1800, 1802, 1804, 1806, 2057, 2059, 2061, 2063,
    ];
    assert_eq!(actual, expected);
}
#[test]
fn applicability_refuses_missing_required_or_unexpected_applied_reservations() {
    let invoked = invoked_prism_fixture(test_fixture::request());
    let names = body_map::issue_resolved_prism_names(&invoked).unwrap();
    let original = capture::applications(&invoked, &names).unwrap();
    assert_missing(invoked.plan(), &original);
    let mut extra = original;
    let slot = invoked.plan().reverse(1032).unwrap().slot;
    extra.push(slot);
    assert_eq!(
        classification::classify(invoked.plan(), &extra),
        Err(PrismReservationCause::InapplicableApplied { slot, actual: 1 })
    );
}
#[test]
fn coverage_refuses_foreign_caller_map_even_when_publication_bytes_match() {
    let invoked = invoked_prism_fixture(test_fixture::request());
    let mut request = test_fixture::request();
    request.operation = CallerOperationId(9);
    let foreign = resolved_prism_fixture(request);
    assert_eq!(
        invoked.result().replay_bytes(),
        foreign.publication.replay_bytes()
    );
    assert_eq!(
        inspect_reservations(&invoked, &foreign.names),
        Err(PrismReservationFailure {
            operation: CallerOperationId(257),
            cause: PrismReservationCause::ForeignMap
        })
    );
}
#[test]
fn applicability_replay_is_owned_send_sync_and_changes_with_caller_dictionary() {
    fn check<T: Send + Sync>() {}
    check::<PrismReservationCoverage>();
    check::<PrismReservationCause>();
    check::<PrismReservationFailure>();
    let invoked = invoked_prism_fixture(test_fixture::request());
    let names = body_map::issue_resolved_prism_names(&invoked).unwrap();
    let first = inspect_reservations(&invoked, &names).unwrap();
    let second = inspect_reservations(&invoked, &names).unwrap();
    assert_eq!(first.replay_bytes(), second.replay_bytes());
    let mut request = test_fixture::request();
    request.operation = CallerOperationId(9);
    let foreign = resolved_prism_fixture(request);
    assert!(!first.binds(&foreign.names));
}
#[test]
fn unsupported_dictionary_and_unknown_application_never_become_inapplicable() {
    let invoked = invoked_prism_fixture(test_fixture::request());
    let v1 = SingleBoundaryNamePlan::v1(invoked.plan().identities(), invoked.plan().selection())
        .unwrap();
    assert_eq!(
        classification::classify(&v1, &[]),
        Err(PrismReservationCause::UnsupportedVersion {
            actual: 1,
            expected: 2
        })
    );
    let slot = SemanticNameSlot {
        stage: SemanticNameStage::Original,
        role: SemanticNameRole::Solid,
    };
    assert_eq!(
        classification::classify(invoked.plan(), &[slot]),
        Err(PrismReservationCause::UnknownToken { actual: 528 })
    );
}

fn assert_named_count(rows: &[PrismReservationRow], token: u64, expected: usize) {
    let row = rows
        .iter()
        .find(|r| r.slot.token().value() == token)
        .unwrap();
    assert_eq!(
        row.status,
        PrismReservationStatus::Applied {
            named_fields: expected
        }
    );
}

fn assert_missing(plan: &SingleBoundaryNamePlan, original: &[SemanticNameSlot]) {
    let slot = plan.reverse(516).unwrap().slot;
    let missing: Vec<_> = original
        .iter()
        .copied()
        .filter(|value| *value != slot)
        .collect();
    assert_eq!(
        classification::classify(plan, &missing),
        Err(PrismReservationCause::MissingApplication { slot })
    );
}
