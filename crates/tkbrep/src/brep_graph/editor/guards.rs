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
// Rust translation of validateMutableNodeId, validateMutableRefId and the shared body
// of the Mut/MutRef/MutChildRef members of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView_Mut.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shared body of every `Mut*()` factory of the editor.
use super::helpers::{is_owned, node_item, program_error, ref_item};
use crate::brep_graph::mut_guard::{GuardTag, MutGuard};
use crate::brep_graph::{BRepGraph, ItemId, NodeId, RefId, TypedId};
use rustycad_tkernel::standard::Failure;

/// `Mut(theId)` of a definition kind: an inactive node raises Standard_ProgramError
/// ("invalid node id"), then an owned node ("owned node"), then a guarded one
/// ("guard already active"); otherwise the new guard.
pub(crate) fn mutate_node<T>(
    graph: &mut BRepGraph,
    id: TypedId<T>,
) -> Result<MutGuard<'_, T>, Failure>
where
    T: GuardTag,
    TypedId<T>: Into<NodeId>,
{
    let node: NodeId = id.into();
    let item = node_item(node);
    if !graph.topo().generic().is_active(node) {
        return Err(program_error(
            "BRepGraph::EditorView::Mut*(): invalid node id",
            item,
        ));
    }
    if is_owned(graph, item) {
        return Err(program_error(
            "BRepGraph::EditorView::Mut*(): owned node",
            item,
        ));
    }
    guard_unless_guarded(graph, id, item)
}

/// `MutRef(theId)` of a reference kind: the reference counterpart of
/// [`mutate_node`] ("invalid reference id", "owned reference").
pub(crate) fn mutate_ref<T>(
    graph: &mut BRepGraph,
    id: TypedId<T>,
) -> Result<MutGuard<'_, T>, Failure>
where
    T: GuardTag,
    TypedId<T>: Into<RefId>,
{
    let reference: RefId = id.into();
    let item = ref_item(reference);
    if !graph.refs().generic().is_active(reference) {
        return Err(program_error(
            "BRepGraph::EditorView::Mut*(): invalid reference id",
            item,
        ));
    }
    if is_owned(graph, item) {
        return Err(program_error(
            "BRepGraph::EditorView::Mut*(): owned reference",
            item,
        ));
    }
    guard_unless_guarded(graph, id, item)
}

fn guard_unless_guarded<T: GuardTag>(
    graph: &mut BRepGraph,
    id: TypedId<T>,
    item: ItemId,
) -> Result<MutGuard<'_, T>, Failure> {
    if graph.inc_storage().is_item_guarded(item) {
        return Err(program_error(
            "BRepGraph::EditorView::Mut*(): guard already active",
            item,
        ));
    }
    MutGuard::new(graph, id)
}
