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
// Rust translation of the reference ownership, occurrence liveness and
// cross-container steps of BRepGraph_Validate::Perform in
// ModelingData/TKBRep/BRepGraph/BRepGraph_Validate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ownership checks that close the audit.
//!
//! Every live reference is a usage record of exactly one live parent slot. The seven
//! native loops that count the owning slots of each reference kind differ only in the
//! parent kind and the slot list, so they are one generic count over an
//! [`OwnedRef`] impl per reference kind.
use super::{Issue, Severity, report};
use crate::brep_graph::iterator::{CoEdgeIterator, OccurrenceIterator};
use crate::brep_graph::refs_iterator::RefIterator;
use crate::brep_graph::*;
use crate::brep_graph_inc::{
    ChildRef, FaceRef, OccurrenceRef, ShellRef, SolidRef, VertexRef, WireRef,
};
use std::collections::HashSet;

// A reference kind whose live parents hold it in their slots.
trait OwnedRef: RefTag {
    // The parent definition kind whose live definitions own the references.
    type Parent: NodeTag;
    // The reference name of the messages (`ChildRef`...).
    const REF_NAME: &'static str;
    // The number of the parent's slots that hold `reference`.
    fn owning_slots(
        graph: &BRepGraph,
        parent: TypedId<Self::Parent>,
        reference: TypedId<Self>,
    ) -> u32;
    // The node the reference names, which the issue reports.
    fn child(entry: &Self::Entry) -> NodeId;
}

// The count of `reference` in a relation list.
fn occurrences_in<T>(slots: &[TypedId<T>], reference: TypedId<T>) -> u32 {
    slots.iter().filter(|&&slot| slot == reference).count() as u32
}

macro_rules! owned_refs {
    ($($tag:ty, $entry:ty, $parent:ty, $name:literal, $relations:ident, $slots:ident,
       $child:ident;)*) => {$(
        impl OwnedRef for $tag {
            type Parent = $parent;
            const REF_NAME: &'static str = $name;
            fn owning_slots(
                graph: &BRepGraph,
                parent: TypedId<$parent>,
                reference: TypedId<Self>,
            ) -> u32 {
                occurrences_in(&graph.topo().$relations().relations(parent).$slots, reference)
            }
            fn child(entry: &$entry) -> NodeId {
                entry.$child.into()
            }
        }
    )*};
}

owned_refs! {
    ChildRefTag, ChildRef, CompoundTag, "ChildRef", compounds, child_ref_ids, child_node_id;
    SolidRefTag, SolidRef, CompSolidTag, "SolidRef", comp_solids, solid_ref_ids, child_solid_id;
    ShellRefTag, ShellRef, SolidTag, "ShellRef", solids, shell_ref_ids, child_shell_id;
    FaceRefTag, FaceRef, ShellTag, "FaceRef", shells, face_ref_ids, child_face_id;
    WireRefTag, WireRef, FaceTag, "WireRef", faces, wire_ref_ids, child_wire_id;
    OccurrenceRefTag, OccurrenceRef, ProductTag, "OccurrenceRef", products,
        occurrence_ref_ids, child_occurrence_id;
}

impl OwnedRef for VertexRefTag {
    type Parent = EdgeTag;
    const REF_NAME: &'static str = "VertexRef";
    // An edge holds its vertex references in its start and end slots.
    fn owning_slots(graph: &BRepGraph, parent: EdgeId, reference: VertexRefId) -> u32 {
        let edge = graph.topo().edges().definition(parent);
        u32::from(edge.start_vertex_ref_id == reference)
            + u32::from(edge.end_vertex_ref_id == reference)
    }
    fn child(entry: &VertexRef) -> NodeId {
        entry.child_vertex_id.into()
    }
}

// `appendOwnerIssue`: an orphan (no owner) or a shared (several owners) reference.
fn append_owner_issue(node: NodeId, owner_count: u32, kind: &str, issues: &mut Vec<Issue>) {
    if owner_count == 0 {
        let text = format!("Orphan {kind}: no live parent owns it");
        report(issues, Severity::Error, node, &text);
    } else if owner_count > 1 {
        let text = format!("Shared {kind}: more than one live parent slot owns it");
        report(issues, Severity::Error, node, &text);
    }
}

// Each live reference of the kind, full traversal, against every live parent.
fn check_owned_refs<T: OwnedRef>(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut references = RefIterator::<T, true>::new(graph);
    while references.more() {
        let reference = references.current_id();
        if !reference.is_removed(graph) {
            let owners = count_owners::<T>(graph, reference);
            append_owner_issue(T::child(references.current()), owners, T::REF_NAME, issues);
        }
        references.next();
    }
}

