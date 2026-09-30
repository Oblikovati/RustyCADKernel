// Rust translation of checkCrossReferenceBounds of
// ModelingData/TKBRep/BRepGraph/BRepGraph_Validate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cross-reference bounds check of the audit.
use super::{Issue, Severity, is_entity_removed, is_valid_node_id, report};
use crate::brep_graph::iterator::{
    CoEdgeIterator, CompSolidIterator, CompoundIterator, EdgeIterator, OccurrenceIterator,
    ProductIterator, ShellIterator, SolidIterator, WireIterator,
};
use crate::brep_graph::refs_iterator::{
    CoEdgesOfWire, RefsChildOfCompound, RefsFaceOfShell, RefsOccurrenceOfProduct, RefsShellOfSolid,
    RefsSolidOfCompSolid,
};
use crate::brep_graph::reverse_iterator::CoEdgesOfEdge;
use crate::brep_graph::{BRepGraph, CoEdgeId, NodeId, NodeKind, VertexRefId};
use crate::brep_graph_inc::CoEdgeDef;

/// `checkCrossReferenceBounds`: every stored child id resolves inside its store.
pub(super) fn check_cross_reference_bounds(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    check_edge_vertex_bounds(graph, issues);
    check_co_edge_bounds(graph, issues);
    check_wire_bounds(graph, issues);
    check_container_bounds(graph, issues);
    check_occurrence_children(graph, issues);
}

// The start and end vertex references of each edge resolve inside the vertex store.
fn check_edge_vertex_bounds(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut edges = EdgeIterator::new(graph);
    while edges.more() {
        let edge = edges.current();
        let messages = [
            "EdgeDef.StartVertexRefId resolves to out-of-bounds ChildVertexId",
            "EdgeDef.EndVertexRefId resolves to out-of-bounds ChildVertexId",
        ];
        let references = [edge.start_vertex_ref_id, edge.end_vertex_ref_id];
        for (reference, message) in references.into_iter().zip(messages) {
            if is_out_of_bounds_vertex(graph, reference) {
                report(issues, Severity::Error, edges.current_id(), message);
            }
        }
        edges.next();
    }
}

fn is_out_of_bounds_vertex(graph: &BRepGraph, reference: VertexRefId) -> bool {
    if !reference.is_valid() {
        return false;
    }
    let vertex = graph.refs().vertices().entry(reference).child_vertex_id;
    vertex.is_valid() && !is_valid_node_id(graph, vertex)
}

// The face, edge, seam pairing and p-curve bounds of each coedge.
fn check_co_edge_bounds(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let nb_curves2d = graph.topo().geometry().nb_co_edge_curves2d();
    let mut co_edges = CoEdgeIterator::new(graph);
    while co_edges.more() {
        let (co_edge_id, co_edge) = (co_edges.current_id(), co_edges.current());
        if co_edge.face_id.is_valid() && !is_valid_node_id(graph, co_edge.face_id) {
            report(
                issues,
                Severity::Error,
                co_edge_id,
                "CoEdgeDef.FaceId out of bounds",
            );
        }
        let edge = co_edge.child_edge_id;
        if edge.is_valid() && !is_valid_node_id(graph, edge) {
            report(
                issues,
                Severity::Error,
                co_edge_id,
                "CoEdgeDef.ChildEdgeId out of bounds",
            );
        }
        if co_edge.child_edge_id.is_valid() && co_edge.face_id.is_valid() {
            check_seam_pair(graph, co_edge_id, co_edge, issues);
        }
        let curve2d = co_edge.curve2d_rep_id;
        if curve2d.is_valid() && !curve2d.is_valid_for(nb_curves2d) {
            let text = "CoEdgeDef.Curve2DRepId out of bounds";
            report(issues, Severity::Error, co_edge_id, text);
        }
        co_edges.next();
    }
}

// Seam consistency: at most 2 coedges per (ChildEdgeId, FaceId) pair; a pair must
// have opposite orientations.
fn check_seam_pair(
    graph: &BRepGraph,
    co_edge_id: CoEdgeId,
    co_edge: &CoEdgeDef,
    issues: &mut Vec<Issue>,
) {
    let (same_face_count, has_opposite) = count_same_face(graph, co_edge_id, co_edge);
    if same_face_count > 2 {
        let text = "More than 2 CoEdges share the same (ChildEdgeId, FaceId)";
        report(issues, Severity::Error, co_edge_id, text);
    } else if same_face_count == 2 && !has_opposite {
        let text = "Seam pair CoEdges have the same Orientation";
        report(issues, Severity::Error, co_edge_id, text);
    }
}

// The active coedges of the edge on the same face, and whether another one of them
// has the opposite orientation.
fn count_same_face(graph: &BRepGraph, co_edge_id: CoEdgeId, co_edge: &CoEdgeDef) -> (u32, bool) {
    let siblings = graph.topo().edges().co_edges(co_edge.child_edge_id);
    let mut iterator = CoEdgesOfEdge::new(graph, siblings);
    let (mut count, mut has_opposite) = (0u32, false);
    while iterator.more() {
        let other = iterator.definition();
        if other.face_id == co_edge.face_id {
            count += 1;
            let is_other = iterator.current_id() != co_edge_id;
            has_opposite |= is_other && other.orientation != co_edge.orientation;
        }
        iterator.next();
    }
    (count, has_opposite)
}

