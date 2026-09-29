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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_MutGuard.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_MutGuard<T>`: the scope token that batches the modification notice of
//! one definition or reference entry (docs/topology-design.md section 11.4).
use super::{
    BRepGraph, ChildRefTag, CoEdgeTag, CompSolidTag, CompoundTag, EdgeTag, FaceRefTag, FaceTag,
    OccurrenceRefTag, OccurrenceTag, ProductTag, ShellRefTag, ShellTag, SolidRefTag, SolidTag,
    TypedId, VertexRefTag, VertexTag, WireRefTag, WireTag,
};
use crate::brep_graph_inc::{NodeTag, OwnershipPlanes, RefTag, Storage, StoredTag};
use rustycad_tkernel::standard::Failure;
use std::ops::Deref;

/// The kinds a guard can hold: the definition kinds (`BaseDef`, notified with
/// `markModified`) and the reference kinds (`BaseRef`, notified with
/// `markRefModified`), the compile-time dispatch of the native template.
pub trait GuardTag: StoredTag<Store: OwnershipPlanes> {
    /// The guarded record (`VertexDef`, `ShellRef`...).
    type Target;
    /// The record of `id` (panics out of range, undefined natively).
    fn target(storage: &Storage, id: TypedId<Self>) -> &Self::Target;
    /// The mutable record of `id`.
    fn target_mut(storage: &mut Storage, id: TypedId<Self>) -> &mut Self::Target;
    /// `notify()`: `markModified` for a definition, `markRefModified` for a reference.
    fn notify(graph: &mut BRepGraph, id: TypedId<Self>);
}

macro_rules! guard_tags {
    ($family:ident, $record:ident, $getter:ident, $changer:ident, $mark:ident;
     $($tag:ty),* $(,)?) => {$(
        impl GuardTag for $tag {
            type Target = <$tag as $family>::$record;
            fn target(storage: &Storage, id: TypedId<Self>) -> &Self::Target {
                storage.$getter(id)
            }
            fn target_mut(storage: &mut Storage, id: TypedId<Self>) -> &mut Self::Target {
                storage.$changer(id)
            }
            fn notify(graph: &mut BRepGraph, id: TypedId<Self>) {
                graph.$mark(id.into());
            }
        }
    )*};
}

guard_tags! {
    NodeTag, Def, def, change_def, mark_modified;
    VertexTag, EdgeTag, CoEdgeTag, WireTag, FaceTag, ShellTag, SolidTag, CompoundTag,
    CompSolidTag, ProductTag, OccurrenceTag,
}

guard_tags! {
    RefTag, Entry, entry, change_entry, mark_ref_modified;
    VertexRefTag, WireRefTag, FaceRefTag, ShellRefTag, SolidRefTag, ChildRefTag,
    OccurrenceRefTag,
}

/// `BRepGraph_MutGuard<T>`: registered on its item (the guard plane) for its whole
/// life; the typed setters of the editor write through [`internal`](Self::internal),
/// which flags it dirty, and dropping a dirty guard fires one `markModified` or
/// `markRefModified`, after the guard plane is cleared.
///
/// The guard holds the graph's only mutable borrow, so no second guard can be taken
/// on any item from outside it; [`graph_mut`](Self::graph_mut) reaches the editor
/// from inside, where a second guard on the same item raises Standard_ProgramError
/// as natively. The record is read through the storage by id, never through a held
/// reference, so no `unsafe` is needed.
///
/// ```
/// use rustycad_tkbrep::brep_graph::{BRepGraph, editor::EdgeOps};
/// use rustycad_tkmath::gp::Point3;
/// let mut graph = BRepGraph::new();
/// let vertex = graph.editor().vertices().add(Point3::new(0.0, 0.0, 0.0), 1.0e-7);
/// let mut guard = graph.editor().vertices().mutate(vertex).unwrap();
/// assert!(!guard.is_dirty());
/// guard.internal().tolerance = 0.5;
/// assert!(guard.is_dirty() && guard.tolerance == 0.5);
/// drop(guard);
/// assert_eq!(graph.topo().vertices().definition(vertex).base.own_gen, 2);
/// ```
#[derive(Debug)]
pub struct MutGuard<'g, T: GuardTag> {
    graph: &'g mut BRepGraph,
    id: TypedId<T>,
    dirty: bool,
}

impl<'g, T: GuardTag> MutGuard<'g, T> {
    /// `BRepGraph_MutGuard(theGraph, theStorage, theEntity, theId)`: registers the
    /// guard on the item; an item already guarded raises Standard_ProgramError.
    pub fn new(graph: &'g mut BRepGraph, id: TypedId<T>) -> Result<Self, Failure> {
        if graph.inc_storage().is_guarded(id) {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                &format!(
                    "BRepGraph_MutGuard: guard already active on this item ({} {})",
                    T::NAME,
                    id.index
                ),
                "",
            ));
        }
        graph.inc_storage_mut().set_guarded(id);
        Ok(Self {
            graph,
            id,
            dirty: false,
        })
    }

    /// `operator bool()`: the guard owns its item. A Rust guard cannot be moved from,
    /// so a live guard always does.
    pub fn is_active(&self) -> bool {
        true
    }

    /// `Id()`: the guarded item.
    pub fn id(&self) -> TypedId<T> {
        self.id
    }

    /// `Graph()`: the owning graph, for reads.
    pub fn graph(&self) -> &BRepGraph {
        self.graph
    }

    /// `Graph()`: the owning graph, for the editor calls made inside the scope.
    pub fn graph_mut(&mut self) -> &mut BRepGraph {
        self.graph
    }

    /// `MarkDirty()`: flags a modification the guard did not see (an in-place change of
    /// a shared geometry handle).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// `IsDirty()`: `internal` or `mark_dirty` flagged the item.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// `Internal()`: the mutable record, flagging the guard dirty. External code goes
    /// through the editor's typed setters.
    pub fn internal(&mut self) -> &mut T::Target {
        self.dirty = true;
        T::target_mut(self.graph.inc_storage_mut(), self.id)
    }
}

impl<T: GuardTag> Deref for MutGuard<'_, T> {
    type Target = T::Target;

    /// `operator->()` / `operator*()`: read-only access to the record.
    fn deref(&self) -> &T::Target {
        T::target(self.graph.inc_storage(), self.id)
    }
}

impl<T: GuardTag> Drop for MutGuard<'_, T> {
    /// `~BRepGraph_MutGuard()`: clears the guard plane, then notifies once if dirty,
    /// so the propagation does not see its own registration.
    fn drop(&mut self) {
        self.graph.inc_storage_mut().clear_guarded(self.id);
        if self.dirty {
            T::notify(self.graph, self.id);
        }
    }
}
