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
// Rust translation of the supplement-layer and UID integrity steps of
// BRepGraph_Validate::Perform in ModelingData/TKBRep/BRepGraph/BRepGraph_Validate.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The supplement-layer and UID round-trip checks of the audit.
use super::{Issue, Severity, report};
use crate::brep_graph::editor::BOUNDARY_KINDS;
use crate::brep_graph::layer_topo_supplement::LayerTopoSupplement;
use crate::brep_graph::{BRepGraph, NodeId, NodeKind};
use rustycad_tkernel::tcollection::AsciiString;

/// The registered topology supplement layer validates itself; its
/// Standard_Failure becomes one error whose text carries the failure's `what()`.
pub(super) fn check_topo_supplement(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let layer = graph.layer_registry().find::<LayerTopoSupplement>();
    if layer.is_null() {
        return;
    }
    let Err(failure) = layer.validate() else {
        return;
    };
    let mut text: Vec<u8> = b"LayerTopoSupplement invalid: ".to_vec();
    text.extend_from_slice(failure.what());
    issues.push(Issue {
        sev: Severity::Error,
        node_id: NodeId::default(),
        description: AsciiString::from_bytes(text),
    });
}

/// UID integrity: every active node has a valid UID that the UIDs view knows and
/// that resolves back to the node, kind by kind in the native order.
pub(super) fn check_uids(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    for kind in BOUNDARY_KINDS {
        check_uid_kind(graph, kind, issues);
    }
}

// `checkUIDKind`.
fn check_uid_kind(graph: &BRepGraph, kind: NodeKind, issues: &mut Vec<Issue>) {
    for index in 0..graph.topo().generic().nb(kind) {
        let node = NodeId::new(kind, index);
        if graph.topo_entity(node).is_none() || node.is_removed(graph) {
            continue;
        }
        if let Some(text) = uid_issue(graph, node) {
            report(issues, Severity::Error, node, text);
        }
    }
}

fn uid_issue(graph: &BRepGraph, node: NodeId) -> Option<&'static str> {
    let uids = graph.uids();
    let uid = uids.of_node(node);
    if !uid.is_valid() {
        return Some("Node has missing or invalid UID in deep audit");
    }
    if !uids.has_node(&uid) {
        return Some("Node UID is not present in UIDs view for this generation");
    }
    if uids.node_id_from(&uid) != node {
        return Some("Node UID does not round-trip to originating NodeId");
    }
    None
}
