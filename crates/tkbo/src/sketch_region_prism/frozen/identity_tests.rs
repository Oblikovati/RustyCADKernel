// SPDX-License-Identifier: GPL-2.0-only
//! Authored identity is a required input; ordering changes never manufacture new identity.
use super::{tests::*, *};

#[test]
fn missing_duplicate_and_colliding_use_bindings_refuse_capture() {
    for defect in 0..3 {
        let (source, receipt) = authored_receipt();
        let mut bindings = authored_bindings(&receipt, &source);
        corrupt_directed_binding(&mut bindings, defect);
        let result = freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress);
        assert!(matches!(
            result,
            PrismFreezeOutcome::Incomplete(PrismFreezeFailure {
                cause: PrismFreezeCause::Identity { .. },
                ..
            })
        ));
    }
}
fn corrupt_directed_binding(bindings: &mut PrismPublicationBindings, defect: usize) {
    match defect {
        0 => {
            bindings.directed.pop();
        }
        1 => bindings.directed.push(PrismDirectedBinding {
            actual: bindings.directed[0].actual.clone(),
            authored_use: AuthoredPrismId(999),
        }),
        2 => bindings.directed[0].authored_use = AuthoredPrismId(0),
        _ => unreachable!(),
    }
}
#[test]
fn source_collision_and_selected_region_mismatch_refuse_capture() {
    for mismatch in [false, true] {
        let (source, receipt) = authored_receipt();
        let mut bindings = authored_bindings(&receipt, &source);
        if mismatch {
            bindings.selected_region = AuthoredPrismId(999);
        } else {
            bindings
                .sources
                .iter_mut()
                .for_each(|b| b.member = AuthoredPrismId(701));
        }
        let result = freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress);
        assert!(matches!(
            result,
            PrismFreezeOutcome::Incomplete(PrismFreezeFailure {
                cause: PrismFreezeCause::Identity { .. },
                ..
            })
        ));
    }
}
#[test]
fn authored_binding_input_order_does_not_change_owned_bytes() {
    let (source, receipt) = authored_receipt();
    let mut bindings = authored_bindings(&receipt, &source);
    let first =
        captured(freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress)).replay_bytes();
    bindings.originals.reverse();
    bindings.sources.reverse();
    bindings.directed.reverse();
    let second =
        captured(freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress)).replay_bytes();
    assert_eq!(first, second);
}
#[test]
fn every_directed_use_has_explicit_parent_and_one_incidence_binding() {
    let projection = fresh_projection();
    let uses = projection.uses();
    assert!(uses.iter().all(|item| {
        item.children.iter().all(|child| {
            uses.iter().any(|actual| &actual.key == child)
                && child.0[..child.0.len() - 1] == item.key.0
        })
    }));
    let coedges = &projection.incidence().coedges;
    let distinct: std::collections::BTreeSet<_> = coedges.iter().map(|c| &c.occurrence).collect();
    assert_eq!(distinct.len(), 6);
    for (_, first, last) in &projection.incidence().endpoints {
        assert!(projection.nodes().iter().any(|n| n.key == *first));
        assert!(projection.nodes().iter().any(|n| n.key == *last));
    }
}
