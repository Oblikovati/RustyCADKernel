// Rust translation of the registration members of BRepGraph_LayerDeferred
// (RegisterDeferred*, UnregisterDeferred, ReserveDeferredItems, Begin/End bulk
// registration, lockItem, unlockItem) of ModelingData/TKBRep/BRepGraph/BRepGraph_LayerDeferred.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Registering and unregistering deferred representations.
use super::{DeferredEntry, DeferredRepresentation, LayerDeferred, RepresentationKind};
use crate::brep_graph::layer::{AttachedGraph, LayerType};
use crate::brep_graph::layer_lock::LayerLock;
use crate::brep_graph::{BRepGraph, ItemId, NodeId, RefId};
use rustycad_tkernel::standard::{Failure, Handle};
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

    /// `UnregisterDeferred(theItem)`: drops the item's entry and unlocks it in the
    /// attached graph's lock layer.
    pub fn unregister_deferred(&self, graph: &BRepGraph, item: ItemId) {
        if !item.is_valid() {
            return;
        }
        self.remove_item(self.base.attached_graph(graph), item);
    }

    /// `UnregisterDeferred(theNode)`.
    pub fn unregister_deferred_node(&self, graph: &BRepGraph, node: NodeId) {
        self.unregister_deferred(graph, node.into());
    }

    /// `UnregisterDeferred(theRef)`.
    pub fn unregister_deferred_ref(&self, graph: &BRepGraph, reference: RefId) {
        self.unregister_deferred(graph, reference.into());
    }

    /// `ReserveDeferredItems(theNbItems)`: grows the entry map, then reserves the owners
    /// of the attached graph's lock layer (`Ensure`d).
    pub fn reserve_deferred_items(&self, graph: &BRepGraph, count: usize) -> Result<(), Failure> {
        {
            let mut records = self.records.write();
            let extent = records.entries.extent();
            records.entries.resize(extent + count);
        }
        let Some(graph) = self.base.attached_graph(graph) else {
            return Ok(());
        };
        let lock = graph.layer_registry().ensure::<LayerLock>()?;
        if let Some(lock) = lock.get() {
            lock.reserve_owners(count);
        }
        Ok(())
    }

    /// `EndBulkRegistration()`: closes one bulk level; the outermost publishes one
    /// revision update when something changed, and `TouchOwners` on the attached
    /// graph's lock layer when there is one. Standard_ProgramError without a matching
    /// begin.
    pub fn end_bulk_registration(&self, graph: &BRepGraph) -> Result<(), Failure> {
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
        if let Some(lock) = find_lock_layer(self.base.attached_graph(graph)) {
            lock.touch_owners();
        }
        Ok(())
    }

    /// `lockItem(theItem)` (private): the attached graph's lock layer (`Ensure`d) owns
    /// the item under this layer's GUID, publishing the change at once outside bulk
    /// mode; a bulk change is remembered. A detached layer locks nothing.
    fn lock_item(&self, attached: AttachedGraph<'_>, item: ItemId) -> Result<(), Failure> {
        let Some(graph) = attached else {
            return Ok(());
        };
        let lock = graph.layer_registry().ensure::<LayerLock>()?;
        let Some(lock) = lock.get() else {
            return Ok(());
        };
        let bulk = self.records.read().bulk_depth != 0;
        let changed = lock.set_owner_with_revision(graph, item, &LayerDeferred::get_id(), !bulk)?;
        if bulk && changed {
            self.records.write().has_bulk_changes = true;
        }
        Ok(())
    }
}

/// `FindLayer<BRepGraph_LayerLock>()` of the attached graph: `None` when detached or
/// when no lock layer is registered.
pub(super) fn find_lock_layer(attached: AttachedGraph<'_>) -> Option<Handle<LayerLock>> {
    let lock = attached?.layer_registry().find::<LayerLock>();
    lock.is_some().then_some(lock)
}
