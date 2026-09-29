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

//! Direct translation of BRepGraph_CacheRegistry_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported).
//!
//! The test caches are Transient cache services (docs/topology-design.md section
//! 11.10): their maps sit behind a lock so every member takes `&self`, and the
//! entry members that read the attached graph natively take the graph explicitly.
//! `BRepGraph_Transform::MoveRef` for an occurrence reference is
//! `transform::move_occurrence_ref`.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::cache::{
    BRepGraphCache, CacheBase, CacheType, ItemEntry, NodeEntry, RefEntry,
};
use rustycad_tkbrep::brep_graph::cache_registry::CacheRegistry;
use rustycad_tkbrep::brep_graph::transform;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Guid, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::{Transform3, Vector3};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, Ordering};
use support::make_box;

fn guid(theText: &str) -> Guid {
    Guid::from_ascii(theText.as_bytes()).unwrap()
}

struct TestCache {
    transient: StandardTransient,
    base: CacheBase,
    myID: Guid,
    myName: AsciiString,
}
impl_transient!(TestCache, "TestCache", dyn BRepGraphCache, transient,
    interfaces: [dyn BRepGraphCache]);

impl TestCache {
    fn create(theID: Guid, theName: &str) -> Handle<dyn BRepGraphCache> {
        let aCache = TestCache {
            transient: StandardTransient::default(),
            base: CacheBase::default(),
            myID: theID,
            myName: AsciiString::from(theName),
        };
        Handle::new(aCache).cast::<dyn BRepGraphCache>()
    }
}

impl BRepGraphCache for TestCache {
    fn cache_base(&self) -> &CacheBase {
        &self.base
    }
    fn id(&self) -> Guid {
        self.myID
    }
    fn name(&self) -> &AsciiString {
        &self.myName
    }
}

/// The `NodeValue`, `RefValue` and `ItemValue` entries with their values.
#[derive(Default)]
struct EntryMaps {
    myNodeOwnValues: HashMap<NodeId, (NodeEntry, i32)>,
    myNodeSubtreeValues: HashMap<NodeId, (NodeEntry, i32)>,
    myRefOwnValues: HashMap<RefId, (RefEntry, i32)>,
    myItemOwnValues: HashMap<ItemId, (ItemEntry, i32)>,
}

#[derive(Default)]
struct EntryTestCache {
    transient: StandardTransient,
    base: CacheBase,
    maps: Mutex<EntryMaps>,
    myClearCount: AtomicI32,
}
impl_transient!(EntryTestCache, "EntryTestCache", dyn BRepGraphCache, transient,
    interfaces: [dyn BRepGraphCache]);

impl CacheType for EntryTestCache {
    fn get_id() -> Guid {
        guid("f1f2f3f4-1234-4321-8888-010203040506")
    }
}

impl BRepGraphCache for EntryTestCache {
    fn cache_base(&self) -> &CacheBase {
        &self.base
    }
    fn id(&self) -> Guid {
        Self::get_id()
    }
    fn name(&self) -> &AsciiString {
        static THE_NAME: std::sync::OnceLock<AsciiString> = std::sync::OnceLock::new();
        THE_NAME.get_or_init(|| AsciiString::from("EntryTestCache"))
    }
    fn clear(&self) {
        self.myClearCount.fetch_add(1, Ordering::Relaxed);
        *self.maps.lock().unwrap() = EntryMaps::default();
    }
}

/// `set(theMap, theKey, theValue)`: bind or overwrite.
fn set<K: Hash + Eq, V>(theMap: &mut HashMap<K, V>, theKey: K, theValue: V) {
    theMap.insert(theKey, theValue);
}

impl EntryTestCache {
    fn Attached(&self) -> bool {
        self.base.is_attached()
    }

    fn SetNodeOwn(&self, theGraph: &BRepGraph, theNode: NodeId, theValue: i32) -> bool {
        let mut anEntry = NodeEntry::default();
        if !anEntry.bind_own_gen(&self.base, theGraph, theNode) {
            return false;
        }
        let mut aMaps = self.maps.lock().unwrap();
        set(&mut aMaps.myNodeOwnValues, theNode, (anEntry, theValue));
        true
    }

