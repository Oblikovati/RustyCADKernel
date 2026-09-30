// Rust translation of EditorView::WireOps::ReplaceEdge, RemoveCoEdge, Reverse and
// SetCoEdgeOrder of ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The structural edits of `EditorView::WireOps`.
use super::helpers::{has_any_active_usage, is_active_node};
use super::wire_checks::{OrderRequest, pre_check_co_edge_order, rotate_connected_after_removal};
use super::wires::{CoEdgeOrderStatus, ReplaceEdgeStatus, WireOps};
use crate::brep_graph::refs_iterator::CoEdgesOfWire;
use crate::brep_graph::*;
use crate::brep_graph_inc::{ParityOrientation, Storage};
use rustycad_tkernel::standard::Failure;

impl WireOps<'_> {
    /// `ReplaceEdge(theChildWireId, theOldEdgeEntity, theNewEdgeEntity, theReversed)`:
    /// every active coedge of the wire on the old edge moves to the new edge (reversed
    /// with its p-curve on request) when the precheck says the chain stays connected.
    /// Inputs that a debug-only native assertion refuses (ids out of range, a removed
    /// new edge) are ignored, as in the release build. Reversing a p-curve can raise.
    pub fn replace_edge(
        &mut self,
        wire: WireId,
        old_edge: EdgeId,
        new_edge: EdgeId,
        reversed: bool,
    ) -> Result<(), Failure> {
        let storage = self.graph.inc_storage();
        if !wire.is_valid_for(storage.nb_wires())
            || !old_edge.is_valid_for(storage.nb_edges())
            || !new_edge.is_valid_for(storage.nb_edges())
            || storage.is_removed(new_edge)
            || !is_active_node(self.graph, wire)
            || !is_active_node(self.graph, old_edge)
        {
            return Ok(());
        }
        if self.check_replace_edge(wire, old_edge, new_edge, reversed) != ReplaceEdgeStatus::Ready {
            return Ok(());
        }
        let co_edges: Vec<CoEdgeId> = CoEdgesOfWire::new(self.graph, wire).collect();
        for co_edge in co_edges {
            retarget_co_edge(
                self.graph.inc_storage_mut(),
                co_edge,
                old_edge,
                new_edge,
                reversed,
            )?;
        }
        self.graph.mark_modified(wire.into());
        Ok(())
    }

    /// `RemoveCoEdge(theChildWireId, theCoEdgeId)`: detaches one coedge use of an active
    /// wire when the remaining coedges still chain (rotated to a connected start), and
    /// removes the coedge when it has no other use; false otherwise (and for ids out
    /// of range, the release behaviour of the native assertions).
    pub fn remove_co_edge(&mut self, wire: WireId, co_edge: CoEdgeId) -> Result<bool, Failure> {
        let storage = self.graph.inc_storage();
        if !wire.is_valid_for(storage.nb_wires())
            || !co_edge.is_valid_for(storage.nb_co_edges())
            || !is_active_node(self.graph, wire)
            || !is_active_node(self.graph, co_edge)
        {
            return Ok(false);
        }
        let current = &storage.wire_relations(wire).co_edge_ids;
        if !current.contains(&co_edge) {
            return Ok(false);
        }
        let Some(remaining) = rotate_connected_after_removal(storage, current, co_edge) else {
            return Ok(false);
        };
        let storage = self.graph.inc_storage_mut();
        storage.detach_co_edge_use(wire, co_edge);
        if !remaining.is_empty() {
            storage.set_wire_co_edges(wire, &remaining);
        }
        let still_used = has_any_active_usage(self.graph, co_edge.into())?;
        self.graph.mark_modified(wire.into());
        if !still_used {
            self.graph
                .editor()
                .generic()
                .remove_subgraph(co_edge.into())?;
        }
        Ok(true)
    }

    /// `Reverse(theWire)`: reverses the coedge order and flips each active coedge; an
    /// inactive wire (or one out of range, refused by a debug-only assertion natively)
    /// is ignored.
    pub fn reverse(&mut self, wire: WireId) {
        if !is_active_node(self.graph, wire) {
            return;
        }
        let storage = self.graph.inc_storage_mut();
        storage.reverse_wire_co_edges(wire);
        let co_edges = storage.wire_relations(wire).co_edge_ids.clone();
        for co_edge in co_edges {
            let storage = self.graph.inc_storage_mut();
            if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
                continue;
            }
            let def = storage.change_co_edge(co_edge);
            def.orientation = ParityOrientation {
                is_reversed: !def.orientation.is_reversed,
            };
            self.graph.mark_modified(co_edge.into());
        }
        self.graph.mark_modified(wire.into());
    }

    /// `SetCoEdgeOrder(theWire, theCoEdgeIds)`: stores a permutation of the wire's
    /// coedges in connected order; true when accepted (or already current), false
    /// when refused or when the stored order does not validate (then the old order is
    /// restored).
    pub fn set_co_edge_order(&mut self, wire: WireId, co_edges: &[CoEdgeId]) -> bool {
        let storage = self.graph.inc_storage();
        if !wire.is_valid_for(storage.nb_wires()) || !is_active_node(self.graph, wire) {
            return false;
        }
        let request = OrderRequest {
            input: co_edges,
            expected_wire: wire,
            require_free: false,
            require_permutation: true,
        };
        let (status, ordered) = pre_check_co_edge_order(storage, &request);
        if status == CoEdgeOrderStatus::AlreadyCurrent {
            return true;
        }
        if status != CoEdgeOrderStatus::Ready && status != CoEdgeOrderStatus::Reordered {
            return false;
        }
        let storage = self.graph.inc_storage_mut();
        let snapshot = storage.wire_relations(wire).co_edge_ids.clone();
        storage.set_wire_co_edges(wire, &ordered);
        if !storage.validate_wire_co_edge_order(wire) {
            storage.set_wire_co_edges(wire, &snapshot);
            return false;
        }
        self.graph.mark_modified(wire.into());
        true
    }
}

// One coedge of `ReplaceEdge`: moved to the new edge (flipped with its p-curve when
// reversed) and rebound, when it is active and on the old edge.
fn retarget_co_edge(
    storage: &mut Storage,
    co_edge: CoEdgeId,
    old_edge: EdgeId,
    new_edge: EdgeId,
    reversed: bool,
) -> Result<(), Failure> {
    if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
        return Ok(());
    }
    if storage.co_edge(co_edge).child_edge_id != old_edge {
        return Ok(());
    }
    let def = storage.change_co_edge(co_edge);
    def.child_edge_id = new_edge;
    if reversed {
        def.orientation = ParityOrientation {
            is_reversed: !def.orientation.is_reversed,
        };
        reverse_pcurve(storage, co_edge)?;
    }
    storage.rebind_co_edge_edge(co_edge, old_edge, new_edge);
    Ok(())
}

// Reverses the p-curve of a reversed replacement so it follows the new edge's
// direction: the range through `ReversedParameter`, then the curve.
fn reverse_pcurve(storage: &mut Storage, co_edge: CoEdgeId) -> Result<(), Failure> {
    let rep = storage.co_edge(co_edge).curve2d_rep_id;
    if !rep.is_valid() || !rep.is_valid_for(storage.nb_co_edge_curves2d()) {
        return Ok(());
    }
    let record = storage.change_co_edge_curve2d_rep(rep);
    if record.curve.is_null() {
        return Ok(());
    }
    let (old_first, old_last) = (record.param_first, record.param_last);
    record.param_first = record.curve.reversed_parameter(old_last)?;
    record.param_last = record.curve.reversed_parameter(old_first)?;
    record.curve = record.curve.reversed()?;
    Ok(())
}
