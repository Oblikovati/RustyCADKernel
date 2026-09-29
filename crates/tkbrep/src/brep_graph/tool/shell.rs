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
// Rust translation of BRepGraph_Tool::Shell (ModelingData/TKBRep/BRepGraph/
// BRepGraph_Tool.hxx and .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Tool::Shell`: the shell closure and face count.
use super::{ShellUsage, derived_state, shell_of_ref};
use crate::brep_graph::{BRepGraph, ShellId, ShellRefId};
use rustycad_tkmath::toploc::Location;

/// `Usage(theGraph, theShellRef)`: the active shell of an active reference with the
/// reference orientation; the invalid usage otherwise.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, ShellRefId, tool};
/// assert!(!tool::shell::usage(&BRepGraph::new(), ShellRefId::new(0)).is_valid());
/// ```
pub fn usage(graph: &BRepGraph, shell_ref: ShellRefId) -> ShellUsage {
    if !shell_ref.is_valid() || graph.refs().generic().is_removed(shell_ref.into()) {
        return ShellUsage::default();
    }
    let entry = graph.inc_storage().shell_ref(shell_ref);
    if !entry.child_shell_id.is_valid() || entry.child_shell_id.is_removed(graph) {
        return ShellUsage::default();
    }
    ShellUsage {
        def_id: entry.child_shell_id,
        location: Location::default(),
        orientation: entry.orientation.orientation(),
    }
}

/// `IsClosed(theGraph, theShell)`: every non-degenerate boundary edge of the active
/// faces is used exactly twice.
pub fn is_closed(graph: &BRepGraph, shell: ShellId) -> bool {
    derived_state::shell_is_closed(graph, shell)
}

/// `IsClosed(theGraph, theShellRef)`.
pub fn is_closed_ref(graph: &BRepGraph, shell_ref: ShellRefId) -> bool {
    let shell = shell_of_ref(graph, shell_ref);
    shell.is_valid() && is_closed(graph, shell)
}

/// `NbFaces(theGraph, theShell)`: the face references (removed included).
pub fn nb_faces(graph: &BRepGraph, shell: ShellId) -> u32 {
    graph
        .inc_storage()
        .shell_relations(shell)
        .face_ref_ids
        .len() as u32
}

/// `NbFaces(theGraph, theShellRef)`: 0 for an inactive reference or shell.
pub fn nb_faces_ref(graph: &BRepGraph, shell_ref: ShellRefId) -> u32 {
    let shell = shell_of_ref(graph, shell_ref);
    if shell.is_valid() {
        nb_faces(graph, shell)
    } else {
        0
    }
}