    fn GetNodeOwn(&self, theGraph: &BRepGraph, theNode: NodeId) -> Option<i32> {
        let aMaps = self.maps.lock().unwrap();
        let (anEntry, aValue) = aMaps.myNodeOwnValues.get(&theNode)?;
        anEntry
            .is_fresh_own(&self.base, theGraph, theNode)
            .then_some(*aValue)
    }

    fn SetNodeSubtree(&self, theGraph: &BRepGraph, theNode: NodeId, theValue: i32) -> bool {
        let mut anEntry = NodeEntry::default();
        if !anEntry.bind_subtree_gen(&self.base, theGraph, theNode) {
            return false;
        }
        let mut aMaps = self.maps.lock().unwrap();
        set(&mut aMaps.myNodeSubtreeValues, theNode, (anEntry, theValue));
        true
    }

    fn GetNodeSubtree(&self, theGraph: &BRepGraph, theNode: NodeId) -> Option<i32> {
        let aMaps = self.maps.lock().unwrap();
        let (anEntry, aValue) = aMaps.myNodeSubtreeValues.get(&theNode)?;
        anEntry
            .is_fresh_subtree(&self.base, theGraph, theNode)
            .then_some(*aValue)
    }

    fn SetRefOwn(&self, theGraph: &BRepGraph, theRef: RefId, theValue: i32) -> bool {
        let mut anEntry = RefEntry::default();
        if !anEntry.bind_own_gen(&self.base, theGraph, theRef) {
            return false;
        }
        let mut aMaps = self.maps.lock().unwrap();
        set(&mut aMaps.myRefOwnValues, theRef, (anEntry, theValue));
        true
    }

    fn GetRefOwn(&self, theGraph: &BRepGraph, theRef: RefId) -> Option<i32> {
        let aMaps = self.maps.lock().unwrap();
        let (anEntry, aValue) = aMaps.myRefOwnValues.get(&theRef)?;
        anEntry
            .is_fresh_own(&self.base, theGraph, theRef)
            .then_some(*aValue)
    }

    fn SetItemOwn(&self, theGraph: &BRepGraph, theItem: ItemId, theValue: i32) -> bool {
        let mut anEntry = ItemEntry::default();
        if !anEntry.bind_own_gen(&self.base, theGraph, theItem) {
            return false;
        }
        let mut aMaps = self.maps.lock().unwrap();
        set(&mut aMaps.myItemOwnValues, theItem, (anEntry, theValue));
        true
    }

    fn GetItemOwn(&self, theGraph: &BRepGraph, theItem: ItemId) -> Option<i32> {
        let aMaps = self.maps.lock().unwrap();
        let (anEntry, aValue) = aMaps.myItemOwnValues.get(&theItem)?;
        anEntry
            .is_fresh_own(&self.base, theGraph, theItem)
            .then_some(*aValue)
    }

    fn ResolveChildForTest(&self, theGraph: &BRepGraph, theRef: RefId) -> Option<NodeId> {
        self.base.resolve_active_ref_child(theGraph, theRef)
    }

    fn ClearCount(&self) -> i32 {
        self.myClearCount.load(Ordering::Relaxed)
    }

    fn NbValues(&self) -> i32 {
        let aMaps = self.maps.lock().unwrap();
        (aMaps.myNodeOwnValues.len()
            + aMaps.myNodeSubtreeValues.len()
            + aMaps.myRefOwnValues.len()
            + aMaps.myItemOwnValues.len()) as i32
    }
}

fn makeBoxGraph() -> BRepGraph {
    let mut aGraph = BRepGraph::new();
    let _aResult = aGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    aGraph
}

fn ensureEntryCache(aGraph: &BRepGraph) -> Handle<EntryTestCache> {
    aGraph
        .cache_registry()
        .ensure::<EntryTestCache>()
        .unwrap()
}

