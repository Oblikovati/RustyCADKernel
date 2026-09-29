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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_CacheRegistry.hxx and
// BRepGraph_CacheRegistry.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_CacheRegistry`: the GUID-keyed registry of a graph's cache services.
use crate::brep_graph::BRepGraph;
use crate::brep_graph::cache::{BRepGraphCache, CacheType, attach_graph, detach_graph};
use crate::brep_graph::copy_remap::{CopyRemap, ItemMap, MappingKind, Mode};
use crate::brep_graph::registry_iterator::CacheIterator;
use rustycad_tkernel::standard::{Failure, Guid, Handle, SharedState};
use std::collections::HashMap;

/// `myCaches` and `myGuidToSlot`, behind the registry lock.
#[derive(Default)]
struct CacheSlots {
    caches: Vec<Handle<dyn BRepGraphCache>>,
    guid_to_slot: HashMap<Guid, u32>,
}

/// `BRepGraph_CacheRegistry`: registered cache services in a slot vector with a GUID
/// lookup, bound to the graph that owns the registry (its attachment token; 0 for a
/// standalone registry). The services own their data; the registry manages identity
/// and binding only.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// let graph = BRepGraph::new();
/// assert_eq!(graph.cache_registry().nb_caches(), 0);
/// assert!(graph.cache_registry().cache(0).is_null());
/// ```
pub struct CacheRegistry {
    slots: SharedState<CacheSlots>,
    graph_token: u64,
}

impl Default for CacheRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CacheRegistry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CacheRegistry")
            .field("nb_caches", &self.nb_caches())
            .field("graph_token", &self.graph_token)
            .finish()
    }
}

impl CacheRegistry {
    /// `BRepGraph_CacheRegistry()`: an empty standalone registry.
    pub fn new() -> Self {
        Self::for_graph(0)
    }

    /// The registry a graph owns (`Attach(this)`).
    pub(crate) fn for_graph(graph_token: u64) -> Self {
        Self {
            slots: SharedState::new(CacheSlots::default()),
            graph_token,
        }
    }

    /// `RegisterCache(theCache)`: the slot of the service, replacing (and detaching) a
    /// different service with the same GUID. Standard_ProgramError for a null service
    /// or one attached to another graph.
    pub fn register_cache(&self, cache: &Handle<dyn BRepGraphCache>) -> Result<u32, Failure> {
        let mut slots = self.slots.write();
        self.register_cache_locked(&mut slots, cache)
    }

    /// `Register(theCache)`: `RegisterCache`.
    pub fn register(&self, cache: &Handle<dyn BRepGraphCache>) -> Result<u32, Failure> {
        self.register_cache(cache)
    }

    // `registerCacheLocked`.
    fn register_cache_locked(
        &self,
        slots: &mut CacheSlots,
        cache: &Handle<dyn BRepGraphCache>,
    ) -> Result<u32, Failure> {
        let object = self.check_registrable(cache)?;
        if let Some(&slot) = slots.guid_to_slot.get(&object.id()) {
            let previous = &slots.caches[slot as usize];
            if let Some(previous_object) = previous.get()
                && previous.object_address() != cache.object_address()
            {
                detach_graph(previous_object);
            }
            attach_graph(object, self.graph_token);
            slots.caches[slot as usize] = cache.clone();
            return Ok(slot);
        }
        let new_slot = u32::try_from(slots.caches.len()).map_err(|_| {
            Failure::with_type(
                "Standard_OutOfRange",
                &format!(
                    "BRepGraph_CacheRegistry - too many registered caches ({}, expected at most \
                     u32::MAX)",
                    slots.caches.len()
                ),
                "",
            )
        })?;
        attach_graph(object, self.graph_token);
        slots.caches.push(cache.clone());
        slots.guid_to_slot.insert(object.id(), new_slot);
        Ok(new_slot)
    }

