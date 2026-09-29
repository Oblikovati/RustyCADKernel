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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_Tool.hxx and .cxx
// (the usage aliases and the anonymous-namespace helpers)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Tool`: the geometry access of a graph, the analogue of `BRep_Tool`.
//!
//! Geometry is stored in the definition frame; the usage values (`VertexUsage`...,
//! the `BRepGraphInc` instances) carry the location of one use, which the 3D accessors
//! apply. The nested classes `Vertex`, `Edge`, `CoEdge`, `Face`, `Wire` and `Shell` are
//! the modules [`vertex`], [`edge`], [`co_edge`], [`face`], [`wire`] and [`shell`];
//! overloads taking a reference id or a usage carry the suffix `_ref` or `_usage`.
//! Members that raise natively (adaptor construction, curve transformation) return
//! `Result`.
//!
//! The derived edge, coedge, wire and shell flags (`Degenerated`, `SameRange`,
//! `IsClosed`...) read `BRepGraph_CacheDerivedState` natively, which memoises the
//! computations of [`cache_derived_state`](crate::brep_graph::cache_derived_state) per
//! OwnGen. Until the caches area installs that service, the private `derived_state`
//! module computes the value on each call with the cache's answer for an inactive
//! entity; the result is the same as long as no mutation leaves an OwnGen unchanged.
pub mod co_edge;
mod derived_state;
pub mod edge;
mod edge_adaptors;
pub mod face;
mod face_adaptors;
pub mod shell;
pub mod vertex;
pub mod wire;

use crate::brep_graph::{BRepGraph, FaceId, FaceRefId, ShellId, ShellRefId, WireId, WireRefId};
use crate::brep_graph_inc::{
    CoEdgeInstance, FaceInstance, ShellInstance, VertexInstance, WireInstance,
};

/// `BRepGraph_Tool::VertexUsage`.
pub type VertexUsage = VertexInstance;
/// `BRepGraph_Tool::CoEdgeUsage`.
pub type CoEdgeUsage = CoEdgeInstance;
/// `BRepGraph_Tool::FaceUsage`.
pub type FaceUsage = FaceInstance;
/// `BRepGraph_Tool::WireUsage`.
pub type WireUsage = WireInstance;
/// `BRepGraph_Tool::ShellUsage`.
pub type ShellUsage = ShellInstance;

// `faceOf`: the active face of an active face reference, invalid otherwise.
fn face_of_ref(graph: &BRepGraph, face_ref: FaceRefId) -> FaceId {
    if !face_ref.is_valid() || graph.refs().generic().is_removed(face_ref.into()) {
        return FaceId::default();
    }
    let face = graph.inc_storage().face_ref(face_ref).child_face_id;
    if face.is_valid() && !face.is_removed(graph) {
        face
    } else {
        FaceId::default()
    }
}

// `wireOf`: the active wire of an active wire reference, invalid otherwise.
fn wire_of_ref(graph: &BRepGraph, wire_ref: WireRefId) -> WireId {
    if !wire_ref.is_valid() || graph.refs().generic().is_removed(wire_ref.into()) {
        return WireId::default();
    }
    let wire = graph.inc_storage().wire_ref(wire_ref).child_wire_id;
    if wire.is_valid() && !wire.is_removed(graph) {
        wire
    } else {
        WireId::default()
    }
}

// `shellOf`: the active shell of an active shell reference, invalid otherwise.
fn shell_of_ref(graph: &BRepGraph, shell_ref: ShellRefId) -> ShellId {
    if !shell_ref.is_valid() || graph.refs().generic().is_removed(shell_ref.into()) {
        return ShellId::default();
    }
    let shell = graph.inc_storage().shell_ref(shell_ref).child_shell_id;
    if shell.is_valid() && !shell.is_removed(graph) {
        shell
    } else {
        ShellId::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_uses_read_as_null() {
        let mut graph = BRepGraph::new();
        let storage = graph.inc_storage_mut();
        let (edge, face, co_edge) = (
            storage.append_edge(),
            storage.append_face(),
            storage.append_co_edge(),
        );
        let curve = storage.append_edge_curve3d_rep();
        storage.change_edge(edge).curve3d_rep_id = curve;
        storage.change_edge_curve3d_rep(curve).param_last = 2.0;
        assert_eq!(edge::range(&graph, edge), (0.0, 2.0));
        assert!(edge::curve(&graph, edge).is_null());
        assert_eq!(co_edge::range(&graph, co_edge), (0.0, 0.0));
        assert!(co_edge::pcurve(&graph, co_edge).is_null());
        assert!(face::surface(&graph, face).is_null());
    }

    #[test]
    fn reference_helpers_need_active_entries() {
        let graph = BRepGraph::new();
        assert!(!face_of_ref(&graph, FaceRefId::new(0)).is_valid());
        assert!(!wire_of_ref(&graph, WireRefId::default()).is_valid());
        assert!(!shell_of_ref(&graph, ShellRefId::new(2)).is_valid());
    }
}
