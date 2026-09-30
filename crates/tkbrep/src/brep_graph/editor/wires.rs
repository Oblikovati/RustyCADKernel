// Rust translation of EditorView::WireOps (the status enums, the prechecks, Add, Mut,
// MutRef and the reference setters) of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.hxx, BRepGraph_EditorView.cxx,
// BRepGraph_EditorView_Mut.cxx and BRepGraph_EditorView_Setters.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `EditorView::WireOps`: wire creation, the order prechecks and the wire reference
//! setters. The structural edits are in `wire_edits`.
use super::guards::{mutate_node, mutate_ref};
use super::helpers::require_settable;
use super::wire_checks::{
    OrderRequest, co_edge_vertices_after_replacement, is_connected_chain, pre_check_co_edge_order,
    to_editor_status,
};
use crate::brep_graph::mut_guard::MutGuard;
use crate::brep_graph::*;
use crate::brep_graph_inc::{
    ParityOrientation, append_unique, build_co_edge_order, co_edge_oriented_vertices,
};
use rustycad_tkernel::standard::Failure;

/// `WireOps::CoEdgeOrderStatus`: the result of a wire coedge-order precheck.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CoEdgeOrderStatus {
    /// Valid and already connected in the given order.
    Ready,
    /// Valid after the canonical reordering.
    Reordered,
    /// The input is the stored order; the mutation can be skipped.
    AlreadyCurrent,
    /// The append precheck found the coedge already in the wire.
    AlreadyContained,
    /// No coedge.
    Empty,
    /// The wire is invalid or removed.
    InvalidWire,
    /// The input size differs from the wire's coedge count.
    SizeMismatch,
    /// A coedge is repeated.
    DuplicateCoEdge,
    /// A coedge is invalid, removed or incomplete.
    InvalidCoEdge,
    /// The add precheck found a coedge owned by a wire.
    CoEdgeAlreadyBound,
    /// The set-order precheck found a coedge not owned by the wire.
    CoEdgeNotOwnedByWire,
    /// The set-order input is not a permutation of the current coedges.
    NotPermutation,
    /// The coedges cannot form a connected chain.
    Disconnected,
}

/// `WireOps::ReplaceEdgeStatus`: the result of an edge-replacement precheck.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReplaceEdgeStatus {
    /// The replacement keeps the wire connected.
    Ready,
    /// The old and new edge are the same and no mutation is needed.
    AlreadyCurrent,
    /// The wire is invalid or removed.
    InvalidWire,
    /// The old edge is invalid, removed or not used by the wire.
    InvalidOldEdge,
    /// The new edge is invalid or removed.
    InvalidNewEdge,
    /// The replacement would break the coedge chain.
    Disconnected,
}

/// `EditorView::WireOps`.
#[derive(Debug)]
pub struct WireOps<'g> {
    pub(super) graph: &'g mut BRepGraph,
}

impl<'g> WireOps<'g> {
    /// `CheckCoEdgeOrder(theCoEdgeIds)`: can the free coedges form a wire?
    pub fn check_co_edge_order(&self, co_edges: &[CoEdgeId]) -> CoEdgeOrderStatus {
        let request = OrderRequest {
            input: co_edges,
            expected_wire: WireId::default(),
            require_free: true,
            require_permutation: false,
        };
        pre_check_co_edge_order(self.graph.inc_storage(), &request).0
    }

    /// `CheckCoEdgeOrder(theWire, theCoEdgeIds)`: can the permutation replace the
    /// wire's stored order?
    pub fn check_wire_co_edge_order(
        &self,
        wire: WireId,
        co_edges: &[CoEdgeId],
    ) -> CoEdgeOrderStatus {
        let storage = self.graph.inc_storage();
        if !wire.is_valid_for(storage.nb_wires()) || storage.is_removed(wire) {
            return CoEdgeOrderStatus::InvalidWire;
        }
        let request = OrderRequest {
            input: co_edges,
            expected_wire: wire,
            require_free: false,
            require_permutation: true,
        };
        pre_check_co_edge_order(storage, &request).0
    }

