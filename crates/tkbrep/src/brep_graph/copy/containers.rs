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
// Rust translation of `ensureShell`, `ensureSolid`, `ensureNode`, `ensureCompound` and
// `ensureCompSolid` of ModelingData/TKBRep/BRepGraph/BRepGraph_Copy.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The memoised copies of the container nodes and the per-kind dispatch.
use super::assembly::{ensure_occurrence, ensure_product};
use super::context::CopyContext;
use super::topology::{ensure_edge, ensure_face, ensure_vertex, ensure_wire, ref_item};
use crate::brep::errors::program_error;
use crate::brep_graph::refs_iterator::{
    RefsChildOfCompound, RefsFaceOfShell, RefsShellOfSolid, RefsSolidOfCompSolid,
};
use crate::brep_graph::{
    ChildRefId, CompSolidId, CompoundId, FaceRefId, NodeId, NodeKind, OccurrenceId, ProductId,
    ShellId, ShellRefId, SolidId, TypedId,
};
use rustycad_tkernel::standard::Failure;

/// `ensureShell(theCtx, theSrcId)`: the copy of a source shell, bound before its faces
/// are appended in reference order.
pub(crate) fn ensure_shell(context: &mut CopyContext<'_>, source: ShellId) -> Result<ShellId, Failure> {
    if !source.is_valid_in(&context.source().topo().shells()) {
        return Ok(ShellId::invalid());
    }
    if let Some(existing) = context.existing(source) {
        return Ok(existing);
    }
    let copied = context.target_mut().editor().shells().add();
    context.copy_removed(source, copied);
    context.bind_node(source, copied);
    let refs: Vec<FaceRefId> = RefsFaceOfShell::new(context.source(), source).collect();
    for reference in refs {
        let entry = context.source().refs().faces().entry(reference);
        let (face, orientation) = (entry.child_face_id, entry.orientation);
        let new_face = ensure_face(context, face)?;
        let new_ref = context
            .target_mut()
            .editor()
            .shells()
            .append(copied, new_face, orientation)?;
        context.copy_removed(reference, new_ref);
        context.bind_item(ref_item(reference), ref_item(new_ref));
    }
    Ok(copied)
}

/// `ensureSolid(theCtx, theSrcId)`: the copy of a source solid, bound before its shells
/// are appended in reference order.
pub(crate) fn ensure_solid(context: &mut CopyContext<'_>, source: SolidId) -> Result<SolidId, Failure> {
    if !source.is_valid_in(&context.source().topo().solids()) {
        return Ok(SolidId::invalid());
    }
    if let Some(existing) = context.existing(source) {
        return Ok(existing);
    }
    let copied = context.target_mut().editor().solids().add();
    context.copy_removed(source, copied);
    context.bind_node(source, copied);
    let refs: Vec<ShellRefId> = RefsShellOfSolid::new(context.source(), source).collect();
    for reference in refs {
        let entry = context.source().refs().shells().entry(reference);
        let (shell, orientation) = (entry.child_shell_id, entry.orientation);
        let new_shell = ensure_shell(context, shell)?;
        let new_ref = context
            .target_mut()
            .editor()
            .solids()
            .append(copied, new_shell, orientation)?;
        context.copy_removed(reference, new_ref);
        context.bind_item(ref_item(reference), ref_item(new_ref));
    }
    Ok(copied)
}

/// `ensureNode(theCtx, theSrcNodeId)`: the copy of a node of any kind; coedges are
/// copied with their wires only, and the invalid kinds copy nothing.
pub(crate) fn ensure_node(context: &mut CopyContext<'_>, source: NodeId) -> Result<(), Failure> {
    let index = source.index;
    match source.kind {
        NodeKind::VERTEX => ensure_vertex(context, TypedId::new(index)).map(drop),
        NodeKind::EDGE => ensure_edge(context, TypedId::new(index)).map(drop),
        NodeKind::WIRE => ensure_wire(context, TypedId::new(index)).map(drop),
        NodeKind::FACE => ensure_face(context, TypedId::new(index)).map(drop),
        NodeKind::SHELL => ensure_shell(context, TypedId::new(index)).map(drop),
        NodeKind::SOLID => ensure_solid(context, TypedId::new(index)).map(drop),
        NodeKind::COMPOUND => ensure_compound(context, TypedId::new(index)).map(drop),
        NodeKind::COMP_SOLID => ensure_comp_solid(context, TypedId::new(index)).map(drop),
        NodeKind::PRODUCT => ensure_product(context, ProductId::new(index)).map(drop),
        NodeKind::OCCURRENCE => ensure_occurrence(context, OccurrenceId::new(index)).map(drop),
        _ => Ok(()),
    }
}

