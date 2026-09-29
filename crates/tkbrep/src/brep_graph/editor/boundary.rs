// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of EditorView::BoundaryIssue, CommitMutation,
// ValidateMutationBoundary, countActiveByKind and cachedActiveByKind of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.hxx and BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The mutation-boundary checks of the editor.
use super::helpers::{is_valid_occurrence_child_kind, kind_name};
use super::{EditorState, EditorView};
use crate::brep_graph::*;
use crate::brep_graph_inc::Storage;
use rustycad_tkernel::tcollection::AsciiString;

/// `EditorView::BoundaryIssue`: one issue found by `ValidateMutationBoundary`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoundaryIssue {
    /// `NodeId`: the node concerned (the invalid id of the kind for a count
    /// mismatch, the invalid id for a relation inconsistency).
    pub node_id: NodeId,
    /// `Description`.
    pub description: AsciiString,
}

pub(crate) const BOUNDARY_KINDS: [NodeKind; 11] = [
    NodeKind::VERTEX,
    NodeKind::EDGE,
    NodeKind::CO_EDGE,
    NodeKind::WIRE,
    NodeKind::FACE,
    NodeKind::SHELL,
    NodeKind::SOLID,
    NodeKind::COMPOUND,
    NodeKind::COMP_SOLID,
    NodeKind::PRODUCT,
    NodeKind::OCCURRENCE,
];

impl EditorView<'_> {
    /// `CommitMutation()`: closes a batch of mutations. Natively its only statement is
    /// a debug-only assertion on `ValidateMutationBoundary`, which the release build
    /// (the oracle) compiles out without evaluating, so it has no effect; call
    /// [`validate_mutation_boundary`](Self::validate_mutation_boundary) to check.
    pub fn commit_mutation(&mut self) {}

    /// `ValidateMutationBoundary(theIssues)`.
    pub fn validate_mutation_boundary(&self, issues: Option<&mut Vec<BoundaryIssue>>) -> bool {
        self.state().validate_mutation_boundary(issues)
    }
}

impl EditorState<'_> {
    /// `ValidateMutationBoundary(theIssues)`: the relations agree with the records,
    /// every cached active count equals the count of active slots, and every active
    /// occurrence's valid child is a product or a topology node; the issues are
    /// appended to `issues` when given.
    pub fn validate_mutation_boundary(self, mut issues: Option<&mut Vec<BoundaryIssue>>) -> bool {
        let storage = self.graph.inc_storage();
        let mut report = |issue: BoundaryIssue| {
            if let Some(issues) = issues.as_deref_mut() {
                issues.push(issue);
            }
        };
        let mut is_valid = true;
        if !storage.validate_relations() {
            is_valid = false;
            report(BoundaryIssue {
                node_id: NodeId::default(),
                description: AsciiString::from("Mutation boundary relation inconsistency"),
            });
        }
        for kind in BOUNDARY_KINDS {
            let (cached, actual) = (
                cached_active(storage, kind),
                actual_active(self.graph, kind),
            );
            if cached != actual {
                is_valid = false;
                report(count_issue(kind, cached, actual));
            }
        }
        for issue in occurrence_kind_issues(storage) {
            is_valid = false;
            report(issue);
        }
        is_valid
    }
}

fn count_issue(kind: NodeKind, cached: i32, actual: i32) -> BoundaryIssue {
    let text = format!(
        "Mutation boundary active count mismatch for {}: cached={cached} actual={actual}",
        kind_name(kind)
    );
    BoundaryIssue {
        node_id: NodeId::invalid(kind),
        description: AsciiString::from(text.as_str()),
    }
}

// The active occurrences whose valid child has a kind an occurrence cannot place.
fn occurrence_kind_issues(storage: &Storage) -> Vec<BoundaryIssue> {
    (0..storage.nb_occurrences())
        .map(OccurrenceId::new)
        .filter(|occurrence| !storage.is_removed(*occurrence))
        .filter_map(|occurrence| {
            let child = storage.occurrence(occurrence).child_node_id;
            if !child.is_valid() || is_valid_occurrence_child_kind(child.kind) {
                return None;
            }
            let text = format!(
                "Storage occurrence child kind invalid at occurrence idx={}: child kind={}",
                occurrence.index as i32, child.kind.0
            );
            Some(BoundaryIssue {
                node_id: occurrence.into(),
                description: AsciiString::from(text.as_str()),
            })
        })
        .collect()
}

/// `cachedActiveByKind(theStorage, theKind)`: the stored active count.
pub(crate) fn cached_active(storage: &Storage, kind: NodeKind) -> i32 {
    let counts = storage.active_counts();
    let count = match kind {
        NodeKind::VERTEX => counts.nb_vertices,
        NodeKind::EDGE => counts.nb_edges,
        NodeKind::CO_EDGE => counts.nb_co_edges,
        NodeKind::WIRE => counts.nb_wires,
        NodeKind::FACE => counts.nb_faces,
        NodeKind::SHELL => counts.nb_shells,
        NodeKind::SOLID => counts.nb_solids,
        NodeKind::COMPOUND => counts.nb_compounds,
        NodeKind::COMP_SOLID => counts.nb_comp_solids,
        NodeKind::PRODUCT => counts.nb_products,
        NodeKind::OCCURRENCE => counts.nb_occurrences,
        _ => 0,
    };
    count as i32
}

/// `countActiveByKind(theGraph, theKind)`: the slots a `BRepGraph_Iterator` visits.
pub(crate) fn actual_active(graph: &BRepGraph, kind: NodeKind) -> i32 {
    let generic = graph.topo().generic();
    (0..generic.nb(kind))
        .filter(|index| !NodeId::new(kind, *index).is_removed(graph))
        .count() as i32
}