fn firstShellRef(aGraph: &BRepGraph) -> ShellRefId {
    let aSolid = SolidId::start();
    assert!(
        !aGraph
            .topo()
            .solids()
            .relations(aSolid)
            .shell_ref_ids
            .is_empty()
    );
    aGraph.topo().solids().relations(aSolid).shell_ref_ids[0]
}

#[test]
fn BRepGraph_CacheRegistryTest__Register_SameGUID_SameSlot() {
    let aGUID = guid("a1b2c3d4-1111-2222-3333-444455556666");
    let aKind1 = TestCache::create(aGUID, "SameGUID");
    let aKind2 = TestCache::create(aGUID, "SameGUID");
    let aRegistry = CacheRegistry::new();
    let aSlot1 = aRegistry.register_cache(&aKind1).unwrap();
    let aSlot2 = aRegistry.register_cache(&aKind2).unwrap();
    assert_eq!(aSlot1, aSlot2);
}

#[test]
fn BRepGraph_CacheRegistryTest__Register_DifferentGUID_DifferentSlot() {
    let aKind1 = TestCache::create(guid("b1b2c3d4-aaaa-bbbb-cccc-ddddeeee0001"), "Kind1");
    let aKind2 = TestCache::create(guid("b1b2c3d4-aaaa-bbbb-cccc-ddddeeee0002"), "Kind2");
    let aRegistry = CacheRegistry::new();
    let aSlot1 = aRegistry.register_cache(&aKind1).unwrap();
    let aSlot2 = aRegistry.register_cache(&aKind2).unwrap();
    assert_ne!(aSlot1, aSlot2);
}

#[test]
fn BRepGraph_CacheRegistryTest__FindSlot_ByGUID_ReturnsCorrectSlot() {
    let aKind = TestCache::create(guid("c1c2c3c4-1111-2222-3333-aabbccddeeff"), "FindByGUID");
    let aRegistry = CacheRegistry::new();
    let aExpectedSlot = aRegistry.register_cache(&aKind).unwrap();

    let aFoundSlot = aRegistry.find_slot(&aKind.get().unwrap().id());
    assert!(aFoundSlot.is_some());
    assert_eq!(aFoundSlot.unwrap(), aExpectedSlot);
}

#[test]
fn BRepGraph_CacheRegistryTest__FindCache_BySlot_ReturnsCorrectDescriptor() {
    let aKind = TestCache::create(guid("d1d2d3d4-5555-6666-7777-888899990000"), "FindBySlot");
    let aRegistry = CacheRegistry::new();
    let aSlot = aRegistry.register_cache(&aKind).unwrap();

    let aFound = aRegistry.cache(aSlot);
    assert!(!aFound.is_null());
    assert!(aFound.get().unwrap().id() == aKind.get().unwrap().id());
    assert!(aFound.get().unwrap().name().is_equal("FindBySlot"));
}

#[test]
fn BRepGraph_CacheRegistryTest__Cache_InvalidSlot_ReturnsNull() {
    let aRegistry = CacheRegistry::new();
    assert!(aRegistry.cache(0).is_null());
}

#[test]
fn BRepGraph_CacheRegistryTest__SameCache_CanUseDifferentGraphLocalSlots() {
    let aShared = TestCache::create(guid("e1e2e3e4-1111-2222-3333-888899990001"), "Shared");
    let aPrefix = TestCache::create(guid("e1e2e3e4-1111-2222-3333-888899990002"), "Prefix");

    let aRegistry1 = CacheRegistry::new();
    let aRegistry2 = CacheRegistry::new();
    let aSlot1 = aRegistry1.register_cache(&aShared).unwrap();
    let _aPrefixSlot = aRegistry2.register_cache(&aPrefix).unwrap();
    let aSlot2 = aRegistry2.register_cache(&aShared).unwrap();

    assert_eq!(aSlot1, 0);
    assert_eq!(aSlot2, 1);
}

