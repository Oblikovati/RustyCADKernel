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

//! Direct translation of BRepGraph_RefTestTools.hxx (namespace `BRepGraph_TestTools`)
//! at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//!
//! The native overloads on `BRepGraph` and on `BRepGraphInc_Storage` share names; the
//! storage overloads here carry the `_in_storage` suffix. The graph overload of
//! `OuterWireOfFace` is `BRepGraph_Tool::Face::OuterWire`, whose p-curve adaptors can
//! raise, so it returns `Result`.
#![allow(dead_code)]
use rustycad_tkbrep::brep_graph::{
    BRepGraph, ChildRefId, CoEdgeId, CompSolidId, CompoundId, FaceId, FaceRefId, NodeId, NodeKind,
    ShellId, ShellRefId, SolidId, SolidRefId, WireId, WireRefId,
};
use rustycad_tkbrep::brep_graph::tool;
use rustycad_tkbrep::brep_graph_inc::Storage;
use rustycad_tkernel::standard::Failure;

/// `SeamPairFromStorage`: the partner coedge when `co_edge` is one half of a seam pair
/// on its face, else an invalid id.
pub fn seam_pair_from_storage(storage: &Storage, co_edge: CoEdgeId) -> CoEdgeId {
    let def = storage.co_edge(co_edge);
    if storage.is_removed(co_edge) || !def.child_edge_id.is_valid() || !def.face_id.is_valid() {
        return CoEdgeId::default();
    }
    for &other in &storage.edge_relations(def.child_edge_id).co_edge_ids {
        if other == co_edge {
            continue;
        }
        let other_def = storage.co_edge(other);
        if storage.is_removed(other) {
            continue;
        }
        if other_def.face_id == def.face_id && other_def.orientation != def.orientation {
            return other;
        }
    }
    CoEdgeId::default()
}

/// `IsSeamCoEdgeFromStorage`.
pub fn is_seam_co_edge_from_storage(storage: &Storage, co_edge: CoEdgeId) -> bool {
    seam_pair_from_storage(storage, co_edge).is_valid()
}

/// `CoEdgesOfWire(theGraph, theWireId)`.
pub fn co_edges_of_wire(graph: &BRepGraph, wire: WireId) -> &[CoEdgeId] {
    &graph.topo().wires().relations(wire).co_edge_ids
}

/// `CountCoEdgesOfWire(theGraph, theWireId)`.
pub fn count_co_edges_of_wire(graph: &BRepGraph, wire: WireId) -> u32 {
    co_edges_of_wire(graph, wire).len() as u32
}

/// `WireRefsOfFace(theGraph, theFaceId)`.
pub fn wire_refs_of_face(graph: &BRepGraph, face: FaceId) -> &[WireRefId] {
    graph.refs().wires().ids_of(face)
}

/// `CountWireRefsOfFace(theGraph, theFaceId)`.
pub fn count_wire_refs_of_face(graph: &BRepGraph, face: FaceId) -> u32 {
    wire_refs_of_face(graph, face).len() as u32
}

/// `FaceUsesWire(theGraph, theFaceId, theWireId)`.
pub fn face_uses_wire(graph: &BRepGraph, face: FaceId, wire: WireId) -> bool {
    let refs = graph.refs();
    wire_refs_of_face(graph, face)
        .iter()
        .any(|&reference| refs.wires().entry(reference).child_wire_id == wire)
}

/// `FaceRefsOfShell(theGraph, theShellId)`.
pub fn face_refs_of_shell(graph: &BRepGraph, shell: ShellId) -> &[FaceRefId] {
    graph.refs().faces().ids_of(shell)
}

/// `CountFaceRefsOfShell(theGraph, theShellId)`.
pub fn count_face_refs_of_shell(graph: &BRepGraph, shell: ShellId) -> u32 {
    face_refs_of_shell(graph, shell).len() as u32
}

/// `ShellRefsOfSolid(theGraph, theSolidId)`.
pub fn shell_refs_of_solid(graph: &BRepGraph, solid: SolidId) -> &[ShellRefId] {
    graph.refs().shells().ids_of(solid)
}

/// `CountShellRefsOfSolid(theGraph, theSolidId)`.
pub fn count_shell_refs_of_solid(graph: &BRepGraph, solid: SolidId) -> u32 {
    shell_refs_of_solid(graph, solid).len() as u32
}

/// `SolidRefsOfCompSolid(theGraph, theCompSolidId)`.
pub fn solid_refs_of_comp_solid(graph: &BRepGraph, comp_solid: CompSolidId) -> &[SolidRefId] {
    graph.refs().solids().ids_of(comp_solid)
}

/// `CountSolidRefsOfCompSolid(theGraph, theCompSolidId)`.
pub fn count_solid_refs_of_comp_solid(graph: &BRepGraph, comp_solid: CompSolidId) -> u32 {
    solid_refs_of_comp_solid(graph, comp_solid).len() as u32
}

