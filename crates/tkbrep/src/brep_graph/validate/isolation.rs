// Rust translation of checkRemovedNodeIsolation and checkWireConnectivity of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Validate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The removed-node isolation check (both modes) and the wire connectivity check
//! (audit).
use super::{Issue, Severity, is_entity_removed, report};
use crate::brep_graph::iterator::{CoEdgeIterator, EdgeIterator, WireIterator};
use crate::brep_graph::refs_iterator::CoEdgesOfWire;
use crate::brep_graph::{BRepGraph, CoEdgeId, NodeId, VertexRefId, WireId};
use crate::brep_graph_inc::{CoEdgeDef, ParityOrientation};

/// `checkRemovedNodeIsolation`: no active edge, wire or coedge names a removed
/// vertex, edge or face.
pub(super) fn check_removed_node_isolation(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    check_edge_vertices_live(graph, issues);
    check_wire_edges_live(graph, issues);
    check_co_edge_faces_live(graph, issues);
}

fn check_edge_vertices_live(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut edges = EdgeIterator::new(graph);
    while edges.more() {
        let edge = edges.current();
        if names_removed_vertex(graph, edge.start_vertex_ref_id) {
            let text = "Non-removed EdgeDef references removed StartVertexEntity";
            report(issues, Severity::Error, edges.current_id(), text);
        }
        if names_removed_vertex(graph, edge.end_vertex_ref_id) {
            let text = "Non-removed EdgeDef references removed EndVertexEntity";
            report(issues, Severity::Error, edges.current_id(), text);
        }
        edges.next();
    }
}

fn names_removed_vertex(graph: &BRepGraph, reference: VertexRefId) -> bool {
    if !reference.is_valid() {
        return false;
    }
    let vertex = graph.refs().vertices().entry(reference).child_vertex_id;
    vertex.is_valid() && is_entity_removed(graph, vertex)
}

fn check_wire_edges_live(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut wires = WireIterator::new(graph);
    while wires.more() {
        let wire = wires.current_id();
        let has_removed_edge = CoEdgesOfWire::new(graph, wire).any(|co_edge| {
            let edge = graph.topo().co_edges().definition(co_edge).child_edge_id;
            edge.is_valid() && is_entity_removed(graph, edge)
        });
        if has_removed_edge {
            let text = "Non-removed WireDef references removed EdgeDef";
            report(issues, Severity::Error, wire, text);
        }
        wires.next();
    }
}

fn check_co_edge_faces_live(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut co_edges = CoEdgeIterator::new(graph);
    while co_edges.more() {
        let face = co_edges.current().face_id;
        if face.is_valid() && is_entity_removed(graph, face) {
            let text = "Non-removed CoEdgeDef.FaceId references removed FaceDef";
            report(issues, Severity::Error, co_edges.current_id(), text);
        }
        co_edges.next();
    }
}

/// `checkWireConnectivity`: the oriented end vertex of each stored coedge is the
/// oriented start vertex of the next one (a warning otherwise).
pub(super) fn check_wire_connectivity(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut wires = WireIterator::new(graph);
    while wires.more() {
        check_one_wire_connectivity(graph, wires.current_id(), issues);
        wires.next();
    }
}

// The raw coedge list, removed coedges included, as natively.
fn check_one_wire_connectivity(graph: &BRepGraph, wire: WireId, issues: &mut Vec<Issue>) {
    let co_edges = &graph.topo().wires().relations(wire).co_edge_ids;
    if co_edges.len() < 2 {
        return;
    }
    for index in 1..co_edges.len() {
        let Some((end, start)) = junction_vertices(graph, co_edges[index - 1], co_edges[index])
        else {
            continue;
        };
        if end.is_valid() && start.is_valid() && end != start {
            let text = format!(
                "Wire edges not connected: edge[{}] end != edge[{}] start",
                (index - 1) as i32,
                index as i32
            );
            report(issues, Severity::Warning, wire, &text);
        }
    }
}

// The oriented end vertex of `previous` and start vertex of `current`; `None` when a
// coedge or its edge is out of range (the native loop skips the pair).
fn junction_vertices(
    graph: &BRepGraph,
    previous: CoEdgeId,
    current: CoEdgeId,
) -> Option<(NodeId, NodeId)> {
    let nb_co_edges = graph.topo().co_edges().nb();
    if !previous.is_valid_for(nb_co_edges) || !current.is_valid_for(nb_co_edges) {
        return None;
    }
    let previous = graph.topo().co_edges().definition(previous);
    let current = graph.topo().co_edges().definition(current);
    let nb_edges = graph.topo().edges().nb();
    if !previous.child_edge_id.is_valid_for(nb_edges)
        || !current.child_edge_id.is_valid_for(nb_edges)
    {
        return None;
    }
    Some((
        oriented_vertex(graph, previous, true),
        oriented_vertex(graph, current, false),
    ))
}

// The oriented end (`at_end`) or start vertex of a coedge's edge.
fn oriented_vertex(graph: &BRepGraph, co_edge: &CoEdgeDef, at_end: bool) -> NodeId {
    let edge = graph.topo().edges().definition(co_edge.child_edge_id);
    let is_forward = co_edge.orientation == ParityOrientation::FORWARD;
    let reference = if is_forward == at_end {
        edge.end_vertex_ref_id
    } else {
        edge.start_vertex_ref_id
    };
    ref_vertex(graph, reference)
}

// The vertex of a reference, the invalid node id for an invalid reference.
fn ref_vertex(graph: &BRepGraph, reference: VertexRefId) -> NodeId {
    if !reference.is_valid() {
        return NodeId::default();
    }
    graph
        .refs()
        .vertices()
        .entry(reference)
        .child_vertex_id
        .into()
}