    /// `CheckAppendCoEdge(theWire, theCoEdgeId)`: can the free coedge be appended with
    /// the wire staying connected (possibly after reordering)?
    pub fn check_append_co_edge(&self, wire: WireId, co_edge: CoEdgeId) -> CoEdgeOrderStatus {
        let storage = self.graph.inc_storage();
        if !wire.is_valid_for(storage.nb_wires()) || storage.is_removed(wire) {
            return CoEdgeOrderStatus::InvalidWire;
        }
        if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
            return CoEdgeOrderStatus::InvalidCoEdge;
        }
        let parent = storage.co_edge(co_edge).parent_wire_id;
        if parent == wire {
            return CoEdgeOrderStatus::AlreadyContained;
        }
        if parent.is_valid() {
            return CoEdgeOrderStatus::CoEdgeAlreadyBound;
        }
        if co_edge_oriented_vertices(storage, co_edge).is_none() {
            return CoEdgeOrderStatus::InvalidCoEdge;
        }
        let mut candidate = Vec::new();
        for current in &storage.wire_relations(wire).co_edge_ids {
            let owned = current.is_valid_for(storage.nb_co_edges())
                && !storage.is_removed(*current)
                && storage.co_edge(*current).parent_wire_id == wire;
            if !owned || co_edge_oriented_vertices(storage, *current).is_none() {
                return CoEdgeOrderStatus::InvalidCoEdge;
            }
            candidate.push(*current);
        }
        candidate.push(co_edge);
        if is_connected_chain(storage, &candidate) {
            return CoEdgeOrderStatus::Ready;
        }
        to_editor_status(build_co_edge_order(storage, WireId::default(), &candidate).0)
    }

    /// `CheckReplaceEdge(theWire, theOldEdge, theNewEdge, theReversed)`.
    pub fn check_replace_edge(
        &self,
        wire: WireId,
        old_edge: EdgeId,
        new_edge: EdgeId,
        reversed: bool,
    ) -> ReplaceEdgeStatus {
        let storage = self.graph.inc_storage();
        if !wire.is_valid_for(storage.nb_wires()) || storage.is_removed(wire) {
            return ReplaceEdgeStatus::InvalidWire;
        }
        let active =
            |edge: EdgeId| edge.is_valid_for(storage.nb_edges()) && !storage.is_removed(edge);
        if !active(old_edge) {
            return ReplaceEdgeStatus::InvalidOldEdge;
        }
        if !active(new_edge) {
            return ReplaceEdgeStatus::InvalidNewEdge;
        }
        if old_edge == new_edge && !reversed {
            return ReplaceEdgeStatus::AlreadyCurrent;
        }
        self.replacement_chain_status(wire, old_edge, new_edge, reversed)
    }

    // The chain walk of `CheckReplaceEdge`.
    fn replacement_chain_status(
        &self,
        wire: WireId,
        old_edge: EdgeId,
        new_edge: EdgeId,
        reversed: bool,
    ) -> ReplaceEdgeStatus {
        let storage = self.graph.inc_storage();
        let (mut has_old, mut previous_end) = (false, None::<VertexId>);
        for co_edge in &storage.wire_relations(wire).co_edge_ids {
            let owned = co_edge.is_valid_for(storage.nb_co_edges())
                && !storage.is_removed(*co_edge)
                && storage.co_edge(*co_edge).parent_wire_id == wire;
            if !owned {
                return ReplaceEdgeStatus::InvalidOldEdge;
            }
            has_old = has_old || storage.co_edge(*co_edge).child_edge_id == old_edge;
            let vertices =
                co_edge_vertices_after_replacement(storage, *co_edge, old_edge, new_edge, reversed);
            let Some((start, end)) = vertices else {
                return ReplaceEdgeStatus::Disconnected;
            };
            if previous_end.is_some_and(|previous| previous != start) {
                return ReplaceEdgeStatus::Disconnected;
            }
            previous_end = Some(end);
        }
        if has_old {
            ReplaceEdgeStatus::Ready
        } else {
            ReplaceEdgeStatus::InvalidOldEdge
        }
    }

    /// `Add(theCoEdgeIds)`: a wire over free coedges, in connected order (reordered when
    /// needed); an empty input gives an empty wire; the invalid id when the prechecks
    /// refuse the input.
    pub fn add(&mut self, co_edges: &[CoEdgeId]) -> WireId {
        if co_edges.is_empty() {
            let wire = self.graph.inc_storage_mut().append_wire();
            self.graph.allocate_uid(wire.into());
            self.graph.mark_modified(wire.into());
            return wire;
        }
        let request = OrderRequest {
            input: co_edges,
            expected_wire: WireId::default(),
            require_free: true,
            require_permutation: false,
        };
        let (status, ordered) = pre_check_co_edge_order(self.graph.inc_storage(), &request);
        if status != CoEdgeOrderStatus::Ready && status != CoEdgeOrderStatus::Reordered {
            return WireId::default();
        }
        let wire = self.graph.inc_storage_mut().append_wire();
        self.graph.allocate_uid(wire.into());
        for co_edge in ordered {
            self.bind_co_edge(wire, co_edge);
        }
        self.graph.mark_modified(wire.into());
        wire
    }

    // One coedge of `Add`: its parent wire, the wire and edge relations, then the
    // coedge and its edge marked modified.
    fn bind_co_edge(&mut self, wire: WireId, co_edge: CoEdgeId) {
        let storage = self.graph.inc_storage_mut();
        storage.change_co_edge(co_edge).parent_wire_id = wire;
        let edge = storage.co_edge(co_edge).child_edge_id;
        storage
            .change_wire_relations_internal(wire)
            .co_edge_ids
            .push(co_edge);
        append_unique(
            &mut storage.change_edge_relations_internal(edge).co_edge_ids,
            co_edge,
        );
        self.graph.mark_modified(co_edge.into());
        self.graph.mark_modified(edge.into());
    }

    /// `Mut(theWire)`: the scoped guard of a wire definition.
    pub fn mutate(self, wire: WireId) -> Result<MutGuard<'g, WireTag>, Failure> {
        mutate_node(self.graph, wire)
    }

    /// `MutRef(theWireRef)`: the scoped guard of a wire reference.
    pub fn mutate_ref(self, reference: WireRefId) -> Result<MutGuard<'g, WireRefTag>, Failure> {
        mutate_ref(self.graph, reference)
    }

    /// `SetRefOrientation(theWireRef, theOrientation)`.
    pub fn set_ref_orientation(
        &mut self,
        reference: WireRefId,
        orientation: ParityOrientation,
    ) -> Result<(), Failure> {
        require_settable(self.graph, reference)?;
        self.graph
            .inc_storage_mut()
            .change_wire_ref(reference)
            .orientation = orientation;
        self.graph.mark_ref_modified(reference.into());
        Ok(())
    }

    /// `SetRefOrientation(theMut, theOrientation)`.
    pub fn set_ref_orientation_guarded(
        guard: &mut MutGuard<'_, WireRefTag>,
        orientation: ParityOrientation,
    ) {
        guard.internal().orientation = orientation;
    }

    /// `SetRefChildWireId(theWireRef, theWire)`: retargets the reference and rebinds the
    /// wire-to-face relations.
    pub fn set_ref_child_wire_id(
        &mut self,
        reference: WireRefId,
        wire: WireId,
    ) -> Result<(), Failure> {
        require_settable(self.graph, reference)?;
        let storage = self.graph.inc_storage_mut();
        let old = storage.wire_ref(reference).child_wire_id;
        if old == wire {
            return Ok(());
        }
        storage.change_wire_ref(reference).child_wire_id = wire;
        storage.rebind_wire_ref(reference, old, wire);
        self.graph.mark_ref_modified(reference.into());
        Ok(())
    }

    /// `SetRefChildWireId(theMut, theWire)`.
    pub fn set_ref_child_wire_id_guarded(guard: &mut MutGuard<'_, WireRefTag>, wire: WireId) {
        let old = guard.child_wire_id;
        if old == wire {
            return;
        }
        guard.internal().child_wire_id = wire;
        let reference = guard.id();
        guard
            .graph_mut()
            .inc_storage_mut()
            .rebind_wire_ref(reference, old, wire);
    }
}
