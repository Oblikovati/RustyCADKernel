// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
pub(super) fn ordered_projection() -> FrozenPrismProjection {
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    captured(freeze_prism_with_behavior(
        &receipt,
        &bindings,
        &mut NamedFreezeProgress,
        PrismFreezeBehavior::OrderedTraversalV2,
    ))
}
#[test]
fn traversal_v2_preserves_v1_containment_and_exact_legacy_replay_selection() {
    let legacy = fresh_projection();
    let mut ordered = ordered_projection();
    assert_eq!(ordered.wire_traversals.as_ref().unwrap().len(), 3);
    assert_eq!(
        ordered
            .uses
            .iter()
            .map(|row| &row.children)
            .collect::<Vec<_>>(),
        legacy
            .uses
            .iter()
            .map(|row| &row.children)
            .collect::<Vec<_>>()
    );
    ordered.version = 1;
    ordered.wire_traversals = None;
    assert_eq!(ordered.replay_bytes(), legacy.replay_bytes());
}
#[test]
fn graph_traversal_differs_from_side_insertion_order_but_binds_every_use_once() {
    let source = ordered_projection();
    let row = source
        .wire_traversals
        .as_ref()
        .unwrap()
        .iter()
        .find(|row| row.coedges.len() == 4)
        .unwrap();
    let wire = source
        .uses
        .iter()
        .find(|wire| wire.key == row.occurrence)
        .unwrap();
    assert_ne!(wire.children, row.coedges);
    let actual: std::collections::BTreeSet<_> = row.coedges.iter().collect();
    let expected: std::collections::BTreeSet<_> = wire.children.iter().collect();
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), row.coedges.len());
}

#[test]
fn ordered_capture_rebinds_actual_wire_containment_after_commit_callback() {
    use super::mutation_tests::{ContainmentEdit, remove_child_progress};
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    let mut progress = remove_child_progress(
        &receipt,
        rustycad_tkg3d::top_abs::ShapeEnum::Wire,
        FreezePhase::Commit,
        ContainmentEdit::Reorder,
    );
    let result = freeze_prism_with_behavior(
        &receipt,
        &bindings,
        &mut progress,
        PrismFreezeBehavior::OrderedTraversalV2,
    );
    assert_stale_order(result);
}
fn assert_stale_order(result: PrismFreezeOutcome) {
    assert!(matches!(
        result,
        PrismFreezeOutcome::Incomplete(PrismFreezeFailure {
            cause: PrismFreezeCause::Identity { .. } | PrismFreezeCause::StaleCapture { .. },
            ..
        })
    ));
}

#[test]
fn versioned_capture_types_are_send_sync_and_own_traversal_values() {
    static_assertions::assert_impl_all!(PrismFreezeBehavior: Send, Sync);
    static_assertions::assert_impl_all!(super::traversal::FrozenWireTraversal: Send, Sync);
    let original = ordered_projection();
    let bytes = original.replay_bytes();
    let mut changed = original.clone();
    changed.wire_traversals.as_mut().unwrap()[0].coedges.clear();
    assert_eq!(original.replay_bytes(), bytes);
    assert_ne!(changed.replay_bytes(), bytes);
}

#[test]
fn borrowed_traversal_view_preserves_capture_identity_and_all_replay_bytes() {
    let legacy = fresh_projection();
    let ordered = ordered_projection();
    let before = ordered.replay_bytes();
    assert!(legacy.wire_traversals().is_none());
    let borrowed = ordered.wire_traversals().unwrap();
    assert_eq!(borrowed.len(), 3);
    assert_eq!(
        borrowed.iter().map(|row| row.coedges.len()).sum::<usize>(),
        6
    );
    assert!(std::ptr::eq(
        borrowed,
        ordered.wire_traversals.as_deref().unwrap()
    ));
    assert_eq!(ordered.replay_bytes(), before);
}
