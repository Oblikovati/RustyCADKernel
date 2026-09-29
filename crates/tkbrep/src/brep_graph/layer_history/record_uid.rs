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
// Rust translation of BRepGraph_LayerHistory::RecordUid, RecordDeletedUid,
// RecordItemUid and RecordDeletedItemUid
// (ModelingData/TKBRep/BRepGraph/BRepGraph_LayerHistory.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The durable (UID and item-UID keyed) recording members, for histories whose
//! inputs and results may live in different graphs.
use super::record::{append_forward, append_unique, new_event};
use super::{HistoryKind, LayerHistory};
use crate::brep_graph::{ItemUid, Uid};
use rustycad_tkernel::tcollection::AsciiString;

/// `BRepGraph_ItemUID::Node(theUid.Kind, theUid.Counter)`.
pub(super) fn node_item_uid(uid: Uid) -> ItemUid {
    ItemUid::node(uid.kind, uid.counter as usize)
}

// The valid, distinct replacements and their item UIDs (`aValidReplacements`,
// `aValidItemReplacements`).
fn valid_uid_images(replacements: &[Uid]) -> (Vec<Uid>, Vec<ItemUid>) {
    let (mut uids, mut item_uids) = (Vec::new(), Vec::new());
    for &derived in replacements.iter().filter(|uid| uid.is_valid()) {
        append_unique(&mut uids, derived);
        append_unique(&mut item_uids, node_item_uid(derived));
    }
    (uids, item_uids)
}

impl LayerHistory {
    /// `RecordUid(theOpLabel, theOriginal, theReplacements, theKind)`: a UID-keyed
    /// event (and its item-UID mirror); the original becomes a known input even when no
    /// valid replacement is left, and empty replacements are a `RecordDeletedUid`.
    pub fn record_uid(
        &self,
        label: &AsciiString,
        original: Uid,
        replacements: &[Uid],
        kind: HistoryKind,
    ) {
        if !self.is_enabled() || !original.is_valid() {
            return;
        }
        let original_item = node_item_uid(original);
        {
            let mut state = self.state.write();
            state.uids.known_inputs.add(original);
            state.item_uids.known_inputs.add(original_item);
        }
        if replacements.is_empty() {
            self.record_deleted_uid(label, &[original]);
            return;
        }
        let (images, item_images) = valid_uid_images(replacements);
        if images.is_empty() {
            return;
        }
        let mut state = self.state.write();
        let mut event = new_event(label, state.records.len(), kind);
        event.uid_mapping.bind(original, images.clone());
        event
            .item_uid_mapping
            .bind(original_item, item_images.clone());
        state.records.push(event);
        for derived in images {
            append_forward(state.uids.forward_mut(kind), original, derived);
        }
        for derived in item_images {
            append_forward(state.item_uids.forward_mut(kind), original_item, derived);
        }
        if kind == HistoryKind::Replaced {
            state.uids.deleted.add(original);
            state.item_uids.deleted.add(original_item);
        }
        drop(state);
        self.base.touch();
    }

    /// `RecordDeletedUid(theOpLabel, theDeleted)`: one Deleted event over the valid UIDs
    /// (none recorded when every UID is invalid).
    pub fn record_deleted_uid(&self, label: &AsciiString, deleted: &[Uid]) {
        if !self.is_enabled() || deleted.is_empty() {
            return;
        }
        let mut state = self.state.write();
        let mut event = new_event(label, state.records.len(), HistoryKind::Deleted);
        event.uid_mapping.resize(deleted.len());
        event.item_uid_mapping.resize(deleted.len());
        for &uid in deleted.iter().filter(|uid| uid.is_valid()) {
            let item = node_item_uid(uid);
            state.uids.known_inputs.add(uid);
            state.uids.deleted.add(uid);
            state.item_uids.known_inputs.add(item);
            state.item_uids.deleted.add(item);
            if !event.uid_mapping.is_bound(&uid) {
                event.uid_mapping.bind(uid, Vec::new());
            }
            if !event.item_uid_mapping.is_bound(&item) {
                event.item_uid_mapping.bind(item, Vec::new());
            }
        }
        if event.uid_mapping.is_empty() && event.item_uid_mapping.is_empty() {
            return;
        }
        state.records.push(event);
        drop(state);
        self.base.touch();
    }

    /// `RecordItemUid(theOpLabel, theOriginal, theReplacements, theKind)`: an
    /// item-UID-keyed event over the valid, distinct replacements.
    pub fn record_item_uid(
        &self,
        label: &AsciiString,
        original: ItemUid,
        replacements: &[ItemUid],
        kind: HistoryKind,
    ) {
        if !self.is_enabled() || !original.is_valid() {
            return;
        }
        self.state.write().item_uids.known_inputs.add(original);
        if replacements.is_empty() {
            self.record_deleted_item_uid(label, &[original]);
            return;
        }
        let mut images = Vec::new();
        for &derived in replacements.iter().filter(|uid| uid.is_valid()) {
            append_unique(&mut images, derived);
        }
        if images.is_empty() {
            return;
        }
        let mut state = self.state.write();
        let mut event = new_event(label, state.records.len(), kind);
        event.item_uid_mapping.bind(original, images.clone());
        state.records.push(event);
        for derived in images {
            append_forward(state.item_uids.forward_mut(kind), original, derived);
        }
        if kind == HistoryKind::Replaced {
            state.item_uids.deleted.add(original);
        }
        drop(state);
        self.base.touch();
    }

    /// `RecordDeletedItemUid(theOpLabel, theDeleted)`: one Deleted event over the valid
    /// item UIDs.
    pub fn record_deleted_item_uid(&self, label: &AsciiString, deleted: &[ItemUid]) {
        if !self.is_enabled() || deleted.is_empty() {
            return;
        }
        let mut state = self.state.write();
        let mut event = new_event(label, state.records.len(), HistoryKind::Deleted);
        event.item_uid_mapping.resize(deleted.len());
        for &uid in deleted.iter().filter(|uid| uid.is_valid()) {
            state.item_uids.known_inputs.add(uid);
            state.item_uids.deleted.add(uid);
            if !event.item_uid_mapping.is_bound(&uid) {
                event.item_uid_mapping.bind(uid, Vec::new());
            }
        }
        if event.item_uid_mapping.is_empty() {
            return;
        }
        state.records.push(event);
        drop(state);
        self.base.touch();
    }
}