/// `ensureCompound(theCtx, theSrcId)`: an empty compound bound before its children
/// recurse (so a self-referencing compound chain terminates), then each copied child
/// appended in reference order. A refused append raises Standard_ProgramError.
pub(crate) fn ensure_compound(
    context: &mut CopyContext<'_>,
    source: CompoundId,
) -> Result<CompoundId, Failure> {
    if !source.is_valid_in(&context.source().topo().compounds()) {
        return Ok(CompoundId::invalid());
    }
    if let Some(existing) = context.existing(source) {
        return Ok(existing);
    }
    let copied = context.target_mut().editor().compounds().add(&[]);
    context.copy_removed(source, copied);
    context.bind_node(source, copied);
    let refs: Vec<ChildRefId> = RefsChildOfCompound::new(context.source(), source).collect();
    for reference in refs {
        let entry = context.source().refs().children().entry(reference);
        let (child, orientation) = (entry.child_node_id, entry.orientation);
        ensure_node(context, child)?;
        let mapped = context.mapped_node(child);
        if !mapped.is_valid() {
            continue;
        }
        let new_ref = context
            .target_mut()
            .editor()
            .compounds()
            .append(copied, mapped, orientation)?;
        if !new_ref.is_valid() {
            return Err(program_error(
                "BRepGraph_Copy: failed to copy compound child ref",
                &format!("{mapped:?} under {copied:?} (expected a valid ChildRefId)"),
            ));
        }
        context.copy_removed(reference, new_ref);
        context.bind_item(ref_item(reference), ref_item(new_ref));
    }
    Ok(copied)
}

/// `ensureCompSolid(theCtx, theSrcId)`: the solids first, then the compsolid over them;
/// its solid references follow the source's pairwise.
pub(crate) fn ensure_comp_solid(
    context: &mut CopyContext<'_>,
    source: CompSolidId,
) -> Result<CompSolidId, Failure> {
    if !source.is_valid_in(&context.source().topo().comp_solids()) {
        return Ok(CompSolidId::invalid());
    }
    if let Some(existing) = context.existing(source) {
        return Ok(existing);
    }
    let graph = context.source();
    let solids: Vec<SolidId> = RefsSolidOfCompSolid::new(graph, source)
        .map(|reference| graph.refs().solids().entry(reference).child_solid_id)
        .collect();
    let mut new_solids = Vec::with_capacity(solids.len());
    for solid in solids {
        new_solids.push(ensure_solid(context, solid)?);
    }
    let copied = context.target_mut().editor().comp_solids().add(&new_solids);
    context.copy_removed(source, copied);
    context.bind_node(source, copied);
    let old_refs = context.source().topo().comp_solids().relations(source).solid_ref_ids.clone();
    let new_refs = relations_solid_refs(context, copied);
    for (old, new) in old_refs.into_iter().zip(new_refs) {
        context.copy_removed(old, new);
        context.bind_item(ref_item(old), ref_item(new));
    }
    Ok(copied)
}

// The solid references of a copied compsolid. A compsolid the editor refused (a removed
// solid) is the invalid id, whose relations the native code reads out of bounds; it has
// none here.
fn relations_solid_refs(
    context: &CopyContext<'_>,
    comp_solid: CompSolidId,
) -> Vec<crate::brep_graph::SolidRefId> {
    let target = context.target();
    if !comp_solid.is_valid_in(&target.topo().comp_solids()) {
        return Vec::new();
    }
    target.topo().comp_solids().relations(comp_solid).solid_ref_ids.clone()
}
