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
// Rust translation of the UID members of ModelingData/TKBRep/BRepGraphInc/
// BRepGraphInc_Storage.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The per-kind UID counters, the UID allocation and the lazily rebuilt reverse indexes.
use super::Storage;
use super::tags::{NodeTag, RefTag};
use crate::brep_graph::{
    NodeId, NodeKind, NodeVisitor, RefId, RefKind, RefUid, RefVisitor, TypedId, Uid,
};
use crate::brep_graph_inc::{HasBaseDef, HasBaseRef};
use std::collections::HashMap;
use std::sync::atomic::Ordering;

// The node kinds in the order `EnsureUIDReverseIndex` walks them.
const NODE_KINDS: [NodeKind; 11] = [
    NodeKind::SOLID,
    NodeKind::SHELL,
    NodeKind::FACE,
    NodeKind::WIRE,
    NodeKind::EDGE,
    NodeKind::VERTEX,
    NodeKind::COMPOUND,
    NodeKind::COMP_SOLID,
    NodeKind::CO_EDGE,
    NodeKind::PRODUCT,
    NodeKind::OCCURRENCE,
];

// The reference kinds in the order `EnsureRefUIDReverseIndex` walks them.
const REF_KINDS: [RefKind; 7] = [
    RefKind::SHELL,
    RefKind::FACE,
    RefKind::WIRE,
    RefKind::VERTEX,
    RefKind::SOLID,
    RefKind::CHILD,
    RefKind::OCCURRENCE,
];

impl Storage {
    /// `NextNodeUIDCounter(theKind)`: 0 for an unsupported kind (release behaviour of the
    /// debug assertion).
    pub fn next_node_uid_counter(&self, kind: NodeKind) -> u32 {
        if !NodeKind::is_valid_kind(kind) {
            return 0;
        }
        NodeId::start(kind).visit(NodeCounter(self))
    }

    /// `SetNextNodeUIDCounter(theKind, theCounter)`; an unsupported kind is ignored.
    pub fn set_next_node_uid_counter(&self, kind: NodeKind, counter: u32) {
        if NodeKind::is_valid_kind(kind) {
            NodeId::start(kind).visit(NodeCounterSetter(self, counter));
        }
    }

    /// `NextRefUIDCounter(theKind)`.
    pub fn next_ref_uid_counter(&self, kind: RefKind) -> u32 {
        if !RefKind::is_valid_kind(kind) {
            return 0;
        }
        RefId::start(kind).visit(RefCounter(self))
    }

    /// `SetNextRefUIDCounter(theKind, theCounter)`.
    pub fn set_next_ref_uid_counter(&self, kind: RefKind, counter: u32) {
        if RefKind::is_valid_kind(kind) {
            RefId::start(kind).visit(RefCounterSetter(self, counter));
        }
    }

    /// `AllocateNodeUID(theNodeId)`: writes the next counter of the kind into the node,
    /// binds the reverse index and advances the counter; an invalid id gets the invalid
    /// UID. A node index past its store panics (undefined natively).
    pub fn allocate_node_uid(&mut self, node: NodeId) -> Uid {
        if !node.is_valid() {
            return Uid::invalid();
        }
        let counter = self.next_node_uid_counter(node.kind);
        self.set_next_node_uid_counter(node.kind, counter.wrapping_add(1));
        node.visit(NodeUidWriter(self, counter));
        let uid = Uid::new(node.kind, counter);
        self.uid_index().insert(uid, node);
        uid
    }

    /// `AllocateRefUID(theRefId)`: as [`allocate_node_uid`](Self::allocate_node_uid); a
    /// reference out of range gets the invalid UID and no counter.
    pub fn allocate_ref_uid(&mut self, reference: RefId) -> RefUid {
        if !reference.is_valid() || self.base_ref(reference).is_none() {
            return RefUid::invalid();
        }
        let counter = self.next_ref_uid_counter(reference.kind);
        self.set_next_ref_uid_counter(reference.kind, counter.wrapping_add(1));
        if let Some(base) = self.change_base_ref(reference) {
            base.uid = counter;
        }
        let uid = RefUid::new(reference.kind, counter);
        self.ref_uid_index().insert(uid, reference);
        uid
    }

    /// `FindNodeIdByUID(theUID)`: the active node of a UID, invalid otherwise.
    pub fn find_node_id_by_uid(&self, uid: &Uid) -> NodeId {
        if !uid.is_valid() {
            return NodeId::default();
        }
        self.ensure_uid_reverse_index();
        let found = self
            .uid_to_node_id
            .read()
            .expect("UID index lock")
            .get(uid)
            .copied();
        match found {
            Some(node) if node.visit(ActiveNode(self)) => node,
            _ => NodeId::default(),
        }
    }

