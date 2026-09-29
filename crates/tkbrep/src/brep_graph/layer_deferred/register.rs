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
// Rust translation of the registration members of BRepGraph_LayerDeferred
// (RegisterDeferred*, UnregisterDeferred, ReserveDeferredItems, Begin/End bulk
// registration, lockItem) of ModelingData/TKBRep/BRepGraph/BRepGraph_LayerDeferred.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Registering and unregistering deferred representations.
use super::{DeferredEntry, DeferredRepresentation, LayerDeferred, RepresentationKind};
use crate::brep_graph::layer::AttachedGraph;
use crate::brep_graph::{BRepGraph, ItemId, NodeId, RefId};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;

// The identity of an item's deferred source: the provider and its key.
#[derive(Clone, Copy, Debug)]
struct DeferredSource<'s> {
    // `theProvider`.
    provider: &'s AsciiString,
    // `theSourceKey`.
    source_key: &'s AsciiString,
}

impl LayerDeferred {
    /// `RegisterDeferred(theItem, theProvider, theSourceKey, theRepresentationKind,
    /// theRepresentationName, theSourceIndex)`: one representation (role 0).
    #[allow(clippy::too_many_arguments)]
    pub fn register_deferred(
        &self,
        graph: &BRepGraph,
        item: ItemId,
        provider: &AsciiString,
        source_key: &AsciiString,
        kind: RepresentationKind,
        name: &AsciiString,
        source_index: u32,
    ) -> Result<(), Failure> {
        let representation = DeferredRepresentation {
            kind,
            name: name.clone(),
            source_index,
            ..DeferredRepresentation::default()
        };
        self.register_deferred_representations(graph, item, provider, source_key, &[representation])
    }

    /// `RegisterDeferred(theNode, ...)`.
    #[allow(clippy::too_many_arguments)]
    pub fn register_deferred_node(
        &self,
        graph: &BRepGraph,
        node: NodeId,
        provider: &AsciiString,
        source_key: &AsciiString,
        kind: RepresentationKind,
        name: &AsciiString,
        source_index: u32,
    ) -> Result<(), Failure> {
        self.register_deferred(
            graph,
            node.into(),
            provider,
            source_key,
            kind,
            name,
            source_index,
        )
    }

    /// `RegisterDeferred(theRef, ...)`.
    #[allow(clippy::too_many_arguments)]
    pub fn register_deferred_ref(
        &self,
        graph: &BRepGraph,
        reference: RefId,
        provider: &AsciiString,
        source_key: &AsciiString,
        kind: RepresentationKind,
        name: &AsciiString,
        source_index: u32,
    ) -> Result<(), Failure> {
        self.register_deferred(
            graph,
            reference.into(),
            provider,
            source_key,
            kind,
            name,
            source_index,
        )
    }

    /// `RegisterDeferredRepresentations(theItem, theProvider, theSourceKey,
    /// theRepresentations, theNbRepresentations)`: sets the provider and key and appends
    /// the representations not stored yet; a change locks the item and bumps the
    /// revision (postponed in bulk mode). Standard_ProgramError past eight
    /// representations (the entry keeps what was appended before).
    pub fn register_deferred_representations(
        &self,
        graph: &BRepGraph,
        item: ItemId,
        provider: &AsciiString,
        source_key: &AsciiString,
        representations: &[DeferredRepresentation],
    ) -> Result<(), Failure> {
        if !item.is_valid() || representations.is_empty() {
            return Ok(());
        }
        let source = DeferredSource {
            provider,
            source_key,
        };
        if !self.merge_entry(item, source, representations)? {
            return Ok(());
        }
        self.lock_item(self.base.attached_graph(graph), item)?;
        self.publish_change();
        Ok(())
    }

    // The entry update of `RegisterDeferredRepresentations` (`TryEmplaced`): whether
    // anything changed.
    fn merge_entry(
        &self,
        item: ItemId,
        source: DeferredSource<'_>,
        representations: &[DeferredRepresentation],
    ) -> Result<bool, Failure> {
        let mut records = self.records.write();
        let entry = records.entries.try_emplaced(item, DeferredEntry::default);
        let mut changed = false;
        if entry.provider != *source.provider {
            entry.provider = source.provider.clone();
            changed = true;
        }
        if entry.source_key != *source.source_key {
            entry.source_key = source.source_key.clone();
            changed = true;
        }
        for representation in representations {
            if entry.representations.as_slice().contains(representation) {
                continue;
            }
            entry.representations.append(representation)?;
            changed = true;
        }
        Ok(changed)
    }

