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
// Rust translation of the query members of BRepGraph_LayerHistory
// (ModelingData/TKBRep/BRepGraph/BRepGraph_LayerHistory.cxx: FindOriginal,
// FindDerived, FindModified, FindGenerated, FindOriginals, IsDeleted, Deleted*,
// HasKnownInput) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The history queries. The native members that return a pointer or reference into
//! a map return copies here (the maps live behind the layer lock).
use super::LayerHistory;
use super::record::append_unique;
use crate::brep_graph::{BRepGraph, ItemUid, NodeId, Uid};
use rustycad_tkernel::ncollection::FlatMap;

impl LayerHistory {
    /// `FindOriginal(theModified)`: follows the first origin of each derived node back
    /// to a root, at most once per reverse-map entry (a cycle guard); the node itself
    /// when it derives from nothing.
    pub fn find_original(&self, modified: NodeId) -> NodeId {
        let state = self.state.read();
        let reverse = &state.nodes.derived_to_originals;
        let mut current = modified;
        let mut budget = reverse.extent();
        while budget > 0 {
            let Some(first) = reverse.seek(&current).and_then(|origins| origins.first()) else {
                break;
            };
            if *first == current {
                break;
            }
            current = *first;
            budget -= 1;
        }
        current
    }

    /// `FindDerived(theOriginal)`: every transitively derived node (Modified and
    /// Generated images) in breadth-first order, without the original.
    pub fn find_derived(&self, original: NodeId) -> Vec<NodeId> {
        let state = self.state.read();
        let nodes = &state.nodes;
        let (mut derived, mut queue) = (Vec::new(), vec![original]);
        let mut visited: FlatMap<NodeId> = FlatMap::default();
        visited.add(original);
        let mut front = 0;
        while front < queue.len() {
            let node = queue[front];
            front += 1;
            if node != original {
                derived.push(node);
            }
            for images in [&nodes.original_to_modified, &nodes.original_to_generated] {
                for &image in images.seek(&node).into_iter().flatten() {
                    if visited.add(image) {
                        queue.push(image);
                    }
                }
            }
        }
        derived
    }

    /// `FindModified(theOriginal)`: the direct Modified images, `None` without a record
    /// (which does not mean deleted).
    pub fn find_modified(&self, original: NodeId) -> Option<Vec<NodeId>> {
        self.state
            .read()
            .nodes
            .original_to_modified
            .seek(&original)
            .cloned()
    }

    /// `FindGenerated(theOriginal)`: the direct Generated images.
    pub fn find_generated(&self, original: NodeId) -> Option<Vec<NodeId>> {
        self.state
            .read()
            .nodes
            .original_to_generated
            .seek(&original)
            .cloned()
    }

    /// `FindOriginals(theDerived)`: every immediate origin of a derived node.
    pub fn find_originals(&self, derived: NodeId) -> Option<Vec<NodeId>> {
        self.state
            .read()
            .nodes
            .derived_to_originals
            .seek(&derived)
            .cloned()
    }

    /// `IsDeleted(theOriginal)`: in the deleted set.
    pub fn is_deleted(&self, original: NodeId) -> bool {
        self.state.read().nodes.deleted.contains(&original)
    }

    /// `DeletedNodes()`: a copy of the deleted set (same slot order).
    pub fn deleted_nodes(&self) -> FlatMap<NodeId> {
        self.state.read().nodes.deleted.clone()
    }

    /// `FindModified(theUID)`: the UID-keyed Modified images.
    pub fn find_modified_uid(&self, uid: &Uid) -> Option<Vec<Uid>> {
        self.state
            .read()
            .uids
            .original_to_modified
            .seek(uid)
            .cloned()
    }

    /// `FindGenerated(theUID)`: the UID-keyed Generated images.
    pub fn find_generated_uid(&self, uid: &Uid) -> Option<Vec<Uid>> {
        self.state
            .read()
            .uids
            .original_to_generated
            .seek(uid)
            .cloned()
    }

    /// `IsDeleted(theUID)`: in the UID deleted set.
    pub fn is_deleted_uid(&self, uid: &Uid) -> bool {
        self.state.read().uids.deleted.contains(uid)
    }