#[test]
fn BRepGraph_CacheRegistryTest__Unregister_CompactsSlotsAndUpdatesIterator() {
    let aGraph = makeBoxGraph();
    let aFirst = TestCache::create(guid("c8f2fd46-7c54-42a1-a73e-83c6ae2c7401"), "First");
    let aSecond = TestCache::create(guid("ac165657-f0b6-4e51-9a1b-ef9d33c12096"), "Second");

    let aFirstSlot = aGraph.cache_registry().register_cache(&aFirst).unwrap();
    let aSecondSlot = aGraph.cache_registry().register_cache(&aSecond).unwrap();
    assert_eq!(aFirstSlot, 0);
    assert_eq!(aSecondSlot, 1);
    assert_eq!(aGraph.cache_registry().nb_caches(), 2);

    aGraph
        .cache_registry()
        .unregister_cache(&aFirst.get().unwrap().id());

    assert_eq!(aGraph.cache_registry().nb_caches(), 1);
    let aSecondId = aSecond.get().unwrap().id();
    let aFoundSlot = aGraph.cache_registry().find_slot(&aSecondId);
    assert!(aFoundSlot.is_some());
    assert_eq!(aFoundSlot.unwrap(), 0);
    let mut aIterCount = 0u32;
    for aCache in aGraph.cache_registry().cache_iter() {
        assert!(!aCache.is_null());
        assert!(aCache.get().unwrap().id() == aSecondId);
        aIterCount += 1;
    }
    assert_eq!(aIterCount, 1);
}

#[test]
fn BRepGraph_CacheRegistryTest__ResolveActiveRefChild_RejectsInvalidAndRemovedRefs() {
    let mut aGraph = makeBoxGraph();
    let aCache = ensureEntryCache(&aGraph);
    let aShellRef = firstShellRef(&aGraph);

    let aNode = aCache
        .get()
        .unwrap()
        .ResolveChildForTest(&aGraph, RefId::from(aShellRef));
    assert!(aNode.is_some());
    assert_eq!(aNode.unwrap().kind, NodeKind::SHELL);

    let aCacheRef = aCache.get().unwrap();
    assert!(aCacheRef.ResolveChildForTest(&aGraph, RefId::default()).is_none());
    let aFarRef = RefId::new(RefKind::SHELL, 999999);
    assert!(aCacheRef.ResolveChildForTest(&aGraph, aFarRef).is_none());

    aGraph
        .editor()
        .generic()
        .remove_ref(RefId::from(aShellRef))
        .unwrap();
    let aRemoved = aCacheRef.ResolveChildForTest(&aGraph, RefId::from(aShellRef));
    assert!(aRemoved.is_none());
}

#[test]
fn BRepGraph_CacheRegistryTest__NodeOwnEntry_MissesAfterOwnGenerationChange() {
    let mut aGraph = makeBoxGraph();
    let aCache = ensureEntryCache(&aGraph);

    let anEdge = NodeId::from(EdgeId::start());
    assert!(aCache.get().unwrap().SetNodeOwn(&aGraph, anEdge, 42));

    let aValue = aCache.get().unwrap().GetNodeOwn(&aGraph, anEdge);
    assert!(aValue.is_some());
    assert_eq!(aValue.unwrap(), 42);

    let mut aGuard = aGraph.editor().edges().mutate(EdgeId::start()).unwrap();
    aGuard.mark_dirty();
    drop(aGuard);

    assert!(aCache.get().unwrap().GetNodeOwn(&aGraph, anEdge).is_none());
}

#[test]
fn BRepGraph_CacheRegistryTest__NodeSubtreeEntry_MissesAfterChildGenerationChange() {
    let mut aGraph = makeBoxGraph();
    let aCache = ensureEntryCache(&aGraph);

    let aWire = NodeId::from(WireId::start());
    assert!(aCache.get().unwrap().SetNodeSubtree(&aGraph, aWire, 77));

    let aValue = aCache.get().unwrap().GetNodeSubtree(&aGraph, aWire);
    assert!(aValue.is_some());
    assert_eq!(aValue.unwrap(), 77);

    let mut aGuard = aGraph.editor().edges().mutate(EdgeId::start()).unwrap();
    aGuard.mark_dirty();
    drop(aGuard);

    assert!(aCache.get().unwrap().GetNodeSubtree(&aGraph, aWire).is_none());
}

