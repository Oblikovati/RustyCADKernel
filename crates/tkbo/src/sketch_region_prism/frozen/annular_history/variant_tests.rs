// SPDX-License-Identifier: GPL-2.0-only
//! Stored/traversed reversal, reflection, negative sweep and request-row order use the same
//! pipeline; the closed-form census and authored source roles are invariant across them.
use super::super::annular_orientation::fixture::Variant;
use super::fixture::AnnularHistoryCase;
use super::operation_tests::inspected;
use crate::sketch_region::RegionSourceId;

fn variants() -> Vec<(Variant, bool)> {
    let mut result = Vec::new();
    for stored_reverse in [[false, false], [true, false], [false, true]] {
        for traversed_reverse in [[false, false], [true, true]] {
            let mut variant = Variant::default();
            (variant.stored_reverse, variant.traversed_reverse) =
                (stored_reverse, traversed_reverse);
            result.push((variant, false));
        }
    }
    let (mut reflected, mut negative) = (Variant::default(), Variant::default());
    (reflected.reflected, negative.negative_sweep) = (true, true);
    let defaults = (Variant::default(), true);
    result.extend([(reflected, false), (negative, false), defaults]);
    result
}
#[test]
fn every_variant_proves_six_source_keyed_continuations_and_two_retirements() {
    for (variant, reversed) in variants() {
        let history = inspected(&AnnularHistoryCase::variant(variant, reversed));
        assert_eq!(history.continued().len(), 6);
        assert!(history.deleted_originals().is_empty());
        let retired: Vec<_> = history.retired_wires().iter().map(|r| r.source).collect();
        assert_eq!(retired, [RegionSourceId(101), RegionSourceId(202)]);
        for row in history.continued() {
            let expected = if row.source == RegionSourceId(101) {
                303
            } else {
                302
            };
            assert_eq!(row.trace.rejected_candidate, expected);
        }
        assert_eq!(history.sweep_queries().len(), 9);
    }
}
/// Request-row order is not identity: reordered sources produce the same authored replay.
#[test]
fn reordered_request_rows_replay_identical_history() {
    let forward = inspected(&AnnularHistoryCase::variant(Variant::default(), false));
    let reversed = inspected(&AnnularHistoryCase::variant(Variant::default(), true));
    assert_eq!(forward.replay_bytes(), reversed.replay_bytes());
}