    /// `FindRefIdByUID(theUID)`: the active reference of a UID, invalid otherwise.
    pub fn find_ref_id_by_uid(&self, uid: &RefUid) -> RefId {
        if !uid.is_valid() {
            return RefId::default();
        }
        self.ensure_ref_uid_reverse_index();
        let found = self
            .ref_uid_to_ref_id
            .read()
            .expect("RefUID index lock")
            .get(uid)
            .copied();
        match found {
            Some(reference) if reference.visit(ActiveRef(self)) => reference,
            _ => RefId::default(),
        }
    }

    /// `ClearUIDIndexes()` (private natively): empties both indexes, not dirty.
    pub(crate) fn clear_uid_indexes(&mut self) {
        self.uid_index().clear();
        self.uid_to_node_id_dirty.store(false, Ordering::Relaxed);
        self.ref_uid_index().clear();
        self.ref_uid_to_ref_id_dirty.store(false, Ordering::Relaxed);
    }

    /// `RebuildUIDReverseIndexes()`: marks both indexes dirty and rebuilds them.
    pub fn rebuild_uid_reverse_indexes(&mut self) {
        self.mark_uid_reverse_indexes_dirty();
        self.ensure_uid_reverse_index();
        self.ensure_ref_uid_reverse_index();
    }

    /// `MarkUIDReverseIndexesDirty()`: empties both indexes and marks them dirty.
    pub fn mark_uid_reverse_indexes_dirty(&mut self) {
        self.uid_index().clear();
        self.uid_to_node_id_dirty.store(true, Ordering::Release);
        self.ref_uid_index().clear();
        self.ref_uid_to_ref_id_dirty.store(true, Ordering::Release);
    }

    /// `EnsureUIDReverseIndex()`: rebuilds the node index from the stored UIDs of the
    /// active nodes when it is dirty.
    pub fn ensure_uid_reverse_index(&self) {
        if !self.uid_to_node_id_dirty.load(Ordering::Acquire) {
            return;
        }
        let mut index = self.uid_to_node_id.write().expect("UID index lock");
        if !self.uid_to_node_id_dirty.load(Ordering::Relaxed) {
            return;
        }
        index.clear();
        for kind in NODE_KINDS {
            NodeId::start(kind).visit(NodeIndexFiller(self, &mut index));
        }
        self.uid_to_node_id_dirty.store(false, Ordering::Release);
    }

    /// `EnsureRefUIDReverseIndex()`: rebuilds the reference index when it is dirty.
    pub fn ensure_ref_uid_reverse_index(&self) {
        if !self.ref_uid_to_ref_id_dirty.load(Ordering::Acquire) {
            return;
        }
        let mut index = self.ref_uid_to_ref_id.write().expect("RefUID index lock");
        if !self.ref_uid_to_ref_id_dirty.load(Ordering::Relaxed) {
            return;
        }
        index.clear();
        for kind in REF_KINDS {
            RefId::start(kind).visit(RefIndexFiller(self, &mut index));
        }
        self.ref_uid_to_ref_id_dirty.store(false, Ordering::Release);
    }

    /// `myUIDToNodeId.UnBind(theUID)` under the index lock: the editor forgets the UID of
    /// a removed node (the index is a friend-accessed member natively).
    pub(crate) fn unbind_node_uid(&mut self, uid: &Uid) {
        self.uid_index().remove(uid);
    }

    /// `myRefUIDToRefId.UnBind(theUID)`: the reference counterpart.
    pub(crate) fn unbind_ref_uid(&mut self, uid: &RefUid) {
        self.ref_uid_index().remove(uid);
    }

    fn uid_index(&mut self) -> &mut HashMap<Uid, NodeId> {
        self.uid_to_node_id.get_mut().expect("UID index lock")
    }

    fn ref_uid_index(&mut self) -> &mut HashMap<RefUid, RefId> {
        self.ref_uid_to_ref_id.get_mut().expect("RefUID index lock")
    }
}

struct NodeCounter<'s>(&'s Storage);

impl NodeVisitor for NodeCounter<'_> {
    type Output = u32;
    fn visit<T: NodeTag>(self, _id: TypedId<T>) -> u32 {
        T::store(self.0).next_uid()
    }
}

struct NodeCounterSetter<'s>(&'s Storage, u32);

impl NodeVisitor for NodeCounterSetter<'_> {
    type Output = ();
    fn visit<T: NodeTag>(self, _id: TypedId<T>) {
        T::store(self.0).set_next_uid(self.1);
    }
}

struct RefCounter<'s>(&'s Storage);

