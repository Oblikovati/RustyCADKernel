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
// Rust translation of BRepGraph_LayerHistory::rebuildCaches of
// ModelingData/TKBRep/BRepGraph/BRepGraph_LayerHistory.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Rebuilding the lookup maps of the history layer from its records.
use super::record::{HistoryKey, append_forward};
use super::{HistoryEvent, HistoryKind, HistoryState, KeyedMaps, LayerHistory};
use rustycad_tkernel::ncollection::DataMap;

impl LayerHistory {
    /// `rebuildCaches()` (private): clears every lookup map and replays the records in
    /// order, then bumps the revision. A copy that replaced the records calls it.
    pub(super) fn rebuild_caches(&self) {
        let mut state = self.state.write();
        state.nodes.clear();
        state.uids.clear();
        state.item_uids.clear();
        let records = std::mem::take(&mut state.records);
        for record in &records {
            replay_record(&mut state, record);
        }
        state.records = records;
        drop(state);
        self.base.touch();
    }
}

// One record: its node mapping, then its UID and item-UID mappings, in map order.
fn replay_record(state: &mut HistoryState, record: &HistoryEvent) {
    let kind = record.record_kind;
    for (original, images) in record.mapping.items() {
        if kind == HistoryKind::Deleted || images.is_empty() {
            state.nodes.deleted.add(*original);
            continue;
        }
        for derived in images.iter().filter(|derived| *derived != original) {
            state.nodes.link(kind, *original, *derived);
        }
        if kind == HistoryKind::Replaced {
            state.nodes.deleted.add(*original);
        }
    }
    replay_keyed(&mut state.uids, kind, &record.uid_mapping);
    replay_keyed(&mut state.item_uids, kind, &record.item_uid_mapping);
}

// The UID or item-UID mapping of a record: every original is a known input.
fn replay_keyed<K: HistoryKey>(
    maps: &mut KeyedMaps<K>,
    kind: HistoryKind,
    mapping: &DataMap<K, Vec<K>>,
) {
    for (original, images) in mapping.items() {
        maps.known_inputs.add(*original);
        if kind == HistoryKind::Deleted || images.is_empty() {
            maps.deleted.add(*original);
            continue;
        }
        for derived in images.iter().filter(|derived| *derived != original) {
            append_forward(maps.forward_mut(kind), *original, *derived);
        }
        if kind == HistoryKind::Replaced {
            maps.deleted.add(*original);
        }
    }
}