/// `ChildRefsOfParent(theGraph, theParentId)`: empty unless the parent is a compound.
pub fn child_refs_of_parent(graph: &BRepGraph, parent: NodeId) -> &[ChildRefId] {
    if parent.kind != NodeKind::COMPOUND {
        return &[];
    }
    graph
        .refs()
        .children()
        .ids_of(CompoundId::from_node_id(parent))
}

/// `CountChildRefsOfParent(theGraph, theParentId)`.
pub fn count_child_refs_of_parent(graph: &BRepGraph, parent: NodeId) -> u32 {
    child_refs_of_parent(graph, parent).len() as u32
}

/// `CoEdgesOfWire(theStorage, theWireId)`.
pub fn co_edges_of_wire_in_storage(storage: &Storage, wire: WireId) -> &[CoEdgeId] {
    &storage.wire_relations(wire).co_edge_ids
}

/// `CountCoEdgesOfWire(theStorage, theWireId)`.
pub fn count_co_edges_of_wire_in_storage(storage: &Storage, wire: WireId) -> u32 {
    co_edges_of_wire_in_storage(storage, wire).len() as u32
}

/// `WireRefsOfFace(theStorage, theFaceId)`.
pub fn wire_refs_of_face_in_storage(storage: &Storage, face: FaceId) -> &[WireRefId] {
    &storage.face_relations(face).wire_ref_ids
}

/// `CountWireRefsOfFace(theStorage, theFaceId)`.
pub fn count_wire_refs_of_face_in_storage(storage: &Storage, face: FaceId) -> u32 {
    wire_refs_of_face_in_storage(storage, face).len() as u32
}

/// `FaceUsesWire(theStorage, theFaceId, theWireId)`.
pub fn face_uses_wire_in_storage(storage: &Storage, face: FaceId, wire: WireId) -> bool {
    let references = wire_refs_of_face_in_storage(storage, face);
    references
        .iter()
        .any(|&reference| storage.wire_ref(reference).child_wire_id == wire)
}

/// `FaceRefsOfShell(theStorage, theShellId)`.
pub fn face_refs_of_shell_in_storage(storage: &Storage, shell: ShellId) -> &[FaceRefId] {
    &storage.shell_relations(shell).face_ref_ids
}

/// `CountFaceRefsOfShell(theStorage, theShellId)`.
pub fn count_face_refs_of_shell_in_storage(storage: &Storage, shell: ShellId) -> u32 {
    face_refs_of_shell_in_storage(storage, shell).len() as u32
}

/// `ShellRefsOfSolid(theStorage, theSolidId)`.
pub fn shell_refs_of_solid_in_storage(storage: &Storage, solid: SolidId) -> &[ShellRefId] {
    &storage.solid_relations(solid).shell_ref_ids
}

/// `CountShellRefsOfSolid(theStorage, theSolidId)`.
pub fn count_shell_refs_of_solid_in_storage(storage: &Storage, solid: SolidId) -> u32 {
    shell_refs_of_solid_in_storage(storage, solid).len() as u32
}

/// `SolidRefsOfCompSolid(theStorage, theCompSolidId)`.
pub fn solid_refs_of_comp_solid_in_storage(
    storage: &Storage,
    comp_solid: CompSolidId,
) -> &[SolidRefId] {
    &storage.comp_solid_relations(comp_solid).solid_ref_ids
}

/// `CountSolidRefsOfCompSolid(theStorage, theCompSolidId)`.
pub fn count_solid_refs_of_comp_solid_in_storage(
    storage: &Storage,
    comp_solid: CompSolidId,
) -> u32 {
    solid_refs_of_comp_solid_in_storage(storage, comp_solid).len() as u32
}

/// `ChildRefsOfParent(theStorage, theParentId)`: empty unless the parent is a compound.
pub fn child_refs_of_parent_in_storage(storage: &Storage, parent: NodeId) -> &[ChildRefId] {
    if parent.kind != NodeKind::COMPOUND {
        return &[];
    }
    &storage
        .compound_relations(CompoundId::from_node_id(parent))
        .child_ref_ids
}

/// `CountChildRefsOfParent(theStorage, theParentId)`.
pub fn count_child_refs_of_parent_in_storage(storage: &Storage, parent: NodeId) -> u32 {
    child_refs_of_parent_in_storage(storage, parent).len() as u32
}

/// `OuterWireOfFace(theGraph, theFaceId)`: `BRepGraph_Tool::Face::OuterWire`.
pub fn outer_wire_of_face(graph: &BRepGraph, face: FaceId) -> Result<WireId, Failure> {
    tool::face::outer_wire(graph, face)
}

/// `OuterWireOfFace(theStorage, theFaceId)`: the child wire of the first wire reference.
pub fn outer_wire_of_face_in_storage(storage: &Storage, face: FaceId) -> WireId {
    let references = wire_refs_of_face_in_storage(storage, face);
    references
        .first()
        .map_or_else(WireId::default, |&reference| {
            storage.wire_ref(reference).child_wire_id
        })
}