    /// `DeletedUids()`: a copy of the UID deleted set.
    pub fn deleted_uids(&self) -> FlatMap<Uid> {
        self.state.read().uids.deleted.clone()
    }

    /// `HasKnownInput(theUID)`: registered as an operation input.
    pub fn has_known_input_uid(&self, uid: &Uid) -> bool {
        self.state.read().uids.known_inputs.contains(uid)
    }

    /// `FindModified(theUID)` over item UIDs.
    pub fn find_modified_item_uid(&self, uid: &ItemUid) -> Option<Vec<ItemUid>> {
        self.state
            .read()
            .item_uids
            .original_to_modified
            .seek(uid)
            .cloned()
    }

    /// `FindGenerated(theUID)` over item UIDs.
    pub fn find_generated_item_uid(&self, uid: &ItemUid) -> Option<Vec<ItemUid>> {
        self.state
            .read()
            .item_uids
            .original_to_generated
            .seek(uid)
            .cloned()
    }

    /// `IsDeleted(theUID)` over item UIDs.
    pub fn is_deleted_item_uid(&self, uid: &ItemUid) -> bool {
        self.state.read().item_uids.deleted.contains(uid)
    }

    /// `DeletedItemUids()`: a copy of the item-UID deleted set.
    pub fn deleted_item_uids(&self) -> FlatMap<ItemUid> {
        self.state.read().item_uids.deleted.clone()
    }

    /// `HasKnownInput(theUID)` over item UIDs.
    pub fn has_known_input_item_uid(&self, uid: &ItemUid) -> bool {
        self.state.read().item_uids.known_inputs.contains(uid)
    }

    /// `FindModified(theGraph, theUID)`: the UID-keyed images, else the node-keyed ones
    /// of the node the UID resolves to in `graph`, as UIDs (empty when unresolved).
    pub fn find_modified_in(&self, graph: &BRepGraph, uid: &Uid) -> Vec<Uid> {
        if let Some(images) = self.find_modified_uid(uid) {
            return images;
        }
        self.node_images_as_uids(graph, uid, |history, node| history.find_modified(node))
    }

    /// `FindGenerated(theGraph, theUID)`: the Generated counterpart of
    /// `find_modified_in`.
    pub fn find_generated_in(&self, graph: &BRepGraph, uid: &Uid) -> Vec<Uid> {
        if let Some(images) = self.find_generated_uid(uid) {
            return images;
        }
        self.node_images_as_uids(graph, uid, |history, node| history.find_generated(node))
    }

    // The node fallback of `FindModified`/`FindGenerated(theGraph, theUID)`.
    fn node_images_as_uids(
        &self,
        graph: &BRepGraph,
        uid: &Uid,
        images_of: impl Fn(&Self, NodeId) -> Option<Vec<NodeId>>,
    ) -> Vec<Uid> {
        let node = graph.uids().node_id_from(uid);
        if !node.is_valid() {
            return Vec::new();
        }
        let images = images_of(self, node).unwrap_or_default();
        images
            .into_iter()
            .map(|image| graph.uids().of_node(image))
            .filter(|image_uid| image_uid.is_valid())
            .collect()
    }

    /// `IsDeleted(theGraph, theUID)`: in the UID deleted set, or the node the UID
    /// resolves to in `graph` is in the node deleted set.
    pub fn is_deleted_in(&self, graph: &BRepGraph, uid: &Uid) -> bool {
        if self.is_deleted_uid(uid) {
            return true;
        }
        let node = graph.uids().node_id_from(uid);
        node.is_valid() && self.is_deleted(node)
    }

    /// `DeletedUids(theGraph)`: the UID deleted set, then the UIDs of the deleted nodes
    /// in `graph`, without repetition, in set order.
    pub fn deleted_uids_in(&self, graph: &BRepGraph) -> Vec<Uid> {
        let state = self.state.read();
        let mut result = Vec::with_capacity(state.nodes.deleted.len() + state.uids.deleted.len());
        for &uid in state.uids.deleted.iter() {
            append_unique(&mut result, uid);
        }
        for &node in state.nodes.deleted.iter() {
            let uid = graph.uids().of_node(node);
            if uid.is_valid() {
                append_unique(&mut result, uid);
            }
        }
        result
    }
}
