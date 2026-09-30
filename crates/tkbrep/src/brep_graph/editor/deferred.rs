// Rust translation of EditorView::BeginDeferredInvalidation and
// EndDeferredInvalidation of ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Deferred invalidation: while it is active, `markModified` only bumps the node's
//! generations and queues it; the end of the mode propagates and dispatches once.
use super::EditorView;
use crate::brep_graph::layer::{kind_bit, ref_kind_bit};
use crate::brep_graph::*;

impl EditorView<'_> {
    /// `BeginDeferredInvalidation()`: queue modifications instead of propagating them.
    /// It batches invalidation only; it does not serialise the mutations.
    pub fn begin_deferred_invalidation(&mut self) {
        self.graph.inc_storage().set_deferred_mode(true);
    }

    /// `EndDeferredInvalidation()`: leaves deferred mode and flushes. The queued nodes
    /// (each once, in queue order) and then their ancestors, breadth first, get one
    /// SubtreeGen bump per ancestor and one batch `OnNodesModified`; the queued
    /// references get one batch `OnRefsModified`. Vertices do not propagate in this
    /// mode, as natively. Nothing happens outside deferred mode.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::BRepGraph;
    /// let mut graph = BRepGraph::new();
    /// graph.editor().begin_deferred_invalidation();
    /// let shell = graph.editor().shells().add();
    /// graph.editor().end_deferred_invalidation();
    /// assert!(!graph.editor_state().is_deferred_mode());
    /// assert_eq!(graph.topo().shells().definition(shell).base.own_gen, 1);
    /// ```
    pub fn end_deferred_invalidation(&mut self) {
        let graph = &mut *self.graph;
        if !graph.inc_storage().deferred_mode() {
            return;
        }
        graph.inc_storage().set_deferred_mode(false);
        if !graph.inc_storage().deferred_modified().is_empty() {
            let (modified, mask) = propagate_deferred(graph);
            let layers = graph.layer_registry();
            if layers.has_modification_subscribers() && !modified.is_empty() {
                layers.dispatch_nodes_modified(Some(&*graph), &modified, mask);
            }
        }
        dispatch_deferred_refs(graph);
        graph.inc_storage_mut().clear_deferred_queues();
    }
}

// The deferred reference modifications, dispatched in one batch to the subscribers.
fn dispatch_deferred_refs(graph: &BRepGraph) {
    let references = graph.inc_storage().deferred_ref_modified();
    let layers = graph.layer_registry();
    if references.is_empty() || !layers.has_ref_modification_subscribers() {
        return;
    }
    let mask = references
        .iter()
        .fold(0, |mask, reference| mask | ref_kind_bit(reference.kind));
    layers.dispatch_refs_modified(Some(graph), references, mask);
}

/// The dense per-kind visited flags of the flush, sized by the store counts when the
/// flush starts (a node past them is never visited).
struct Visited {
    flags: Vec<Vec<bool>>,
}

impl Visited {
    fn new(graph: &BRepGraph) -> Self {
        let generic = graph.topo().generic();
        let flags = (0..NodeId::KIND_COUNT)
            .map(|kind| vec![false; generic.nb(NodeKind(kind)) as usize])
            .collect();
        Self { flags }
    }

    /// `markVisited(theNode)`: true the first time an in-range node is seen.
    fn mark(&mut self, node: NodeId) -> bool {
        let Some(flags) = usize::try_from(node.kind.0)
            .ok()
            .and_then(|kind| self.flags.get_mut(kind))
        else {
            return false;
        };
        match flags.get_mut(node.index as usize) {
            Some(flag) if !*flag => {
                *flag = true;
                true
            }
            _ => false,
        }
    }
}

