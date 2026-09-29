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
// Rust translation of BRepGraph_LayerHistory::CopyTo (the compaction migration) of
// ModelingData/TKBRep/BRepGraph/BRepGraph_LayerHistory.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Copying the history into another graph, or migrating it through a compaction.
use super::record::{append_unique, new_event};
use super::{HistoryEvent, LayerHistory};
use crate::brep_graph::copy_remap::CopyRemap;
use crate::brep_graph::{ItemId, NodeId};
use rustycad_tkernel::standard::Failure;

mod replay;

// `THE_COMPACT_REMAP_LABEL`: the records a compaction itself appends.
const COMPACT_REMAP_LABEL: &str = "Compact:Remap";

impl LayerHistory {
    /// `CopyTo(theCopy)`: nothing for an empty history. A compaction rebuilds the
    /// target's records (durable UID mappings kept, node mappings remapped through the
    /// item map, stale entries and earlier `Compact:Remap` records dropped); a copy
    /// replays every record into the target's history. The target keeps this layer's
    /// enabled flag.
    pub(super) fn copy_history_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        let (records, enabled) = {
            let state = self.state.read();
            (state.records.clone(), state.enabled)
        };
        if records.is_empty() {
            return Ok(());
        }
        let target = copy
            .target_graph_const()
            .layer_registry()
            .ensure::<LayerHistory>()?;
        let Some(target) = target.get() else {
            return Ok(());
        };
        target.set_enabled(true);
        if copy.is_compact() {
            migrate_records(copy, target, &records);
        } else {
            replay::replay_records(copy, target, &records);
        }
        target.set_enabled(enabled);
        Ok(())
    }
}

// The compaction branch: the target's records are replaced by the remapped ones, then
// its lookup maps are rebuilt.
fn migrate_records(copy: &CopyRemap<'_>, target: &LayerHistory, records: &[HistoryEvent]) {
    let mut migrated: Vec<HistoryEvent> = Vec::with_capacity(32);
    for record in records {
        if record.operation_name == COMPACT_REMAP_LABEL {
            continue;
        }
        let event = migrate_record(copy, record, migrated.len());
        let empty = event.mapping.is_empty()
            && event.uid_mapping.is_empty()
            && event.item_uid_mapping.is_empty();
        if !empty {
            migrated.push(event);
        }
    }
    target.state.write().records = migrated;
    target.rebuild_caches();
}

// One migrated record: the durable mappings as they are, the node mapping remapped.
fn migrate_record(copy: &CopyRemap<'_>, record: &HistoryEvent, sequence: usize) -> HistoryEvent {
    let mut event = new_event(&record.operation_name, sequence, record.record_kind);
    event.extra_info = record.extra_info.clone();
    for (original, images) in record.uid_mapping.items() {
        event.uid_mapping.bind(*original, images.clone());
    }
    for (original, images) in record.item_uid_mapping.items() {
        event.item_uid_mapping.bind(*original, images.clone());
    }
    for (original, images) in record.mapping.items() {
        let target_original = copy.target_item(ItemId::from(*original));
        if !target_original.is_node() {
            continue;
        }
        event
            .mapping
            .bind(target_original.node_id(), remap_nodes(copy, images));
    }
    event
}

/// The distinct target nodes of the source nodes that map to nodes.
pub(super) fn remap_nodes(copy: &CopyRemap<'_>, nodes: &[NodeId]) -> Vec<NodeId> {
    let mut images = Vec::new();
    for node in nodes {
        let target = copy.target_item(ItemId::from(*node));
        if target.is_node() {
            append_unique(&mut images, target.node_id());
        }
    }
    images
}