    // The two `Standard_ProgramError_Raise_if` checks of `registerCacheLocked`.
    fn check_registrable<'c>(
        &self,
        cache: &'c Handle<dyn BRepGraphCache>,
    ) -> Result<&'c dyn BRepGraphCache, Failure> {
        let Some(object) = cache.get() else {
            return Err(program_failure(
                "Standard_ProgramError",
                "RegisterCache() - null cache (expected a non-null handle)",
            ));
        };
        let token = object.cache_base().graph_token();
        if token != 0 && token != self.graph_token {
            return Err(program_failure(
                "Standard_ProgramError",
                &format!(
                    "RegisterCache() - cache is attached to another graph (token {token}, \
                     expected {})",
                    self.graph_token
                ),
            ));
        }
        Ok(object)
    }

    /// `UnregisterCache(theGUID)`: moves the last service into the freed slot and
    /// detaches the removed one (which clears it).
    pub fn unregister_cache(&self, guid: &Guid) {
        let mut slots = self.slots.write();
        let Some(slot) = slots.guid_to_slot.get(guid).copied() else {
            return;
        };
        let removed = slots.caches.swap_remove(slot as usize);
        if let Some(moved) = slots.caches.get(slot as usize).and_then(Handle::get) {
            let moved_id = moved.id();
            slots.guid_to_slot.insert(moved_id, slot);
        }
        slots.guid_to_slot.remove(guid);
        if let Some(cache) = removed.get() {
            detach_graph(cache);
        }
    }

    /// `FindCache(theGUID)`: the registered service, a null handle when absent.
    pub fn find_cache(&self, guid: &Guid) -> Handle<dyn BRepGraphCache> {
        let slots = self.slots.read();
        find_cache_locked(&slots, guid)
    }

    /// `Find<T>()` and `FindCache<T>()`: the service registered under `T::GetID()` when
    /// it is a `T`, a null handle otherwise.
    pub fn find<T: CacheType>(&self) -> Handle<T> {
        self.find_cache(&T::get_id()).cast::<T>()
    }

    /// `Ensure<T>()`: the service registered under `T::GetID()`, registering a new `T`
    /// first when there is none.
    pub fn ensure<T: CacheType>(&self) -> Result<Handle<T>, Failure> {
        let guid = T::get_id();
        let found = self.find_cache(&guid);
        if found.is_some() {
            return Ok(found.cast::<T>());
        }
        let mut slots = self.slots.write();
        let found = find_cache_locked(&slots, &guid);
        if found.is_some() {
            return Ok(found.cast::<T>());
        }
        let created = Handle::new(T::default()).cast::<dyn BRepGraphCache>();
        self.register_cache_locked(&mut slots, &created)?;
        Ok(created.cast::<T>())
    }

    /// `FindSlot(theGUID, theSlot)`: the slot of a registered GUID.
    pub fn find_slot(&self, guid: &Guid) -> Option<u32> {
        self.slots.read().guid_to_slot.get(guid).copied()
    }

    /// `FindSlot(theCache, theSlot)`: the slot of the service's GUID (`None` for a null
    /// handle).
    pub fn find_slot_of(&self, cache: &Handle<dyn BRepGraphCache>) -> Option<u32> {
        cache.get().and_then(|object| self.find_slot(&object.id()))
    }

    /// `Cache(theSlot)`: the service in a slot, a null handle past the end.
    pub fn cache(&self, slot: u32) -> Handle<dyn BRepGraphCache> {
        let slots = self.slots.read();
        slots.caches.get(slot as usize).cloned().unwrap_or_default()
    }

    /// `NbCaches()`: the number of registered services.
    pub fn nb_caches(&self) -> u32 {
        self.slots.read().caches.len() as u32
    }

    /// `CacheIter()`: an iterator over the registered services.
    pub fn cache_iter(&self) -> CacheIterator<'_> {
        CacheIterator::new(self)
    }

    /// `ClearAll()`: `Clear` on every service, keeping the registrations.
    pub fn clear_all(&self) {
        for slot in 0_u32.. {
            let cache = self.cache(slot);
            let Some(cache) = cache.get() else {
                return;
            };
            cache.clear();
        }
    }

    /// `CopyFreshCachesTo(theTargetGraph, theItemRemap, theMode)`: `CopyFreshTo` on every
    /// service of `source` (whose registry this is natively) in slot order, into a
    /// distinct target graph.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::cache_registry::CacheRegistry;
    /// use rustycad_tkbrep::brep_graph::copy_remap::{ItemMap, Mode};
    /// use rustycad_tkbrep::brep_graph::BRepGraph;
    /// let (source, mut target) = (BRepGraph::new(), BRepGraph::new());
    /// CacheRegistry::copy_fresh_caches_to(&source, &mut target, &ItemMap::default(), Mode::Copy)
    ///     .unwrap();
    /// assert_eq!(target.cache_registry().nb_caches(), 0);
    /// ```
    pub fn copy_fresh_caches_to(
        source: &BRepGraph,
        target: &mut BRepGraph,
        items: &ItemMap,
        mode: Mode,
    ) -> Result<(), Failure> {
        let copy = CopyRemap::new(source, target, items, mode);
        source.cache_registry().copy_each_slot(&copy)
    }

    /// `CopyFreshCachesTo(theTargetGraph, theMappingKind, theMode)`: the identity (or
    /// empty explicit) mapping.
    pub fn copy_fresh_caches_to_with_mapping(
        source: &BRepGraph,
        target: &mut BRepGraph,
        mapping_kind: MappingKind,
        mode: Mode,
    ) -> Result<(), Failure> {
        let copy = CopyRemap::with_mapping(source, target, mapping_kind, mode);
        source.cache_registry().copy_each_slot(&copy)
    }

    /// `CopyFreshCachesTo(theGraph, theItemRemap, theMode)` called on the registry of
    /// `theGraph` itself: the services copy onto their own graph through the item map.
    pub fn copy_fresh_caches_in_place(
        graph: &mut BRepGraph,
        items: &ItemMap,
        mode: Mode,
    ) -> Result<(), Failure> {
        let copy = CopyRemap::in_place(graph, Some(items), MappingKind::Explicit, mode);
        copy.source_graph().cache_registry().copy_each_slot(&copy)
    }

    // The copy loop: one slot read per service until the first empty slot. A
    // standalone registry has no source graph and copies nothing, as natively; a Rust
    // registry reached from a graph is always attached.
    fn copy_each_slot(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        if self.graph_token == 0 {
            return Ok(());
        }
        for slot in 0_u32.. {
            let cache = self.cache(slot);
            let Some(cache) = cache.get() else {
                return Ok(());
            };
            cache.copy_fresh_to(copy)?;
        }
        Ok(())
    }

    /// `Clear()`: detaches and unregisters every service.
    pub fn clear(&self) {
        let mut slots = self.slots.write();
        for cache in slots.caches.iter().filter_map(Handle::get) {
            detach_graph(cache);
        }
        slots.caches.clear();
        slots.guid_to_slot.clear();
    }
}

impl Drop for CacheRegistry {
    // `~BRepGraph` calls `Detach()`: every service is detached (and cleared).
    fn drop(&mut self) {
        for cache in self.slots.read().caches.iter().filter_map(Handle::get) {
            detach_graph(cache);
        }
    }
}

fn find_cache_locked(slots: &CacheSlots, guid: &Guid) -> Handle<dyn BRepGraphCache> {
    slots
        .guid_to_slot
        .get(guid)
        .and_then(|&slot| slots.caches.get(slot as usize).cloned())
        .unwrap_or_default()
}

fn program_failure(kind: &'static str, message: &str) -> Failure {
    Failure::with_type(kind, &format!("BRepGraph_CacheRegistry::{message}"), "")
}
