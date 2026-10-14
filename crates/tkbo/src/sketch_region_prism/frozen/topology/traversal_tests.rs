// SPDX-License-Identifier: GPL-2.0-only
use super::super::{tests::fresh_projection, traversal_tests::ordered_projection};
use super::{tests::*, *};
#[test]
fn containment_v1_and_v2_without_order_are_incomplete_not_closed() {
    let legacy = fresh_projection();
    assert!(matches!(
        certify_frozen_topology(&legacy, &mut NamedTopologyProgress::default()),
        FrozenTopologyOutcome::Incomplete(_)
    ));
    let mut source = ordered_projection();
    source.wire_traversals = None;
    assert!(matches!(
        certify_frozen_topology(&source, &mut NamedTopologyProgress::default()),
        FrozenTopologyOutcome::Incomplete(_)
    ));
}
#[test]
fn missing_duplicate_foreign_and_empty_traversal_rows_fail_bijection() {
    for defect in 0..5 {
        let mut source = ordered_projection();
        let rows = source.wire_traversals.as_mut().unwrap();
        match defect {
            0 => {
                rows.pop();
            }
            1 => rows.push(rows[0].clone()),
            2 => {
                let duplicate = rows[0].coedges[0].clone();
                rows[0].coedges.push(duplicate);
            }
            3 => rows[0].coedges[0] = rows[1].coedges[0].clone(),
            _ => rows[0].coedges.clear(),
        }
        rejected(&source, TopologyInvariant::ClosedContainment);
    }
}
#[test]
fn source_insertion_order_cannot_impersonate_observed_graph_traversal() {
    let mut source = ordered_projection();
    let side = source
        .wire_traversals
        .as_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row.coedges.len() == 4)
        .unwrap();
    let wire = source
        .uses
        .iter()
        .find(|row| row.key == side.occurrence)
        .unwrap();
    side.coedges.clone_from(&wire.children);
    rejected(&source, TopologyInvariant::CyclicWires);
}
#[test]
fn missing_or_extra_same_face_seam_use_never_gets_a_certificate() {
    for extra in [false, true] {
        let mut source = ordered_projection();
        alter_seam_traversal(&mut source, extra);
        rejected(&source, TopologyInvariant::ClosedContainment);
    }
}

fn alter_seam_traversal(source: &mut super::super::FrozenPrismProjection, extra: bool) {
    let side = source
        .wire_traversals
        .as_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row.coedges.len() == 4)
        .unwrap();
    let seam = side
        .coedges
        .iter()
        .position(|key| key.0.last().unwrap().0.source.0 == 704)
        .unwrap();
    if extra {
        side.coedges.push(side.coedges[seam].clone());
    } else {
        side.coedges.remove(seam);
    }
}

#[test]
fn wrong_import_behavior_and_excess_order_work_are_explicitly_incomplete() {
    use rustycad_tkbrep::brep_graph_inc::populate::PCurveImportBehavior;
    for excess in [false, true] {
        let mut source = ordered_projection();
        if excess {
            source.wire_traversals.as_mut().unwrap()[0]
                .coedges
                .resize(4097, source.uses[0].key.clone());
        } else {
            source.graph.import_behavior = PCurveImportBehavior::default();
        }
        assert!(matches!(
            certify_frozen_topology(&source, &mut NamedTopologyProgress::default()),
            FrozenTopologyOutcome::Incomplete(_)
        ));
    }
}
