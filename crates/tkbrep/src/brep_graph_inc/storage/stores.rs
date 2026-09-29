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
// Rust translation of the DefStore, RefStore and RepStore templates of
// ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_Storage.hxx and the store helpers of
// BRepGraphInc_Storage.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The per-kind stores of the storage: records plus their bit-flag planes.
use super::super::{BitFlags, SharedBitFlags};
use std::sync::atomic::{AtomicU32, Ordering};

/// `DefStore<T>` and `RefStore<T>` (the same fields natively): the records of one kind,
/// the five flag planes, the active count and the per-kind UID counter.
#[derive(Debug)]
pub struct EntityStore<T> {
    pub(crate) items: Vec<T>,
    pub(crate) removed: BitFlags,
    // Written by BRepGraph_LayerLock through `&BRepGraph` (docs/topology-design.md 11.10).
    pub(crate) owned: SharedBitFlags,
    pub(crate) guard: BitFlags,
    pub(crate) has_compound_parent: BitFlags,
    pub(crate) has_occurrence_parent: BitFlags,
    pub(crate) nb_active: u32,
    // UIDs stay monotonic across `Clear()` cycles, so `clear` never resets the counter.
    pub(crate) next_uid_counter: AtomicU32,
}

impl<T> Default for EntityStore<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            removed: BitFlags::new(),
            owned: SharedBitFlags::new(),
            guard: BitFlags::new(),
            has_compound_parent: BitFlags::new(),
            has_occurrence_parent: BitFlags::new(),
            nb_active: 0,
            next_uid_counter: AtomicU32::new(1),
        }
    }
}

impl<T: Default> EntityStore<T> {
    /// `Append()`: a default record, one more active slot, every plane resized.
    pub(crate) fn append(&mut self) -> u32 {
        let index = self.items.len() as u32;
        self.nb_active += 1;
        self.items.push(T::default());
        self.resize_planes(index as usize + 1);
        index
    }

    /// `prepareDefStore`/`prepareRefStore`: `count` default slots, all active.
    pub(crate) fn prepare(&mut self, count: u32) {
        prepare_items(&mut self.items, count);
        self.resize_planes(count as usize);
        self.nb_active = count;
    }
}

impl<T> EntityStore<T> {
    /// `Nb()`.
    pub(crate) fn nb(&self) -> u32 {
        self.items.len() as u32
    }

    fn resize_planes(&mut self, count: usize) {
        self.removed.resize(count);
        self.owned.resize(count);
        self.guard.resize(count);
        self.has_compound_parent.resize(count);
        self.has_occurrence_parent.resize(count);
    }

    /// `DecrementActive()`: never below zero (the underflow assertion is debug-only).
    pub(crate) fn decrement_active(&mut self) {
        self.nb_active = self.nb_active.saturating_sub(1);
    }

    /// `MarkRemoved(theId)`: true when the slot turns from active to removed.
    pub(crate) fn mark_removed(&mut self, index: u32) -> bool {
        if index >= self.nb() || self.removed.test(index) {
            return false;
        }
        self.removed.set(index);
        self.decrement_active();
        true
    }

    /// `Clear()`: the records and the flags; the UID counter is kept.
    pub(crate) fn clear(&mut self) {
        self.items = Vec::new();
        self.removed.clear_all();
        self.owned.clear_all();
        self.guard.clear_all();
        self.has_compound_parent.clear_all();
        self.has_occurrence_parent.clear_all();
        self.nb_active = 0;
    }

    /// `NextUIDCounter.load`.
    pub(crate) fn next_uid(&self) -> u32 {
        self.next_uid_counter.load(Ordering::Relaxed)
    }

    /// `NextUIDCounter.store`.
    pub(crate) fn set_next_uid(&self, counter: u32) {
        self.next_uid_counter.store(counter, Ordering::Relaxed);
    }

    /// `recountActiveStore`: the active count from the removed plane.
    pub(crate) fn recount_active(&mut self) {
        let removed = &self.removed;
        self.nb_active = (0..self.nb()).filter(|index| !removed.test(*index)).count() as u32;
    }
}

/// `RepStore<T>`: the representation records of one kind, the removed plane and the
/// active count (no ownership, guard or parent planes).
#[derive(Debug, Default)]
pub struct RepStore<T> {
    pub(crate) items: Vec<T>,
    pub(crate) removed: BitFlags,
    pub(crate) nb_active: u32,
}

impl<T: Default> RepStore<T> {
    /// `Append()`.
    pub(crate) fn append(&mut self) -> u32 {
        let index = self.items.len() as u32;
        self.nb_active += 1;
        self.items.push(T::default());
        self.removed.resize(index as usize + 1);
        index
    }

    /// `prepareRepStore`.
    pub(crate) fn prepare(&mut self, count: u32) {
        prepare_items(&mut self.items, count);
        self.removed.resize(count as usize);
        self.nb_active = count;
    }
}

impl<T> RepStore<T> {
    /// `Nb()`.
    pub(crate) fn nb(&self) -> u32 {
        self.items.len() as u32
    }