fn count_owners<T: OwnedRef>(graph: &BRepGraph, reference: TypedId<T>) -> u32 {
    live_ids::<T::Parent>(graph)
        .map(|parent| T::owning_slots(graph, parent, reference))
        .sum()
}

// The ids a `BRepGraph_Iterator` visits: the definitions that are not removed.
fn live_ids<T: NodeTag>(graph: &BRepGraph) -> impl Iterator<Item = TypedId<T>> + '_ {
    (0..graph.inc_storage().nb::<T>())
        .map(TypedId::new)
        .filter(move |id| !id.is_removed(graph))
}

/// Reference ownership: every live reference, coedge and occurrence is owned by
/// exactly one live parent slot, in the native kind order.
pub(super) fn check_ref_ownership(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    check_owned_refs::<ChildRefTag>(graph, issues);
    check_owned_refs::<SolidRefTag>(graph, issues);
    check_owned_refs::<ShellRefTag>(graph, issues);
    check_owned_refs::<FaceRefTag>(graph, issues);
    check_owned_refs::<WireRefTag>(graph, issues);
    check_co_edge_owners(graph, issues);
    check_owned_refs::<VertexRefTag>(graph, issues);
    check_owned_refs::<OccurrenceRefTag>(graph, issues);
    check_occurrence_owners(graph, issues);
}

// Every live coedge belongs to a live wire.
fn check_co_edge_owners(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut co_edges = CoEdgeIterator::new(graph);
    while co_edges.more() {
        let co_edge = co_edges.current_id();
        let wire = graph.topo().co_edges().wire(co_edge);
        let has_live_wire = wire.is_valid_in(&graph.topo().wires()) && !wire.is_removed(graph);
        if !NodeId::from(co_edge).is_removed(graph) && !has_live_wire {
            let text = "Orphan CoEdge: not owned by any live Wire";
            report(issues, Severity::Error, co_edge, text);
        }
        co_edges.next();
    }
}

// Every live occurrence is placed by exactly one live occurrence reference.
fn check_occurrence_owners(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut occurrences = OccurrenceIterator::new(graph);
    while occurrences.more() {
        let occurrence = occurrences.current_id();
        let owners = (0..graph.inc_storage().nb::<OccurrenceRefTag>())
            .map(OccurrenceRefId::new)
            .filter(|&reference| {
                let entry = graph.refs().occurrences().entry(reference);
                !reference.is_removed(graph) && entry.child_occurrence_id == occurrence
            })
            .count() as u32;
        append_owner_issue(occurrence.into(), owners, "OccurrenceDef", issues);
        occurrences.next();
    }
}

/// Occurrence-to-product liveness: a live occurrence that places a product places a
/// live, in-range product (a soft-removed product with surviving occurrences).
pub(super) fn check_occurrence_product_liveness(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut occurrences = OccurrenceIterator::new(graph);
    while occurrences.more() {
        let child = occurrences.current().child_node_id;
        if child.kind == NodeKind::PRODUCT {
            let product = ProductId::from_node_id(child);
            if !product.is_valid_in(&graph.topo().products()) || product.is_removed(graph) {
                let text =
                    "Active OccurrenceDef.ChildNodeId points to a removed or out-of-range Product";
                report(issues, Severity::Error, occurrences.current_id(), text);
            }
        }
        occurrences.next();
    }
}

/// Cross-container ownership: a solid is owned by a compound or by a compsolid, never
/// by both.
pub(super) fn check_cross_container_ownership(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let compound_solids = compound_owned_solids(graph);
    let mut references = RefIterator::<SolidRefTag, true>::new(graph);
    while references.more() {
        let solid = references.current().child_solid_id;
        let is_live = !references.current_id().is_removed(graph);
        if is_live && solid.is_valid() && compound_solids.contains(&solid) {
            let text = "Cross-container ownership: Solid is referenced by both a Compound \
                        (ChildRef) and a CompSolid (SolidRef)";
            report(issues, Severity::Error, solid, text);
        }
        references.next();
    }
}

// The solids named by live compound child references.
fn compound_owned_solids(graph: &BRepGraph) -> HashSet<SolidId> {
    let mut solids = HashSet::new();
    let mut references = RefIterator::<ChildRefTag, true>::new(graph);
    while references.more() {
        let child = references.current().child_node_id;
        let is_live = !references.current_id().is_removed(graph);
        if is_live && child.kind == NodeKind::SOLID && child.is_valid() {
            solids.insert(SolidId::from_node_id(child));
        }
        references.next();
    }
    solids
}
