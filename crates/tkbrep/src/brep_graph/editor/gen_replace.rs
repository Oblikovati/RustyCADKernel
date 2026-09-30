// Rust translation of EditorView::GenOps::ReplaceNode, rebindCoEdgesForEdgeReplacement
// and unbindCoEdgesOfRemovedEdge of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GenOps::ReplaceNode`: removal with a replacement the layers migrate to.
use super::gen_ops::GenOps;
use super::gen_remove::{finish_node_removal, mark_owned_ref_removed, propagate_before_removal};
use super::helpers::{is_active_node, is_node_in_range, node_item, require_mutable};
use crate::brep_graph::*;
use crate::brep_graph_inc::Storage;
use rustycad_tkernel::standard::Failure;

impl GenOps<'_> {
    /// `ReplaceNode(theNode, theReplacement)`: removes the node like `RemoveNode`, but an
    /// edge's coedges first move to a replacement edge, and the layers receive
    /// `OnNodeReplaced` (after the removal) instead of `OnNodeRemoved`. An invalid or
    /// inactive replacement falls back to `RemoveNode`. A node equal to its
    /// replacement, an invalid node or an inactive one is ignored; a locked or guarded
    /// node raises Standard_ProgramError.
    pub fn replace_node(&mut self, node: NodeId, replacement: NodeId) -> Result<(), Failure> {
        if !node.is_valid() || replacement == node {
            return Ok(());
        }
        require_mutable(
            self.graph,
            node_item(node),
            "BRepGraph::EditorView::ReplaceNode(): locked node",
            "BRepGraph::EditorView::ReplaceNode(): guard active on node",
        )?;
        if !replacement.is_valid() || !is_active_node(self.graph, replacement) {
            return self.remove_node(node);
        }
        if node.kind == NodeKind::EDGE && replacement.kind == NodeKind::EDGE {
            if !is_node_in_range(self.graph, node) {
                return Ok(());
            }
            let (source, target) = (
                EdgeId::from_node_id(node),
                EdgeId::from_node_id(replacement),
            );
            rebind_co_edges_of_edge(self.graph.inc_storage_mut(), source, target);
        }
        if !is_node_in_range(self.graph, node) || !is_active_node(self.graph, node) {
            return Ok(());
        }
        propagate_before_removal(self.graph, node);
        match node.kind {
            NodeKind::EDGE => replace_edge_uses(self.graph, EdgeId::from_node_id(node)),
            NodeKind::CO_EDGE => {
                detach_co_edge(self.graph.inc_storage_mut(), CoEdgeId::from_node_id(node))
            }
            NodeKind::WIRE => retire_wire_co_edges(self.graph, WireId::from_node_id(node)),
            _ => {}
        }
        finish_node_removal(self.graph, node);
        let layers = self.graph.layer_registry();
        layers.dispatch_on_node_replaced(Some(&*self.graph), node, replacement);
        Ok(())
    }
}

/// `rebindCoEdgesForEdgeReplacement` (target valid) and `unbindCoEdgesOfRemovedEdge`
/// (target invalid): every active coedge of the source edge (over a snapshot of its
/// relation) moves to the target and the edge-to-coedge relation follows.
fn rebind_co_edges_of_edge(storage: &mut Storage, source: EdgeId, target: EdgeId) {
    let snapshot = storage.edge_relations(source).co_edge_ids.clone();
    for co_edge in snapshot {
        if !co_edge.is_valid_for(storage.nb_co_edges()) {
            continue;
        }
        if storage.is_removed(co_edge) || storage.co_edge(co_edge).child_edge_id != source {
            continue;
        }
        storage.change_co_edge(co_edge).child_edge_id = target;
        storage.rebind_co_edge_edge(co_edge, source, target);
    }
}

// The edge case: coedges still on the edge lose it, then the boundary references are
// removed.
fn replace_edge_uses(graph: &mut BRepGraph, edge: EdgeId) {
    let def = graph.inc_storage().edge(edge).clone();
    rebind_co_edges_of_edge(graph.inc_storage_mut(), edge, EdgeId::default());
    mark_owned_ref_removed(graph, def.start_vertex_ref_id.into());
    mark_owned_ref_removed(graph, def.end_vertex_ref_id.into());
}

// The coedge case: an active coedge leaves its wire.
fn detach_co_edge(storage: &mut Storage, co_edge: CoEdgeId) {
    if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
        return;
    }
    let wire = storage.co_edge(co_edge).parent_wire_id;
    if wire.is_valid_for(storage.nb_wires()) {
        storage.detach_co_edge_use(wire, co_edge);
    }
}

// The wire case: every active coedge of the wire (its list, then every coedge still
// naming the wire) is detached, removed and marked modified.
fn retire_wire_co_edges(graph: &mut BRepGraph, wire: WireId) {
    if !wire.is_valid_for(graph.inc_storage().nb_wires()) {
        return;
    }
    let snapshot = graph.inc_storage().wire_relations(wire).co_edge_ids.clone();
    for co_edge in snapshot {
        retire_co_edge_of_wire(graph, wire, co_edge, false);
    }
    for index in 0..graph.inc_storage().nb_co_edges() {
        retire_co_edge_of_wire(graph, wire, CoEdgeId::new(index), true);
    }
}

fn retire_co_edge_of_wire(
    graph: &mut BRepGraph,
    wire: WireId,
    co_edge: CoEdgeId,
    require_parent: bool,
) {
    let storage = graph.inc_storage_mut();
    if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
        return;
    }
    if require_parent && storage.co_edge(co_edge).parent_wire_id != wire {
        return;
    }
    storage.detach_co_edge_use(wire, co_edge);
    storage.set_removed(co_edge, true);
    graph.mark_modified(co_edge.into());
}