#[test]
fn BRepGraph_CacheRegistryTest__RefOwnEntry_MissesAfterRefGenerationChange() {
    let mut aGraph = makeBoxGraph();
    let aCache = ensureEntryCache(&aGraph);

    let aProductId = ProductId::start();
    let aOccRefs = &aGraph.topo().products().relations(aProductId).occurrence_ref_ids;
    assert!(!aOccRefs.is_empty());
    let anOccRef = aOccRefs[0];
    let aRef = RefId::from(anOccRef);
    assert!(aCache.get().unwrap().SetRefOwn(&aGraph, aRef, 99));

    let aValue = aCache.get().unwrap().GetRefOwn(&aGraph, aRef);
    assert!(aValue.is_some());
    assert_eq!(aValue.unwrap(), 99);

    let aTrsf = Transform3::translation_vector(Vector3::new(1.0, 0.0, 0.0));
    assert!(transform::move_occurrence_ref(&mut aGraph, anOccRef, &aTrsf).unwrap());

    assert!(aCache.get().unwrap().GetRefOwn(&aGraph, aRef).is_none());
}

#[test]
fn BRepGraph_CacheRegistryTest__ItemOwnEntry_WorksForNodeAndRef() {
    let aGraph = makeBoxGraph();
    let aCache = ensureEntryCache(&aGraph);
    let aCacheRef = aCache.get().unwrap();

    let aFaceItem = ItemId::from(NodeId::from(FaceId::start()));
    assert!(aCacheRef.SetItemOwn(&aGraph, aFaceItem, 11));

    let aShellRef = firstShellRef(&aGraph);
    let aRefItem = ItemId::from(RefId::from(aShellRef));
    assert!(aCacheRef.SetItemOwn(&aGraph, aRefItem, 22));

    let aValue = aCacheRef.GetItemOwn(&aGraph, aFaceItem);
    assert!(aValue.is_some());
    assert_eq!(aValue.unwrap(), 11);
    let aValue = aCacheRef.GetItemOwn(&aGraph, aRefItem);
    assert!(aValue.is_some());
    assert_eq!(aValue.unwrap(), 22);
}

#[test]
fn BRepGraph_CacheRegistryTest__Unregister_DetachesAndClearsRepresentation() {
    let aGraph = makeBoxGraph();
    let aCache = ensureEntryCache(&aGraph);
    let aCacheRef = aCache.get().unwrap();
    let anEdge = NodeId::from(EdgeId::start());
    assert!(aCacheRef.SetNodeOwn(&aGraph, anEdge, 1));
    assert_eq!(aCacheRef.NbValues(), 1);

    aGraph
        .cache_registry()
        .unregister_cache(&EntryTestCache::get_id());

    assert!(!aCacheRef.Attached());
    assert_eq!(aCacheRef.NbValues(), 0);
    assert!(aCacheRef.ClearCount() >= 1);

    assert!(aCacheRef.GetNodeOwn(&aGraph, anEdge).is_none());
}

#[test]
fn BRepGraph_CacheRegistryTest__GraphDestruction_DetachesAndClearsExternalCacheHandle() {
    let aCache;
    {
        let aGraph = makeBoxGraph();
        aCache = ensureEntryCache(&aGraph);
        let anEdge = NodeId::from(EdgeId::start());
        assert!(aCache.get().unwrap().SetNodeOwn(&aGraph, anEdge, 5));
        assert!(aCache.get().unwrap().Attached());
        assert_eq!(aCache.get().unwrap().NbValues(), 1);
    }

    assert!(!aCache.is_null());
    assert!(!aCache.get().unwrap().Attached());
    assert_eq!(aCache.get().unwrap().NbValues(), 0);
    assert!(aCache.get().unwrap().ClearCount() >= 1);
}
