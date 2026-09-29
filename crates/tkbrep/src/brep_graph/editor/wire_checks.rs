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
// Rust translation of edgeOrientedVertices, coEdgeOrientedVerticesAfterReplacement,
// toEditorCoEdgeOrderStatus, preCheckCoEdgeOrder and
// rotateConnectedCoEdgesAfterRemoval of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The wire order checks shared by the wire operations.
use super::wires::CoEdgeOrderStatus;
use crate::brep_graph::{CoEdgeId, EdgeId, VertexId, WireId};
use crate::brep_graph_inc::{
    ParityOrientation, Storage, WireCoEdgeOrderStatus, build_co_edge_order, co_edge_ids_are_unique,
    co_edge_oriented_vertices,
};

/// `isActiveNode` over the storage for a typed id.
fn is_active_edge(storage: &Storage, edge: EdgeId) -> bool {
    edge.is_valid_for(storage.nb_edges()) && !storage.is_removed(edge)
}

/// `edgeOrientedVertices(theStorage, theEdgeId, theOrientation, ...)`: the start and
/// end vertices of an active edge taken in the orientation.
pub(crate) fn edge_oriented_vertices(
    storage: &Storage,
    edge: EdgeId,
    orientation: ParityOrientation,
) -> Option<(VertexId, VertexId)> {
    if !is_active_edge(storage, edge) {
        return None;
    }
    let def = storage.edge(edge);
    let (start_ref, end_ref) = if orientation.is_reversed {
        (def.end_vertex_ref_id, def.start_vertex_ref_id)
    } else {
        (def.start_vertex_ref_id, def.end_vertex_ref_id)
    };
    let active_ref = |id: crate::brep_graph::VertexRefId| {
        id.is_valid_for(storage.nb_vertex_refs()) && !storage.is_removed(id)
    };
    if !active_ref(start_ref) || !active_ref(end_ref) {
        return None;
    }
    let start = storage.vertex_ref(start_ref).child_vertex_id;
    let end = storage.vertex_ref(end_ref).child_vertex_id;
    let alive = |id: VertexId| id.is_valid_for(storage.nb_vertices()) && !storage.is_removed(id);
    (alive(start) && alive(end)).then_some((start, end))
}

/// `coEdgeOrientedVerticesAfterReplacement`: the oriented vertices of a coedge once
/// the old edge is replaced by the new one (reversed on request).
pub(crate) fn co_edge_vertices_after_replacement(
    storage: &Storage,
    co_edge: CoEdgeId,
    old_edge: EdgeId,
    new_edge: EdgeId,
    reversed: bool,
) -> Option<(VertexId, VertexId)> {
    if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
        return None;
    }
    let def = storage.co_edge(co_edge);
    if def.child_edge_id != old_edge {
        return co_edge_oriented_vertices(storage, co_edge);
    }
    let orientation = ParityOrientation {
        is_reversed: def.orientation.is_reversed != reversed,
    };
    edge_oriented_vertices(storage, new_edge, orientation)
}

/// `toEditorCoEdgeOrderStatus`: Connected is Ready, Reordered stays, the rest is
/// Disconnected.
pub(crate) fn to_editor_status(status: WireCoEdgeOrderStatus) -> CoEdgeOrderStatus {
    match status {
        WireCoEdgeOrderStatus::Connected => CoEdgeOrderStatus::Ready,
        WireCoEdgeOrderStatus::Reordered => CoEdgeOrderStatus::Reordered,
        _ => CoEdgeOrderStatus::Disconnected,
    }
}

/// The input of [`pre_check_co_edge_order`].
pub(crate) struct OrderRequest<'a> {
    pub input: &'a [CoEdgeId],
    pub expected_wire: WireId,
    pub require_free: bool,
    pub require_permutation: bool,
}