impl RefVisitor for RefCounter<'_> {
    type Output = u32;
    fn visit<T: RefTag>(self, _id: TypedId<T>) -> u32 {
        T::store(self.0).next_uid()
    }
}

struct RefCounterSetter<'s>(&'s Storage, u32);

impl RefVisitor for RefCounterSetter<'_> {
    type Output = ();
    fn visit<T: RefTag>(self, _id: TypedId<T>) {
        T::store(self.0).set_next_uid(self.1);
    }
}

struct NodeUidWriter<'s>(&'s mut Storage, u32);

impl NodeVisitor for NodeUidWriter<'_> {
    type Output = ();
    fn visit<T: NodeTag>(self, id: TypedId<T>) {
        self.0.change_def(id).base_mut().uid = self.1;
    }
}

// The per-kind check of `FindNodeIdByUID`: in range and not removed.
struct ActiveNode<'s>(&'s Storage);

impl NodeVisitor for ActiveNode<'_> {
    type Output = bool;
    fn visit<T: NodeTag>(self, id: TypedId<T>) -> bool {
        id.is_valid_for(T::store(self.0).nb()) && !self.0.is_removed(id)
    }
}

// The check of `FindRefIdByUID`: not removed (no range check natively).
struct ActiveRef<'s>(&'s Storage);

impl RefVisitor for ActiveRef<'_> {
    type Output = bool;
    fn visit<T: RefTag>(self, id: TypedId<T>) -> bool {
        !self.0.is_removed(id)
    }
}

// `bindFromStore` of `EnsureUIDReverseIndex`: every active node with a stored UID.
struct NodeIndexFiller<'s, 'i>(&'s Storage, &'i mut HashMap<Uid, NodeId>);

impl NodeVisitor for NodeIndexFiller<'_, '_> {
    type Output = ();
    fn visit<T: NodeTag>(self, id: TypedId<T>) {
        let store = T::store(self.0);
        let kind = NodeId::from(id).kind;
        for index in 0..store.nb() {
            let counter = store.items[index as usize].base().uid;
            if !store.removed.test(index) && counter > 0 {
                self.1
                    .insert(Uid::new(kind, counter), NodeId::new(kind, index));
            }
        }
    }
}

// `bindFromStore` of `EnsureRefUIDReverseIndex`.
struct RefIndexFiller<'s, 'i>(&'s Storage, &'i mut HashMap<RefUid, RefId>);

impl RefVisitor for RefIndexFiller<'_, '_> {
    type Output = ();
    fn visit<T: RefTag>(self, id: TypedId<T>) {
        let store = T::store(self.0);
        let kind = RefId::from(id).kind;
        for index in 0..store.nb() {
            let counter = store.items[index as usize].base().uid;
            if !store.removed.test(index) && counter > 0 {
                self.1
                    .insert(RefUid::new(kind, counter), RefId::new(kind, index));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brep_graph::{FaceId, WireRefId};

    #[test]
    fn uids_allocate_and_resolve() {
        let mut storage = Storage::new();
        let face = storage.append_face();
        let uid = storage.allocate_node_uid(face.into());
        assert_eq!(uid, Uid::new(NodeKind::FACE, 1));
        assert_eq!(storage.face(face).base.uid, 1);
        assert_eq!(storage.next_node_uid_counter(NodeKind::FACE), 2);
        assert_eq!(storage.find_node_id_by_uid(&uid), NodeId::from(face));
        storage.set_removed(face, true);
        assert!(!storage.find_node_id_by_uid(&uid).is_valid());
        storage.set_removed(face, false);
        storage.rebuild_uid_reverse_indexes();
        assert_eq!(storage.find_node_id_by_uid(&uid), NodeId::from(face));
        assert_eq!(storage.next_node_uid_counter(NodeKind(9)), 0);
        assert!(!storage.allocate_node_uid(NodeId::default()).is_valid());
    }

    #[test]
    fn ref_uids_allocate_and_resolve() {
        let mut storage = Storage::new();
        assert!(
            !storage
                .allocate_ref_uid(WireRefId::new(0).into())
                .is_valid()
        );
        let wire_ref = storage.append_wire_ref();
        let uid = storage.allocate_ref_uid(wire_ref.into());
        assert_eq!(uid, RefUid::new(RefKind::WIRE, 1));
        storage.mark_uid_reverse_indexes_dirty();
        assert_eq!(storage.find_ref_id_by_uid(&uid), RefId::from(wire_ref));
        storage.set_next_ref_uid_counter(RefKind::WIRE, 7);
        assert_eq!(storage.next_ref_uid_counter(RefKind::WIRE), 7);
        storage.clear_uid_indexes();
        assert!(!storage.find_ref_id_by_uid(&uid).is_valid());
        let _ = FaceId::default();
    }
}
