// SPDX-License-Identifier: GPL-2.0-only
//! Isolated malformed owned-table tests; these are not constructor-admitted joins.
use super::super::tests::fresh_projection;
use super::*;

#[test]
fn isolated_named_direction_uses_exact_cumulative_context() {
    let projection = fresh_projection();
    for row in projection.uses() {
        assert_eq!(
            naming::actual_direction(&projection, &row.key, row.orientation)
                .ok()
                .unwrap(),
            row.orientation
        );
        let wrong = rustycad_tkg3d::top_abs::reverse(row.orientation);
        assert!(
            matches!(naming::actual_direction(&projection, &row.key, wrong),
            Err(JoinedPrismOrientationCause::Direction { actual, expected })
            if actual == row.orientation && expected == wrong)
        );
    }
}

#[test]
fn isolated_missing_or_duplicate_named_uses_refuse_unique_lookup() {
    let mut projection = fresh_projection();
    let removed = projection.uses.pop().unwrap();
    require_use_count(&projection, &removed, 0);
    projection.uses.push(removed.clone());
    projection.uses.push(removed.clone());
    require_use_count(&projection, &removed, 2);
}

#[test]
fn isolated_interval_matching_keeps_zero_sign_and_exact_endpoint_bits() {
    assert!(naming::interval([0., 1.], [0., 1.]).is_ok());
    for actual in [[-0., 1.], [0., f64::from_bits(1_f64.to_bits() + 1)]] {
        assert!(matches!(naming::interval(actual, [0., 1.]),
            Err(JoinedPrismOrientationCause::Interval { actual: observed, expected })
            if observed == actual.map(f64::to_bits) && expected == [0., 1.].map(f64::to_bits)));
    }
}

#[test]
fn isolated_parent_counts_preserve_multiplicity() {
    assert!(naming::parent_count(2, 2).is_ok());
    assert!(matches!(
        naming::parent_count(2, 1),
        Err(JoinedPrismOrientationCause::ParentMultiplicity {
            actual: 2,
            expected: 1
        })
    ));
}

#[test]
fn joined_orientation_owned_types_are_send_sync() {
    static_assertions::assert_impl_all!(JoinedPrismOrientationCertificate: Send, Sync);
    static_assertions::assert_impl_all!(JoinedPrismOrientationOptions: Send, Sync);
    static_assertions::assert_impl_all!(JoinedOrientationEvidenceKind: Send, Sync);
    static_assertions::assert_impl_all!(JoinedFaceDirection: Send, Sync);
    static_assertions::assert_impl_all!(JoinedTrimDirection: Send, Sync);
    static_assertions::assert_impl_all!(JoinedPrismOrientationFailure: Send, Sync);
    static_assertions::assert_impl_all!(JoinedPrismOrientationCause: Send, Sync);
    static_assertions::assert_impl_all!(JoinedPrismOrientationPhase: Send, Sync);
}

fn require_use_count(projection: &FrozenPrismProjection, row: &FrozenPrismUse, expected: usize) {
    assert!(
        matches!(naming::actual_direction(projection, &row.key, row.orientation),
        Err(JoinedPrismOrientationCause::Count { actual, expected: 1, .. })
        if actual == expected)
    );
}

#[test]
fn isolated_distinct_names_do_not_collapse_two_authored_uses() {
    let projection = fresh_projection();
    let a = &projection.uses()[0].key;
    let b = &projection.uses()[1].key;
    assert!(naming::distinct_uses(vec![a, b], 2).is_ok());
    assert!(matches!(
        naming::distinct_uses(vec![a, a], 2),
        Err(JoinedPrismOrientationCause::Count {
            actual: 1,
            expected: 2,
            ..
        })
    ));
}

#[test]
fn isolated_extra_duplicate_use_cannot_hide_behind_expected_unique_count() {
    let projection = fresh_projection();
    let a = &projection.uses()[0].key;
    let b = &projection.uses()[1].key;
    assert!(matches!(
        naming::distinct_uses(vec![a, a, b], 2),
        Err(JoinedPrismOrientationCause::Count {
            actual: 3,
            expected: 2,
            ..
        })
    ));
}