    /// `MarkRemoved(theId)`.
    pub(crate) fn mark_removed(&mut self, index: u32) -> bool {
        if index >= self.nb() || self.removed.test(index) {
            return false;
        }
        self.removed.set(index);
        self.nb_active = self.nb_active.saturating_sub(1);
        true
    }

    /// `Clear()`.
    pub(crate) fn clear(&mut self) {
        self.items = Vec::new();
        self.removed.clear_all();
        self.nb_active = 0;
    }

    /// `recountActiveStore`.
    pub(crate) fn recount_active(&mut self) {
        let removed = &self.removed;
        self.nb_active = (0..self.nb()).filter(|index| !removed.test(*index)).count() as u32;
    }
}

/// The removal planes every store has (`TypedStorePlanes<T>::Nb/NbActive/Removed`).
pub trait RemovalPlanes {
    /// `Nb()`.
    fn count(&self) -> u32;
    /// The removed plane.
    fn removed_plane(&self) -> &BitFlags;
    /// The mutable removed plane and active count.
    fn removal_mut(&mut self) -> (&mut BitFlags, &mut u32);
}

/// The ownership, guard and parent planes of the definition and reference stores.
pub trait OwnershipPlanes: RemovalPlanes {
    /// `OwnedFlags`, which the lock layer writes through a shared reference.
    fn owned_plane(&self) -> &SharedBitFlags;
    /// The plane selected by `plane`.
    fn plane(&self, plane: FlagPlane) -> &BitFlags;
    /// The mutable plane selected by `plane`.
    fn plane_mut(&mut self, plane: FlagPlane) -> &mut BitFlags;
}

/// The non-removal planes of a definition or reference store other than the owned
/// plane ([`OwnershipPlanes::owned_plane`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlagPlane {
    /// `GuardFlags`.
    Guard,
    /// `HasCompoundParentFlags`.
    HasCompoundParent,
    /// `HasOccurrenceParentFlags`.
    HasOccurrenceParent,
}

impl<T> RemovalPlanes for EntityStore<T> {
    fn count(&self) -> u32 {
        self.nb()
    }
    fn removed_plane(&self) -> &BitFlags {
        &self.removed
    }
    fn removal_mut(&mut self) -> (&mut BitFlags, &mut u32) {
        (&mut self.removed, &mut self.nb_active)
    }
}

impl<T> OwnershipPlanes for EntityStore<T> {
    fn owned_plane(&self) -> &SharedBitFlags {
        &self.owned
    }
    fn plane(&self, plane: FlagPlane) -> &BitFlags {
        match plane {
            FlagPlane::Guard => &self.guard,
            FlagPlane::HasCompoundParent => &self.has_compound_parent,
            FlagPlane::HasOccurrenceParent => &self.has_occurrence_parent,
        }
    }
    fn plane_mut(&mut self, plane: FlagPlane) -> &mut BitFlags {
        match plane {
            FlagPlane::Guard => &mut self.guard,
            FlagPlane::HasCompoundParent => &mut self.has_compound_parent,
            FlagPlane::HasOccurrenceParent => &mut self.has_occurrence_parent,
        }
    }
}

impl<T> RemovalPlanes for RepStore<T> {
    fn count(&self) -> u32 {
        self.nb()
    }
    fn removed_plane(&self) -> &BitFlags {
        &self.removed
    }
    fn removal_mut(&mut self) -> (&mut BitFlags, &mut u32) {
        (&mut self.removed, &mut self.nb_active)
    }
}

// `prepareDynamicArray`: a zero count releases the array; otherwise the array grows to
// `count` default slots (`SetValue(count - 1, T())`) and keeps longer content.
fn prepare_items<T: Default>(items: &mut Vec<T>, count: u32) {
    if count == 0 {
        *items = Vec::new();
        return;
    }
    let count = count as usize;
    if items.len() < count {
        items.resize_with(count, T::default);
    } else {
        items[count - 1] = T::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_store_tracks_active_slots() {
        let mut store: EntityStore<u8> = EntityStore::default();
        assert_eq!(store.append(), 0);
        assert_eq!(store.append(), 1);
        assert!(store.mark_removed(1));
        assert!(!store.mark_removed(1));
        assert!(!store.mark_removed(5));
        assert_eq!(store.nb_active, 1);
        store.nb_active = 7;
        store.recount_active();
        assert_eq!(store.nb_active, 1);
        store.set_next_uid(4);
        store.clear();
        assert_eq!((store.nb(), store.next_uid()), (0, 4));
        store.prepare(3);
        assert_eq!(
            (store.nb(), store.nb_active, store.removed.bit_count()),
            (3, 3, 3)
        );
        assert!(!store.plane(FlagPlane::Guard).has_any_bit_set());
    }

    #[test]
    fn rep_store_tracks_active_slots() {
        let mut store: RepStore<u8> = RepStore::default();
        store.prepare(2);
        assert!(store.mark_removed(0));
        store.recount_active();
        assert_eq!(store.nb_active, 1);
        assert_eq!(store.removed_plane().bit_count(), 2);
        store.clear();
        assert_eq!(store.count(), 0);
    }
}