// The coedges of each wire and their edges resolve inside their stores.
fn check_wire_bounds(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut wires = WireIterator::new(graph);
    while wires.more() {
        let wire_id = wires.current_id();
        let mut co_edges = CoEdgesOfWire::new(graph, wire_id);
        while co_edges.more() {
            let co_edge_id = co_edges.current_id();
            if co_edge_id.is_valid() && !is_valid_node_id(graph, co_edge_id) {
                let text = "WireDef.CoEdgeUsage CoEdgeIdx out of bounds";
                report(issues, Severity::Error, wire_id, text);
            }
            let edge = graph.topo().co_edges().definition(co_edge_id).child_edge_id;
            if edge.is_valid() && !is_valid_node_id(graph, edge) {
                let text = "WireDef.CoEdgeUsage EdgeIdx out of bounds";
                report(issues, Severity::Error, wire_id, text);
            }
            co_edges.next();
        }
        wires.next();
    }
}

// The children of compounds, compsolids, shells, solids and products.
fn check_container_bounds(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    check_compound_children(graph, issues);
    check_comp_solid_children(graph, issues);
    check_shell_children(graph, issues);
    check_solid_children(graph, issues);
    check_product_children(graph, issues);
}

// Reports `text` on `parent` when `child` is valid but outside its store.
fn report_out_of_bounds(
    graph: &BRepGraph,
    parent: NodeId,
    child: NodeId,
    text: &str,
    issues: &mut Vec<Issue>,
) {
    if child.is_valid() && !is_valid_node_id(graph, child) {
        report(issues, Severity::Error, parent, text);
    }
}

fn check_compound_children(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut compounds = CompoundIterator::new(graph);
    while compounds.more() {
        let compound = compounds.current_id();
        for reference in RefsChildOfCompound::new(graph, compound) {
            let child = graph.refs().children().entry(reference).child_node_id;
            let text = "CompoundDef.ChildNodeId out of bounds";
            report_out_of_bounds(graph, compound.into(), child, text, issues);
        }
        compounds.next();
    }
}

fn check_comp_solid_children(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut comp_solids = CompSolidIterator::new(graph);
    while comp_solids.more() {
        let comp_solid = comp_solids.current_id();
        for reference in RefsSolidOfCompSolid::new(graph, comp_solid) {
            let child = graph.refs().solids().entry(reference).child_solid_id;
            let text = "CompSolidDef.ChildSolidId out of bounds";
            report_out_of_bounds(graph, comp_solid.into(), child.into(), text, issues);
        }
        comp_solids.next();
    }
}

fn check_shell_children(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut shells = ShellIterator::new(graph);
    while shells.more() {
        let shell = shells.current_id();
        for reference in RefsFaceOfShell::new(graph, shell) {
            let child = graph.refs().faces().entry(reference).child_face_id;
            let text = "ShellDef.FaceUsage FaceIdx out of bounds";
            report_out_of_bounds(graph, shell.into(), child.into(), text, issues);
        }
        shells.next();
    }
}

fn check_solid_children(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut solids = SolidIterator::new(graph);
    while solids.more() {
        let solid = solids.current_id();
        for reference in RefsShellOfSolid::new(graph, solid) {
            let child = graph.refs().shells().entry(reference).child_shell_id;
            let text = "SolidDef.ShellUsage ShellIdx out of bounds";
            report_out_of_bounds(graph, solid.into(), child.into(), text, issues);
        }
        solids.next();
    }
}

fn check_product_children(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut products = ProductIterator::new(graph);
    while products.more() {
        let product = products.current_id();
        for reference in RefsOccurrenceOfProduct::new(graph, product) {
            let child = graph
                .refs()
                .occurrences()
                .entry(reference)
                .child_occurrence_id;
            let text = "ProductDef.OccurrenceRefId out of bounds";
            report_out_of_bounds(graph, product.into(), child.into(), text, issues);
        }
        products.next();
    }
}

// Each occurrence places a live product or topology node that is in range.
fn check_occurrence_children(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut occurrences = OccurrenceIterator::new(graph);
    while occurrences.more() {
        let occurrence = occurrences.current_id();
        let child = occurrences.current().child_node_id;
        if let Some(text) = occurrence_child_issue(graph, child) {
            report(issues, Severity::Error, occurrence, text);
        }
        occurrences.next();
    }
}

// The issue text of an occurrence child, `None` when the child is acceptable.
fn occurrence_child_issue(graph: &BRepGraph, child: NodeId) -> Option<&'static str> {
    if !child.is_valid() {
        return Some("OccurrenceDef.ChildNodeId invalid");
    }
    let kind = child.kind;
    if kind != NodeKind::PRODUCT && !NodeKind::is_topology_kind(kind) {
        return Some(if kind == NodeKind::OCCURRENCE {
            "OccurrenceDef.ChildNodeId cannot reference an Occurrence"
        } else {
            "OccurrenceDef.ChildNodeId kind is not Product or a topology kind"
        });
    }
    if !is_valid_node_id(graph, child) || is_entity_removed(graph, child) {
        return Some("OccurrenceDef.ChildNodeId invalid");
    }
    None
}
