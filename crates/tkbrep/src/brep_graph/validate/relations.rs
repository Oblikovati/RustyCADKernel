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
// Rust translation of checkRelationConsistency, checkRelationFaceCountCache and
// checkIncidenceRefConsistency of ModelingData/TKBRep/BRepGraph/BRepGraph_Validate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The relation and incidence checks of the audit.
//!
//! The native sets are `NCollection_FlatMap`s whose iteration order never reaches a
//! report (a size, a membership or the first missing member ends in one fixed text), so
//! they are standard hash sets here.
use super::{Issue, Severity, is_entity_removed, is_valid_node_id, report};
use crate::brep_graph::iterator::WireIterator;
use crate::brep_graph::iterator::{EdgeIterator, FaceIterator, ShellIterator, SolidIterator};
use crate::brep_graph::refs_iterator::{
    CoEdgesOfWire, RefsFaceOfShell, RefsShellOfSolid, RefsWireOfFace,
};
use crate::brep_graph::{BRepGraph, EdgeId, FaceId, NodeId, WireId};
use std::collections::{HashMap, HashSet};

/// `checkRelationConsistency`: the wires of each edge equal the wires whose coedges
/// name it.
pub(super) fn check_relation_consistency(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let expected = expected_edge_wires(graph);
    let mut edges = EdgeIterator::new(graph);
    while edges.more() {
        check_edge_wires(
            graph,
            edges.current_id(),
            expected.get(&edges.current_id()),
            issues,
        );
        edges.next();
    }
}

// The expected edge-to-wires mapping, from the coedges of every active wire.
fn expected_edge_wires(graph: &BRepGraph) -> HashMap<EdgeId, HashSet<WireId>> {
    let mut expected: HashMap<EdgeId, HashSet<WireId>> = HashMap::new();
    let mut wires = WireIterator::new(graph);
    while wires.more() {
        let wire = wires.current_id();
        for co_edge in CoEdgesOfWire::new(graph, wire) {
            let edge = graph.topo().co_edges().definition(co_edge).child_edge_id;
            if edge.is_valid() {
                expected.entry(edge).or_default().insert(wire);
            }
        }
        wires.next();
    }
    expected
}

fn check_edge_wires(
    graph: &BRepGraph,
    edge: EdgeId,
    expected: Option<&HashSet<WireId>>,
    issues: &mut Vec<Issue>,
) {
    let expected_count = expected.map_or(0, HashSet::len);
    let actual: HashSet<WireId> = graph.topo().edges().wires_of(edge).collect();
    if actual.len() != expected_count {
        let text = wire_size_mismatch_text(graph, edge, expected_count, actual.len());
        report(issues, Severity::Error, edge, &text);
        return;
    }
    let Some(expected) = expected else {
        return;
    };
    if expected.iter().any(|wire| !actual.contains(wire)) {
        let text = "Relation Edges.Wires missing wire entry";
        report(issues, Severity::Error, edge, text);
    }
}

// The size-mismatch description, listing the actual wires (and whether each is out of
// range or removed).
fn wire_size_mismatch_text(
    graph: &BRepGraph,
    edge: EdgeId,
    expected: usize,
    actual: usize,
) -> String {
    let nb_wires = graph.topo().wires().nb();
    let listed: Vec<String> = graph
        .topo()
        .edges()
        .wires_of(edge)
        .map(|wire| {
            let is_gone = !wire.is_valid_for(nb_wires) || wire.is_removed(graph);
            format!(
                "{}{}",
                wire.index as i32,
                if is_gone { " removed" } else { "" }
            )
        })
        .collect();
    format!(
        "Relation Edges.Wires size mismatch: expected={}, actual={}, edge={}, actualWires=[{}]",
        expected as i32,
        actual as i32,
        edge.index as i32,
        listed.join(",")
    )
}

/// `checkRelationFaceCountCache`: the cached face count of each edge equals its
/// distinct active coedge faces.
pub(super) fn check_relation_face_count_cache(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut edges = EdgeIterator::new(graph);
    while edges.more() {
        let edge = edges.current_id();
        let cached = graph.topo().edges().nb_faces(edge);
        let actual = distinct_co_edge_faces(graph, edge) as u32;
        if cached != actual {
            let text = face_count_mismatch_text(graph, edge, cached, actual);
            report(issues, Severity::Error, edge, &text);
        }
        edges.next();
    }
}

// The distinct faces of the edge's active coedges that name a valid face.
fn distinct_co_edge_faces(graph: &BRepGraph, edge: EdgeId) -> usize {
    let mut faces: HashSet<FaceId> = HashSet::new();
    for &co_edge_id in graph.topo().edges().co_edges(edge) {
        let co_edge = graph.topo().co_edges().definition(co_edge_id);
        if NodeId::from(co_edge_id).is_removed(graph) || !co_edge.face_id.is_valid() {
            continue;
        }
        faces.insert(co_edge.face_id);
    }
    faces.len()
}

fn face_count_mismatch_text(graph: &BRepGraph, edge: EdgeId, cached: u32, actual: u32) -> String {
    let listed: Vec<String> = graph
        .topo()
        .edges()
        .faces_of(edge)
        .map(|face| (face.index as i32).to_string())
        .collect();
    format!(
        "Relation face-count cache mismatch: cached={} actual={} edge={} faces=[{}]",
        cached as i32,
        actual as i32,
        edge.index as i32,
        listed.join(",")
    )
}

/// `checkIncidenceRefConsistency`: face wire references are in range, shells and
/// solids do not name removed children.
pub(super) fn check_incidence_ref_consistency(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    check_face_wire_refs(graph, issues);
    check_shell_face_refs(graph, issues);
    check_solid_shell_refs(graph, issues);
}

fn check_face_wire_refs(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut faces = FaceIterator::new(graph);
    while faces.more() {
        let face = faces.current_id();
        for reference in RefsWireOfFace::new(graph, face) {
            let wire = graph.refs().wires().entry(reference).child_wire_id;
            if wire.is_valid() && !is_valid_node_id(graph, wire) {
                let text = "FaceDef.WireUsage WireIdx out of bounds";
                report(issues, Severity::Error, face, text);
            }
        }
        faces.next();
    }
}

fn check_shell_face_refs(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut shells = ShellIterator::new(graph);
    while shells.more() {
        let shell = shells.current_id();
        for reference in RefsFaceOfShell::new(graph, shell) {
            let face = graph.refs().faces().entry(reference).child_face_id;
            if face.is_valid() && is_entity_removed(graph, face) {
                let text = "ShellDef references removed FaceDef";
                report(issues, Severity::Error, shell, text);
            }
        }
        shells.next();
    }
}

fn check_solid_shell_refs(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut solids = SolidIterator::new(graph);
    while solids.more() {
        let solid = solids.current_id();
        for reference in RefsShellOfSolid::new(graph, solid) {
            let shell = graph.refs().shells().entry(reference).child_shell_id;
            if shell.is_valid() && is_entity_removed(graph, shell) {
                let text = "SolidDef references removed ShellDef";
                report(issues, Severity::Error, solid, text);
            }
        }
        solids.next();
    }
}
