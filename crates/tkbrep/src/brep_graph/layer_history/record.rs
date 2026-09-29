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
// Rust translation of BRepGraph_LayerHistory::Record, RecordBatch, RecordDeleted,
// RecordReplaced and RecordReplacedBatch and of the map helpers of
// ModelingData/TKBRep/BRepGraph/BRepGraph_LayerHistory.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The node-keyed recording members: every event also records the item UIDs of its
//! nodes when the layer is attached to the graph.
use super::{HistoryEvent, HistoryKind, HistoryState, KeyedMaps, LayerHistory, NodeMaps};
use crate::brep_graph::layer::AttachedGraph;
use crate::brep_graph::{BRepGraph, ItemId, ItemUid, NodeId};
use rustycad_tkernel::ncollection::{DataMap, HashKey};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use std::fmt::Debug;

/// A key of the history maps (a node id, a UID or an item UID).
pub(super) trait HistoryKey: Copy + Debug + PartialEq + HashKey {}
impl<K: Copy + Debug + PartialEq + HashKey> HistoryKey for K {}

/// `appendUniqueNode`/`appendUniqueUid`/`appendUniqueItemUid`: append when absent.
pub(super) fn append_unique<K: PartialEq + Copy>(values: &mut Vec<K>, value: K) {
    if !values.contains(&value) {
        values.push(value);
    }
}

/// `appendItemUidForward` (and the inline forward updates of the node and UID maps):
/// append `derived` to the images of `original`, binding a new list when absent.
pub(super) fn append_forward<K: HistoryKey>(map: &mut DataMap<K, Vec<K>>, original: K, derived: K) {
    if let Some(images) = map.change_seek(&original) {
        append_unique(images, derived);
        return;
    }
    map.bind(original, vec![derived]);
}

/// `appendDerivedOrigin(theMap, theDerived, theOriginal)`: the reverse map update.
fn append_derived_origin(
    map: &mut DataMap<NodeId, Vec<NodeId>>,
    derived: NodeId,
    original: NodeId,
) {
    append_forward(map, derived, original);
}

/// `itemUidOf(theGraph, ItemId(theNode))`: the durable UID of a node in the attached
/// graph, invalid when detached.
pub(super) fn item_uid_of(attached: AttachedGraph<'_>, node: NodeId) -> ItemUid {
    attached.map_or_else(ItemUid::invalid, |graph| {
        graph.uids().of_item(ItemId::from(node))
    })
}

/// `itemUidsOfNodes(theGraph, theNodes)`: the valid, distinct UIDs of the nodes.
fn item_uids_of_nodes(attached: AttachedGraph<'_>, nodes: &[NodeId]) -> Vec<ItemUid> {
    let mut uids = Vec::new();
    if attached.is_none() {
        return uids;
    }
    for &node in nodes {
        let uid = item_uid_of(attached, node);
        if uid.is_valid() {
            append_unique(&mut uids, uid);
        }
    }
    uids
}

impl NodeMaps {
    /// `selectForwardMap(theKind, ...)`: Generated images, else Modified ones.
    fn forward_mut(&mut self, kind: HistoryKind) -> &mut DataMap<NodeId, Vec<NodeId>> {
        if kind == HistoryKind::Generated {
            &mut self.original_to_generated
        } else {
            &mut self.original_to_modified
        }
    }

    // One original-to-derived link: the reverse map, then the forward images.
    fn link(&mut self, kind: HistoryKind, original: NodeId, derived: NodeId) {
        append_derived_origin(&mut self.derived_to_originals, derived, original);
        append_forward(self.forward_mut(kind), original, derived);
    }
}

impl<K: HistoryKey> KeyedMaps<K> {
    /// `selectUidForwardMap` / `selectItemUidForwardMap`.
    pub(super) fn forward_mut(&mut self, kind: HistoryKind) -> &mut DataMap<K, Vec<K>> {
        if kind == HistoryKind::Generated {
            &mut self.original_to_generated
        } else {
            &mut self.original_to_modified
        }
    }
}

impl HistoryState {
    // The deleted-set update of a node event: the node and its valid item UID.
    fn mark_deleted(&mut self, node: NodeId, uid: ItemUid) {
        self.nodes.deleted.add(node);
        if uid.is_valid() {
            self.item_uids.deleted.add(uid);
        }
    }