// The breadth-first flush: the seeds, then each node's parents appended once.
fn propagate_deferred(graph: &mut BRepGraph) -> (Vec<NodeId>, i32) {
    let mut visited = Visited::new(graph);
    let mut modified = Vec::with_capacity(graph.inc_storage().deferred_modified().len() * 2);
    let mut mask = 0;
    for node in graph.inc_storage().deferred_modified().to_vec() {
        if visited.mark(node) {
            modified.push(node);
            mask |= kind_bit(node.kind);
        }
    }
    let mut index = 0;
    while let Some(node) = modified.get(index).copied() {
        for parent in deferred_parents(graph, node) {
            if !visited.mark(parent) {
                continue;
            }
            let removed = parent.is_removed(graph);
            let Some(entity) = graph.change_topo_entity(parent) else {
                continue;
            };
            if removed {
                continue;
            }
            entity.subtree_gen = entity.subtree_gen.wrapping_add(1);
            modified.push(parent);
            mask |= kind_bit(parent.kind);
        }
        index += 1;
    }
    (modified, mask)
}

// The parents the flush visits for one node, in native order.
fn deferred_parents(graph: &BRepGraph, node: NodeId) -> Vec<NodeId> {
    let storage = graph.inc_storage();
    let active_ref = |reference: RefId| graph.refs().generic().is_active(reference);
    match node.kind {
        NodeKind::VERTEX => Vec::new(),
        NodeKind::EDGE => graph
            .topo()
            .edges()
            .wires_of(EdgeId::from_node_id(node))
            .map(NodeId::from)
            .collect(),
        NodeKind::CO_EDGE => {
            let co_edge = CoEdgeId::from_node_id(node);
            if !co_edge.is_valid_for(storage.nb_co_edges()) || storage.is_removed(co_edge) {
                return Vec::new();
            }
            let wire = storage.co_edge(co_edge).parent_wire_id;
            if wire.is_valid_for(storage.nb_wires()) {
                vec![wire.into()]
            } else {
                Vec::new()
            }
        }
        NodeKind::WIRE => storage
            .wire_relations(WireId::from_node_id(node))
            .parent_wire_ref_ids
            .iter()
            .filter(|r| active_ref((**r).into()))
            .map(|r| storage.wire_ref(*r).parent_face_id.into())
            .collect(),
        NodeKind::FACE => storage
            .face_relations(FaceId::from_node_id(node))
            .parent_face_ref_ids
            .iter()
            .filter(|r| active_ref((**r).into()))
            .map(|r| storage.face_ref(*r).parent_shell_id.into())
            .collect(),
        NodeKind::SHELL => storage
            .shell_relations(ShellId::from_node_id(node))
            .parent_shell_ref_ids
            .iter()
            .filter(|r| active_ref((**r).into()))
            .map(|r| storage.shell_ref(*r).parent_solid_id.into())
            .collect(),
        NodeKind::OCCURRENCE => occurrence_products(graph, OccurrenceId::from_node_id(node)),
        _ => placing_occurrences(graph, node),
    }
}

// Every active product with an active reference placing the occurrence (index order).
fn occurrence_products(graph: &BRepGraph, occurrence: OccurrenceId) -> Vec<NodeId> {
    let storage = graph.inc_storage();
    (0..storage.nb_products())
        .map(ProductId::new)
        .filter(|product| !storage.is_removed(*product))
        .filter(|product| {
            storage
                .product_relations(*product)
                .occurrence_ref_ids
                .iter()
                .any(|reference| {
                    graph.refs().generic().is_active((*reference).into())
                        && storage.occurrence_ref(*reference).child_occurrence_id == occurrence
                })
        })
        .map(NodeId::from)
        .collect()
}

// The active occurrences that place a topology node or product through an active
// reference (the other kinds have none).
fn placing_occurrences(graph: &BRepGraph, node: NodeId) -> Vec<NodeId> {
    if !NodeKind::is_topology_kind(node.kind) && node.kind != NodeKind::PRODUCT {
        return Vec::new();
    }
    let storage = graph.inc_storage();
    storage
        .occurrence_refs_of_node(node)
        .iter()
        .filter(|reference| graph.refs().generic().is_active((**reference).into()))
        .map(|reference| storage.occurrence_ref(*reference).child_occurrence_id)
        .filter(|occurrence| {
            occurrence.is_valid_for(storage.nb_occurrences()) && !storage.is_removed(*occurrence)
        })
        .map(NodeId::from)
        .collect()
}