    /// `RegisterDeferredRepresentationsDirect(...)`: the trusted bulk-load path for an
    /// item without an entry (an item that has one is left unchanged: the release-mode
    /// `Standard_ASSERT_RETURN`); no duplicate check.
    pub fn register_deferred_representations_direct(
        &self,
        graph: &BRepGraph,
        item: ItemId,
        provider: &AsciiString,
        source_key: &AsciiString,
        representations: &[DeferredRepresentation],
    ) -> Result<(), Failure> {
        if !item.is_valid() || representations.is_empty() || self.has_deferred(item) {
            return Ok(());
        }
        let mut entry = DeferredEntry {
            provider: provider.clone(),
            source_key: source_key.clone(),
            ..DeferredEntry::default()
        };
        for representation in representations {
            entry.representations.append(representation)?;
        }
        self.records.write().entries.bind(item, entry);
        self.lock_item(self.base.attached_graph(graph), item)?;
        self.publish_change();
        Ok(())
    }

    // The tail of the registrations: a bulk change is remembered, else published.
    fn publish_change(&self) {
        let mut records = self.records.write();
        if records.bulk_depth != 0 {
            records.has_bulk_changes = true;
            return;
        }
        drop(records);
        self.base.touch();
    }

    /// `UnregisterDeferred(theItem)`: drops the item's entry and unlocks it.
    pub fn unregister_deferred(&self, item: ItemId) {
        if !item.is_valid() {
            return;
        }
        self.remove_item(item);
    }

    /// `UnregisterDeferred(theNode)`.
    pub fn unregister_deferred_node(&self, node: NodeId) {
        self.unregister_deferred(node.into());
    }

    /// `UnregisterDeferred(theRef)`.
    pub fn unregister_deferred_ref(&self, reference: RefId) {
        self.unregister_deferred(reference.into());
    }

    /// `ReserveDeferredItems(theNbItems)`: grows the entry map, then reserves the lock
    /// layer's owners in the attached graph.
    pub fn reserve_deferred_items(&self, graph: &BRepGraph, count: usize) -> Result<(), Failure> {
        {
            let mut records = self.records.write();
            let extent = records.entries.extent();
            records.entries.resize(extent + count);
        }
        match self.base.attached_graph(graph) {
            Some(_) => Err(lock_layer_missing("ReserveDeferredItems")),
            None => Ok(()),
        }
    }

    /// `EndBulkRegistration()`: closes one bulk level; the outermost publishes one
    /// revision update when something changed (the lock layer's `TouchOwners` finds no
    /// lock layer yet). Standard_ProgramError without a matching begin.
    pub fn end_bulk_registration(&self) -> Result<(), Failure> {
        let mut records = self.records.write();
        if records.bulk_depth == 0 {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "*** ERROR: ASSERT: BRepGraph_LayerDeferred::EndBulkRegistration without matching \
                 begin (bulk depth 0, expected at least 1)",
                "",
            ));
        }
        records.bulk_depth -= 1;
        if records.bulk_depth != 0 || !records.has_bulk_changes {
            return Ok(());
        }
        records.has_bulk_changes = false;
        drop(records);
        self.base.touch();
        Ok(())
    }

    /// `lockItem(theItem)` (private): an attached layer locks the item through the
    /// graph's `BRepGraph_LayerLock` (`Ensure`d), which is not ported yet: that path
    /// fails with Standard_NotImplemented (docs/topology-design.md 11.9). A detached
    /// layer locks nothing, as natively.
    fn lock_item(&self, attached: AttachedGraph<'_>, _item: ItemId) -> Result<(), Failure> {
        match attached {
            Some(_) => Err(lock_layer_missing("RegisterDeferred")),
            None => Ok(()),
        }
    }
}

fn lock_layer_missing(member: &str) -> Failure {
    Failure::with_type(
        "Standard_NotImplemented",
        &format!(
            "BRepGraph_LayerDeferred::{member} on an attached layer needs BRepGraph_LayerLock, \
             which is not ported yet (expected a detached layer)"
        ),
        "",
    )
}