    // The map updates of a `Record` event with images: every image other than the
    // original is linked (reverse and forward maps, then the item-UID images). The
    // native assertion that `kind` is not Deleted is compiled out in release builds, so
    // a Deleted event with images feeds the Modified map. Replaced marks the original.
    fn link_images(
        &mut self,
        kind: HistoryKind,
        (original, original_uid): (NodeId, ItemUid),
        (images, item_images): (&[NodeId], &[ItemUid]),
    ) {
        for &derived in images.iter().filter(|&&derived| derived != original) {
            self.nodes.link(kind, original, derived);
        }
        if original_uid.is_valid() {
            let forward = self.item_uids.forward_mut(kind);
            for &derived in item_images.iter().filter(|&&uid| uid != original_uid) {
                append_forward(forward, original_uid, derived);
            }
        }
        if kind == HistoryKind::Replaced {
            self.mark_deleted(original, original_uid);
        }
    }

    // One pair of the second `RecordBatch` loop (a pair mapping a node to itself is
    // skipped).
    fn link_pair(
        &mut self,
        kind: HistoryKind,
        (original, replacement): (NodeId, NodeId),
        (original_uid, replacement_uid): (ItemUid, ItemUid),
    ) {
        if replacement == original {
            return;
        }
        self.nodes.link(kind, original, replacement);
        if kind == HistoryKind::Replaced {
            self.nodes.deleted.add(original);
        }
        let valid = original_uid.is_valid() && replacement_uid.is_valid();
        if valid && replacement_uid != original_uid {
            append_forward(
                self.item_uids.forward_mut(kind),
                original_uid,
                replacement_uid,
            );
        }
        if kind == HistoryKind::Replaced && original_uid.is_valid() {
            self.item_uids.deleted.add(original_uid);
        }
    }
}

// The new event of a recording member.
pub(super) fn new_event(
    label: &AsciiString,
    sequence_number: usize,
    kind: HistoryKind,
) -> HistoryEvent {
    HistoryEvent {
        operation_name: label.clone(),
        sequence_number,
        record_kind: kind,
        ..HistoryEvent::default()
    }
}

impl LayerHistory {
    /// `Record(theOpLabel, theOriginal, theReplacements, theKind)`: one event mapping
    /// the original to its replacements; empty replacements make it a Deleted event
    /// whatever the kind. A Replaced event also marks the original deleted. Nothing is
    /// recorded while disabled.
    pub fn record(
        &self,
        graph: &BRepGraph,
        label: &AsciiString,
        original: NodeId,
        replacements: &[NodeId],
        kind: HistoryKind,
    ) {
        self.record_in(
            self.base.attached_graph(graph),
            label,
            original,
            replacements,
            kind,
        );
    }

    pub(super) fn record_in(
        &self,
        attached: AttachedGraph<'_>,
        label: &AsciiString,
        original: NodeId,
        replacements: &[NodeId],
        kind: HistoryKind,
    ) {
        if !self.is_enabled() {
            return;
        }
        let original_uid = item_uid_of(attached, original);
        let item_replacements = item_uids_of_nodes(attached, replacements);
        let mut state = self.state.write();
        let event_kind = if replacements.is_empty() {
            HistoryKind::Deleted
        } else {
            kind
        };
        let mut event = new_event(label, state.records.len(), event_kind);
        event.mapping.bind(original, replacements.to_vec());
        if original_uid.is_valid() {
            state.item_uids.known_inputs.add(original_uid);
            let images = item_replacements.clone();
            event.item_uid_mapping.bind(original_uid, images);
        }
        state.records.push(event);
        if replacements.is_empty() {
            state.mark_deleted(original, original_uid);
        } else {
            let images = (replacements, item_replacements.as_slice());
            state.link_images(kind, (original, original_uid), images);
        }
        drop(state);
        self.base.touch();
    }

    /// `RecordBatch(theOpLabel, theOriginals, theReplacements, theExtraInfo, theKind)`:
    /// one event of 1-to-1 pairs. Fewer replacements than originals raise
    /// Standard_OutOfRange at the first missing one (the array read of the native loop),
    /// after the item-UID inputs of the earlier pairs were registered.
    pub fn record_batch(
        &self,
        graph: &BRepGraph,
        label: &AsciiString,
        originals: &[NodeId],
        replacements: &[NodeId],
        extra_info: &AsciiString,
        kind: HistoryKind,
    ) -> Result<(), Failure> {
        let attached = self.base.attached_graph(graph);
        self.record_batch_in(attached, label, originals, replacements, extra_info, kind)
    }

