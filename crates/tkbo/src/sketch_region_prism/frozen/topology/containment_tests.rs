// SPDX-License-Identifier: GPL-2.0-only
use super::super::{traversal_tests::ordered_projection as fresh_projection, *};
use super::{tests::*, *};

#[test]
fn empty_projection_duplicate_and_orphan_definitions_refuse_certificate() {
    for defect in 0..3 {
        let mut source = fresh_projection();
        match defect {
            0 => {
                source.nodes.clear();
                source.uses.clear();
            }
            1 => source.nodes.push(source.nodes[0].clone()),
            _ => {
                let mut orphan = source.nodes[0].clone();
                orphan.key.source = AuthoredPrismId(9999);
                source.nodes.push(orphan);
            }
        }
        rejected(&source, TopologyInvariant::ClosedContainment);
    }
}
#[test]
fn dangling_duplicate_or_absent_container_child_is_exact_failure() {
    for defect in 0..3 {
        let mut source = fresh_projection();
        let root = source
            .uses
            .iter_mut()
            .find(|row| row.key.0.len() == 1)
            .unwrap();
        match defect {
            0 => root.children[0].0.last_mut().unwrap().1 = AuthoredPrismId(9999),
            1 => root.children.push(root.children[0].clone()),
            _ => root.children.clear(),
        }
        rejected(&source, TopologyInvariant::ClosedContainment);
    }
}
#[test]
fn duplicate_occurrence_empty_ancestry_and_wrong_depth_are_typed_failures() {
    for defect in 0..3 {
        let mut source = fresh_projection();
        match defect {
            0 => source.uses.push(source.uses[0].clone()),
            1 => source.uses[0].key.0.clear(),
            _ => {
                source.uses[0].key.0.pop();
            }
        }
        rejected(&source, TopologyInvariant::ClosedContainment);
    }
}
#[test]
fn coedge_binding_direction_and_missing_rows_are_checked_before_topology() {
    for defect in 0..4 {
        let mut source = fresh_projection();
        match defect {
            0 => source.graph.coedges[0].reversed = !source.graph.coedges[0].reversed,
            1 => source.graph.coedges[0].face.source = AuthoredPrismId(9999),
            2 => {
                source.graph.coedges.pop();
            }
            _ => source.graph.coedges.push(source.graph.coedges[0].clone()),
        }
        rejected(&source, TopologyInvariant::ClosedContainment);
    }
}
#[test]
fn missing_duplicate_wrong_kind_and_unbound_endpoints_are_typed_failures() {
    for defect in 0..4 {
        let mut source = fresh_projection();
        match defect {
            0 => {
                source.graph.endpoints.pop();
            }
            1 => source.graph.endpoints.push(source.graph.endpoints[0]),
            2 => source.graph.endpoints[0].1.kind = 6,
            _ => source.graph.endpoints[0].2.source = AuthoredPrismId(9999),
        }
        rejected(&source, TopologyInvariant::ClosedContainment);
    }
}
#[test]
fn canonical_vertex_use_roles_and_reported_counts_cannot_disagree() {
    for defect in 0..2 {
        let mut source = fresh_projection();
        if defect == 0 {
            let vertex = source
                .uses
                .iter_mut()
                .find(|row| row.key.0.len() == 6)
                .unwrap();
            vertex.orientation = super::combinatorial_tests::reverse(vertex.orientation);
        } else {
            source.graph.counts[2] += 1;
        }
        rejected(&source, TopologyInvariant::ClosedContainment);
    }
}