/// `preCheckCoEdgeOrder(...)`: the status and, when usable, the ordered coedges.
pub(crate) fn pre_check_co_edge_order(
    storage: &Storage,
    request: &OrderRequest<'_>,
) -> (CoEdgeOrderStatus, Vec<CoEdgeId>) {
    let input = request.input;
    if input.is_empty() {
        return (CoEdgeOrderStatus::Empty, Vec::new());
    }
    if !co_edge_ids_are_unique(input) {
        return (CoEdgeOrderStatus::DuplicateCoEdge, Vec::new());
    }
    let old = match expected_wire_co_edges(storage, request) {
        Ok(old) => old,
        Err(status) => return (status, Vec::new()),
    };
    if let Some(status) = check_each_co_edge(storage, request) {
        return (status, Vec::new());
    }
    if let Some(old) = old.filter(|_| request.require_permutation) {
        if !is_permutation(&old, input) {
            return (CoEdgeOrderStatus::NotPermutation, Vec::new());
        }
        if old.as_slice() == input && storage.validate_wire_co_edge_order(request.expected_wire) {
            return (CoEdgeOrderStatus::AlreadyCurrent, input.to_vec());
        }
    }
    let (status, ordered) = build_co_edge_order(storage, WireId::default(), input);
    (to_editor_status(status), ordered)
}

// The expected wire's coedges (none without an expected wire), or the status that
// refuses the wire or the input size.
fn expected_wire_co_edges(
    storage: &Storage,
    request: &OrderRequest<'_>,
) -> Result<Option<Vec<CoEdgeId>>, CoEdgeOrderStatus> {
    let wire = request.expected_wire;
    if !wire.is_valid() {
        return Ok(None);
    }
    if !wire.is_valid_for(storage.nb_wires()) || storage.is_removed(wire) {
        return Err(CoEdgeOrderStatus::InvalidWire);
    }
    let old = storage.wire_relations(wire).co_edge_ids.clone();
    if request.require_permutation && request.input.len() != old.len() {
        return Err(CoEdgeOrderStatus::SizeMismatch);
    }
    Ok(Some(old))
}

// The per-coedge checks, in native order: active, free when required, owned by the
// expected wire, oriented vertices resolvable.
fn check_each_co_edge(storage: &Storage, request: &OrderRequest<'_>) -> Option<CoEdgeOrderStatus> {
    for co_edge in request.input {
        if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(*co_edge) {
            return Some(CoEdgeOrderStatus::InvalidCoEdge);
        }
        let parent = storage.co_edge(*co_edge).parent_wire_id;
        if request.require_free && parent.is_valid() {
            return Some(CoEdgeOrderStatus::CoEdgeAlreadyBound);
        }
        if request.expected_wire.is_valid() && parent != request.expected_wire {
            return Some(CoEdgeOrderStatus::CoEdgeNotOwnedByWire);
        }
        if co_edge_oriented_vertices(storage, *co_edge).is_none() {
            return Some(CoEdgeOrderStatus::InvalidCoEdge);
        }
    }
    None
}

// Every input coedge matches one distinct old coedge (the native matched-flag scan).
fn is_permutation(old: &[CoEdgeId], input: &[CoEdgeId]) -> bool {
    let mut matched = vec![false; old.len()];
    for id in input {
        let Some(slot) = (0..old.len()).find(|index| !matched[*index] && old[*index] == *id) else {
            return false;
        };
        matched[slot] = true;
    }
    true
}

/// True when the coedges chain end to start in the given order.
pub(crate) fn is_connected_chain(storage: &Storage, co_edges: &[CoEdgeId]) -> bool {
    let mut previous_end: Option<VertexId> = None;
    for co_edge in co_edges {
        let Some((start, end)) = co_edge_oriented_vertices(storage, *co_edge) else {
            return false;
        };
        if previous_end.is_some_and(|previous| previous != start) {
            return false;
        }
        previous_end = Some(end);
    }
    true
}

/// `rotateConnectedCoEdgesAfterRemoval`: the remaining coedges rotated to the first
/// start that chains them all (fewer than two are kept as they are); `None` when no
/// rotation chains.
pub(crate) fn rotate_connected_after_removal(
    storage: &Storage,
    current: &[CoEdgeId],
    removed: CoEdgeId,
) -> Option<Vec<CoEdgeId>> {
    let remaining: Vec<CoEdgeId> = current
        .iter()
        .copied()
        .filter(|id| *id != removed)
        .collect();
    if remaining.len() < 2 {
        return Some(remaining);
    }
    let count = remaining.len();
    (0..count).find_map(|start| {
        let rotated: Vec<CoEdgeId> = (0..count)
            .map(|offset| remaining[(start + offset) % count])
            .collect();
        is_connected_chain(storage, &rotated).then_some(rotated)
    })
}