    fn record_batch_in(
        &self,
        attached: AttachedGraph<'_>,
        label: &AsciiString,
        originals: &[NodeId],
        replacements: &[NodeId],
        extra_info: &AsciiString,
        kind: HistoryKind,
    ) -> Result<(), Failure> {
        if !self.is_enabled() || originals.is_empty() {
            return Ok(());
        }
        let pairs: Vec<(NodeId, NodeId)> = originals
            .iter()
            .copied()
            .zip(replacements.iter().copied())
            .collect();
        let uids: Vec<(ItemUid, ItemUid)> = pairs
            .iter()
            .map(|&(original, replacement)| {
                (
                    item_uid_of(attached, original),
                    item_uid_of(attached, replacement),
                )
            })
            .collect();
        let mut event = new_event(label, self.nb_records(), kind);
        event.extra_info = extra_info.clone();
        self.fill_batch_event(&mut event, originals.len(), &pairs, &uids)?;
        let mut state = self.state.write();
        state.records.push(event);
        let extent = state.nodes.forward_mut(kind).extent();
        state
            .nodes
            .forward_mut(kind)
            .resize(extent + originals.len());
        for (&pair, &pair_uids) in pairs.iter().zip(&uids) {
            state.link_pair(kind, pair, pair_uids);
        }
        drop(state);
        self.base.touch();
        Ok(())
    }

    // The first loop of `RecordBatch`: one mapping per pair (the event map pre-sized
    // for every original), registering the item-UID inputs; Standard_OutOfRange at the
    // first original without a replacement.
    fn fill_batch_event(
        &self,
        event: &mut HistoryEvent,
        nb_pairs: usize,
        pairs: &[(NodeId, NodeId)],
        uids: &[(ItemUid, ItemUid)],
    ) -> Result<(), Failure> {
        event.mapping.resize(nb_pairs);
        for (&(original, replacement), &(original_uid, replacement_uid)) in pairs.iter().zip(uids) {
            event.mapping.bind(original, vec![replacement]);
            if !original_uid.is_valid() {
                continue;
            }
            self.state.write().item_uids.known_inputs.add(original_uid);
            let images = if replacement_uid.is_valid() {
                vec![replacement_uid]
            } else {
                Vec::new()
            };
            event.item_uid_mapping.bind(original_uid, images);
        }
        if pairs.len() < nb_pairs {
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!(
                    "NCollection_Array1::at - replacement index {} out of range [0, {}) \
                     (RecordBatch expects one replacement per original)",
                    pairs.len(),
                    pairs.len()
                ),
                "",
            ));
        }
        Ok(())
    }

    /// `RecordDeleted(theOpLabel, theDeleted)`: one Deleted event listing the inputs
    /// (with their item UIDs), each added to the deleted sets.
    pub fn record_deleted(&self, graph: &BRepGraph, label: &AsciiString, deleted: &[NodeId]) {
        self.record_deleted_in(self.base.attached_graph(graph), label, deleted);
    }

    pub(super) fn record_deleted_in(
        &self,
        attached: AttachedGraph<'_>,
        label: &AsciiString,
        deleted: &[NodeId],
    ) {
        if !self.is_enabled() || deleted.is_empty() {
            return;
        }
        let uids: Vec<ItemUid> = deleted
            .iter()
            .map(|&node| item_uid_of(attached, node))
            .collect();
        let mut state = self.state.write();
        let mut event = new_event(label, state.records.len(), HistoryKind::Deleted);
        event.mapping.resize(deleted.len());
        event.item_uid_mapping.resize(deleted.len());
        for (&node, &uid) in deleted.iter().zip(&uids) {
            if !event.mapping.is_bound(&node) {
                event.mapping.bind(node, Vec::new());
            }
            state.nodes.deleted.add(node);
            if uid.is_valid() {
                state.item_uids.known_inputs.add(uid);
                state.item_uids.deleted.add(uid);
                if !event.item_uid_mapping.is_bound(&uid) {
                    event.item_uid_mapping.bind(uid, Vec::new());
                }
            }
        }
        state.records.push(event);
        drop(state);
        self.base.touch();
    }

    /// `RecordReplaced(theOpLabel, theOriginal, theReplacement)`: a Replaced `Record`
    /// of one replacement; nothing for an invalid id.
    pub fn record_replaced(
        &self,
        graph: &BRepGraph,
        label: &AsciiString,
        original: NodeId,
        replacement: NodeId,
    ) {
        if !original.is_valid() || !replacement.is_valid() {
            return;
        }
        self.record(
            graph,
            label,
            original,
            &[replacement],
            HistoryKind::Replaced,
        );
    }

    /// `RecordReplacedBatch(theOpLabel, theOriginals, theReplacements, theExtraInfo)`:
    /// `RecordBatch` of kind Replaced.
    pub fn record_replaced_batch(
        &self,
        graph: &BRepGraph,
        label: &AsciiString,
        originals: &[NodeId],
        replacements: &[NodeId],
        extra_info: &AsciiString,
    ) -> Result<(), Failure> {
        self.record_batch(
            graph,
            label,
            originals,
            replacements,
            extra_info,
            HistoryKind::Replaced,
        )
    }
}
