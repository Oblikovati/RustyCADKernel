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
// Rust translation of `redirectOccurrenceChildren`, the compound child redirect and the
// replace-and-record tail that every merge phase of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Deduplicate.cxx repeats, at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parents every merged definition is redirected in, and the merge's closing
//! `ReplaceNode` with its history record.
use crate::brep_graph::editor::GenOps;
use crate::brep_graph::id_walk::active_ids;
use crate::brep_graph::layer_history::LayerHistory;
use crate::brep_graph::refs_iterator::RefsChildOfCompound;
use crate::brep_graph::{BRepGraph, ChildRefId, CompoundTag, NodeId, OccurrenceRefId};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

/// The compound child references that still point at `old` point at `canon`: each
/// active reference of each active compound, rewritten through a child-reference guard.
pub(super) fn redirect_compound_children(
    graph: &mut BRepGraph,
    old: NodeId,
    canon: NodeId,
) -> Result<(), Failure> {
    for compound in active_ids::<CompoundTag>(graph) {
        if compound.is_removed(graph) {
            continue;
        }
        let refs: Vec<ChildRefId> = RefsChildOfCompound::new(graph, compound).collect();
        for reference in refs {
            let child = graph.refs().children().entry(reference).child_node_id;
            if graph.refs().generic().is_removed(reference.into()) || child != old {
                continue;
            }
            let mut guard = graph.editor().generic().mutate_child_ref(reference)?;
            GenOps::set_child_ref_child_node_id_guarded(&mut guard, canon);
        }
    }
    Ok(())
}

/// `redirectOccurrenceChildren(theGraph, theOldNodeId, theNewNodeId)`: every active
/// occurrence (reached through an active reference) whose child is `old` gets `canon`.
/// The reference list is snapshotted first: `SetChildNodeId` rewrites it.
pub(super) fn redirect_occurrence_children(
    graph: &mut BRepGraph,
    old: NodeId,
    canon: NodeId,
) -> Result<(), Failure> {
    let refs: Vec<OccurrenceRefId> = graph.topo().generic().occurrence_ref_ids(old).to_vec();
    for reference in refs {
        if !reference.is_valid() || graph.refs().generic().is_removed(reference.into()) {
            continue;
        }
        let occurrence = graph.refs().occurrences().entry(reference).child_occurrence_id;
        if !occurrence.is_valid() || occurrence.is_removed(graph) {
            continue;
        }
        if graph.topo().occurrences().definition(occurrence).child_node_id == old {
            graph
                .editor()
                .occurrences()
                .set_child_node_id(occurrence, canon)?;
        }
    }
    Ok(())
}

/// The parents shared by every merge: the compound children, then the occurrence
/// children.
pub(super) fn redirect_generic_parents(
    graph: &mut BRepGraph,
    old: NodeId,
    canon: NodeId,
) -> Result<(), Failure> {
    redirect_compound_children(graph, old, canon)?;
    redirect_occurrence_children(graph, old, canon)
}

/// The merge's tail: `Gen().ReplaceNode(old, canon)`, then a Replaced history record
/// under `label`.
pub(super) fn replace_and_record(
    graph: &mut BRepGraph,
    history: &Handle<LayerHistory>,
    label: &AsciiString,
    old: NodeId,
    canon: NodeId,
) -> Result<(), Failure> {
    graph.editor().generic().replace_node(old, canon)?;
    history.record_replaced(graph, label, old, canon);
    Ok(())
}
