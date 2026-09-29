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
// Rust translation of EditorView::OccurrenceOps of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView_Mut.cxx and
// BRepGraph_EditorView_Setters.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `EditorView::OccurrenceOps`: the occurrence and occurrence reference setters.
use super::guards::{mutate_node, mutate_ref};
use super::helpers::{is_valid_occurrence_child_kind, require_settable};
use crate::brep_graph::mut_guard::MutGuard;
use crate::brep_graph::*;
use crate::brep_graph_inc::Storage;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::toploc::Location;

/// `EditorView::OccurrenceOps`.
#[derive(Debug)]
pub struct OccurrenceOps<'g> {
    pub(super) graph: &'g mut BRepGraph,
}

impl<'g> OccurrenceOps<'g> {
    /// `Mut(theOccurrence)`.
    pub fn mutate(self, occurrence: OccurrenceId) -> Result<MutGuard<'g, OccurrenceTag>, Failure> {
        mutate_node(self.graph, occurrence)
    }

    /// `MutRef(theOccurrenceRef)`.
    pub fn mutate_ref(
        self,
        reference: OccurrenceRefId,
    ) -> Result<MutGuard<'g, OccurrenceRefTag>, Failure> {
        mutate_ref(self.graph, reference)
    }

    /// `SetRefLocalLocation(theOccurrenceRef, theLoc)`.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::{BRepGraph, OccurrenceRefId};
    /// use rustycad_tkmath::toploc::Location;
    /// let mut graph = BRepGraph::new();
    /// let part = graph.editor().products().add_empty();
    /// let assembly = graph.editor().products().add_empty();
    /// let (_, reference) = graph.editor().products()
    ///     .append(assembly, part, &Location::default(), Default::default()).unwrap();
    /// graph.editor().occurrences().set_ref_local_location(reference, &Location::default()).unwrap();
    /// ```
    pub fn set_ref_local_location(
        &mut self,
        reference: OccurrenceRefId,
        location: &Location,
    ) -> Result<(), Failure> {
        require_settable(self.graph, reference)?;
        self.graph
            .inc_storage_mut()
            .change_occurrence_ref(reference)
            .local_location = location.clone();
        self.graph.mark_ref_modified(reference.into());
        Ok(())
    }

    /// `SetRefLocalLocation(theMut, theLoc)`.
    pub fn set_ref_local_location_guarded(
        guard: &mut MutGuard<'_, OccurrenceRefTag>,
        location: &Location,
    ) {
        guard.internal().local_location = location.clone();
    }

    /// `SetChildNodeId(theOccurrence, theChildNodeId)`: an inactive occurrence, or a
    /// child that is not an active topology node or product, is ignored; a locked
    /// occurrence raises Standard_ProgramError.
    pub fn set_child_node_id(
        &mut self,
        occurrence: OccurrenceId,
        child: NodeId,
    ) -> Result<(), Failure> {
        let storage = self.graph.inc_storage();
        let active =
            occurrence.is_valid_for(storage.nb_occurrences()) && !storage.is_removed(occurrence);
        if !active || !is_active_occurrence_child(self.graph, child) {
            return Ok(());
        }
        require_settable(self.graph, occurrence)?;
        let storage = self.graph.inc_storage_mut();
        let old = storage.occurrence(occurrence).child_node_id;
        if old == child {
            return Ok(());
        }
        storage.change_occurrence(occurrence).child_node_id = child;
        storage.rebind_occurrence_child(occurrence, old, child);
        self.graph.mark_modified(occurrence.into());
        Ok(())
    }

    /// `SetChildNodeId(theMut, theChildNodeId)`.
    pub fn set_child_node_id_guarded(guard: &mut MutGuard<'_, OccurrenceTag>, child: NodeId) {
        if !is_active_occurrence_child(guard.graph(), child) {
            return;
        }
        let old = guard.child_node_id;
        if old == child {
            return;
        }
        guard.internal().child_node_id = child;
        let occurrence = guard.id();
        guard
            .graph_mut()
            .inc_storage_mut()
            .rebind_occurrence_child(occurrence, old, child);
    }

    /// `SetRefChildOccurrenceId(theOccurrenceRef, theOccurrence)`: retargets the
    /// reference and rebinds the product relations; an occurrence another active
    /// reference places is refused (the release behaviour of the native assertion).
    pub fn set_ref_child_occurrence_id(
        &mut self,
        reference: OccurrenceRefId,
        occurrence: OccurrenceId,
    ) -> Result<(), Failure> {
        require_settable(self.graph, reference)?;
        let storage = self.graph.inc_storage_mut();
        let old = storage.occurrence_ref(reference).child_occurrence_id;
        if old == occurrence || occurrence_has_other_active_ref(storage, occurrence, reference) {
            return Ok(());
        }
        storage.change_occurrence_ref(reference).child_occurrence_id = occurrence;
        storage.rebind_occurrence_ref(reference, old, occurrence);
        self.graph.mark_ref_modified(reference.into());
        Ok(())
    }

    /// `SetRefChildOccurrenceId(theMut, theOccurrence)`.
    pub fn set_ref_child_occurrence_id_guarded(
        guard: &mut MutGuard<'_, OccurrenceRefTag>,
        occurrence: OccurrenceId,
    ) {
        let old = guard.child_occurrence_id;
        let reference = guard.id();
        if old == occurrence
            || occurrence_has_other_active_ref(guard.graph().inc_storage(), occurrence, reference)
        {
            return;
        }
        guard.internal().child_occurrence_id = occurrence;
        guard
            .graph_mut()
            .inc_storage_mut()
            .rebind_occurrence_ref(reference, old, occurrence);
    }
}

/// `isActiveOccurrenceChild(theStorage, theChild)`: an active node of a topology kind
/// or an active product (never an occurrence).
fn is_active_occurrence_child(graph: &BRepGraph, child: NodeId) -> bool {
    child.is_valid()
        && is_valid_occurrence_child_kind(child.kind)
        && graph.topo().generic().is_active(child)
}

/// `occurrenceHasOtherActiveRef(theStorage, theOccurrence, theExcludingRef)`: another
/// active reference places the occurrence.
fn occurrence_has_other_active_ref(
    storage: &Storage,
    occurrence: OccurrenceId,
    excluding: OccurrenceRefId,
) -> bool {
    if !occurrence.is_valid_for(storage.nb_occurrences()) {
        return false;
    }
    (0..storage.nb_occurrence_refs())
        .map(OccurrenceRefId::new)
        .any(|reference| {
            reference != excluding
                && !storage.is_removed(reference)
                && storage.occurrence_ref(reference).child_occurrence_id == occurrence
        })
}
