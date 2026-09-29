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

//! Direct translation of BRepGraph_Lock_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported). The probe layer keeps its records
//! behind a lock, as a Rust layer is shared through a handle. `EXPECT_THROW(...,
//! Standard_ProgramError)` checks the exception type of the returned `Failure`.
#![allow(non_snake_case, non_camel_case_types)]
mod support;
use rustycad_tkbrep::brep_graph::compact;
use rustycad_tkbrep::brep_graph::copy_remap::CopyRemap;
use rustycad_tkbrep::brep_graph::layer::{
    AttachedGraph, BRepGraphLayer, LayerBase, as_layer, kind_bit, ref_kind_bit,
};
use rustycad_tkbrep::brep_graph::layer_deferred::{LayerDeferred, RepresentationKind};
use rustycad_tkbrep::brep_graph::layer_lock::LayerLock;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Guid, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Point3;
use std::sync::Mutex;
use support::make_box;

fn makeBoxGraph() -> BRepGraph {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aResult = aGraph.shapes_mut().add(&make_box(10.0, 20.0, 30.0));
    aGraph
}

fn guid(theText: &str) -> Guid {
    Guid::from_ascii(theText.as_bytes()).unwrap()
}

fn testOwnerId() -> Guid {
    guid("2ee13474-4cc6-4f6f-bde2-f0e4dcf9e5f1")
}

fn otherOwnerId() -> Guid {
    guid("63c7da5f-4adc-4024-85f2-329525dc76b7")
}

fn lock_layer(theGraph: &BRepGraph) -> Handle<LayerLock> {
    theGraph.layer_registry().ensure::<LayerLock>().unwrap()
}

fn lockItem(theGraph: &BRepGraph, theItem: ItemId) {
    let aLayer = lock_layer(theGraph);
    aLayer.set_owner(theGraph, theItem, &testOwnerId()).unwrap();
}

fn unlockItem(theGraph: &BRepGraph, theItem: VertexId) {
    let aLayer = lock_layer(theGraph);
    let anItem = ItemId::from(NodeId::from(theItem));
    aLayer
        .unset_owner_matching(theGraph, anItem, &testOwnerId())
        .unwrap();
}

/// The four `isOwned` overloads.
fn isOwned(theGraph: &BRepGraph, theItem: ItemId) -> bool {
    let aLayer = theGraph.layer_registry().find::<LayerLock>();
    aLayer
        .get()
        .is_some_and(|aLayer| aLayer.has_owner(theGraph, theItem))
}

fn node(theId: impl Into<NodeId>) -> ItemId {
    ItemId::from(theId.into())
}

fn reference(theId: impl Into<RefId>) -> ItemId {
    ItemId::from(theId.into())
}

fn expect_program_error<T: std::fmt::Debug>(theResult: Result<T, Failure>) {
    assert_eq!(
        theResult.unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
}

fn start_vertex_ref(theGraph: &BRepGraph) -> VertexRefId {
    theGraph
        .topo()
        .edges()
        .definition(EdgeId::start())
        .start_vertex_ref_id
}

#[derive(Default)]
struct DispatchRecords {
    myRemovedItems: Vec<ItemId>,
    myModifiedItems: Vec<ItemId>,
}

struct BRepGraph_LayerItemDispatchProbe {
    transient: StandardTransient,
    base: LayerBase,
    myName: AsciiString,
    records: Mutex<DispatchRecords>,
}
impl_transient!(BRepGraph_LayerItemDispatchProbe, "BRepGraph_LayerItemDispatchProbe",
    dyn BRepGraphLayer, transient, interfaces: [dyn BRepGraphLayer]);

impl BRepGraph_LayerItemDispatchProbe {
    fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: LayerBase::new(),
            myName: AsciiString::new("ItemDispatchProbe"),
            records: Mutex::new(DispatchRecords::default()),
        }
    }

    fn RemovedItems(&self) -> Vec<ItemId> {
        self.records.lock().unwrap().myRemovedItems.clone()
    }

    fn ModifiedItems(&self) -> Vec<ItemId> {
        self.records.lock().unwrap().myModifiedItems.clone()
    }
}

impl BRepGraphLayer for BRepGraph_LayerItemDispatchProbe {
    fn layer_base(&self) -> &LayerBase {
        &self.base
    }
    fn id(&self) -> Guid {
        guid("9f11b126-430a-4535-aa4a-8b8a4446dd5f")
    }
    fn name(&self) -> &AsciiString {
        &self.myName
    }
    fn subscribed_kinds(&self) -> i32 {
        kind_bit(NodeKind::FACE)
    }
    fn subscribed_ref_kinds(&self) -> i32 {
        ref_kind_bit(RefKind::VERTEX)
    }
    fn on_node_removed(&self, _attached: AttachedGraph<'_>, theNode: NodeId) {
        self.records
            .lock()
            .unwrap()
            .myRemovedItems
            .push(ItemId::from(theNode));
    }
    fn on_ref_removed(&self, _attached: AttachedGraph<'_>, theRef: RefId) {
        self.records
            .lock()
            .unwrap()
            .myRemovedItems
            .push(ItemId::from(theRef));
    }
    fn on_node_modified(&self, _attached: AttachedGraph<'_>, theNode: NodeId) {
        self.records
            .lock()
            .unwrap()
            .myModifiedItems
            .push(ItemId::from(theNode));
    }
    fn on_ref_modified(&self, _attached: AttachedGraph<'_>, theRef: RefId) {
        self.records
            .lock()
            .unwrap()
            .myModifiedItems
            .push(ItemId::from(theRef));
    }
    fn copy_to(&self, _copy: &CopyRemap<'_>) -> Result<(), Failure> {
        Ok(())
    }
    fn invalidate_all(&self, _attached: AttachedGraph<'_>) {}
    fn clear(&self, _attached: AttachedGraph<'_>) {
        *self.records.lock().unwrap() = DispatchRecords::default();
    }
}

#[test]
fn BRepGraph_LockTest__NewItemsAreUnlockedByDefault() {
    let aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aVertexId = VertexId::start();
    let aVertexRefId = start_vertex_ref(&aGraph);

    assert!(!isOwned(&aGraph, node(aVertexId)));
    assert!(!isOwned(&aGraph, reference(aVertexRefId)));
}

#[test]
fn BRepGraph_LockTest__LockLayerControlsStorageFlag() {
    let aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aVertexId = VertexId::start();
    assert!(!isOwned(&aGraph, node(aVertexId)));

    lockItem(&aGraph, node(aVertexId));
    assert!(isOwned(&aGraph, node(aVertexId)));

    unlockItem(&aGraph, aVertexId);
    assert!(!isOwned(&aGraph, node(aVertexId)));
}

#[test]
fn BRepGraph_LockTest__LockedNodeRejectsMutableGuard() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aVertexId = VertexId::start();
    lockItem(&aGraph, node(aVertexId));

    expect_program_error(aGraph.editor().vertices().mutate(aVertexId).map(drop));
}

#[test]
fn BRepGraph_LockTest__LockedReferenceRejectsMutableGuard() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aVertexRefId = start_vertex_ref(&aGraph);
    lockItem(&aGraph, reference(aVertexRefId));

    expect_program_error(
        aGraph
            .editor()
            .vertices()
            .mutate_ref(aVertexRefId)
            .map(drop),
    );
}

#[test]
fn BRepGraph_LockTest__LockedNodeRejectsRemoval() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aVertexId = VertexId::start();
    lockItem(&aGraph, node(aVertexId));

    expect_program_error(aGraph.editor().generic().remove_node(aVertexId.into()));
}

#[test]
fn BRepGraph_LockTest__LockedParentRejectsStructuralAdd() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aShellId = ShellId::start();
    lockItem(&aGraph, node(aShellId));

    let aFaceId = FaceId::start();
    expect_program_error(aGraph.editor().shells().append(
        aShellId,
        aFaceId,
        ParityOrientation::FORWARD,
    ));
}

#[test]
fn BRepGraph_LockTest__LockedNodeRejectsDirectSetterWithoutMutation() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aVertexId = VertexId::start();
    let aPoint = aGraph.topo().vertices().definition(aVertexId).point;
    lockItem(&aGraph, node(aVertexId));

    expect_program_error(
        aGraph
            .editor()
            .vertices()
            .set_point(aVertexId, Point3::new(1.0, 2.0, 3.0)),
    );
    let aDistance = aGraph
        .topo()
        .vertices()
        .definition(aVertexId)
        .point
        .distance(&aPoint);
    assert!((aDistance - 0.0).abs() <= 1.0e-12);
}

#[test]
fn BRepGraph_LockTest__LockedReferenceRejectsOrientationSetterWithoutMutation() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aVertexRefId = start_vertex_ref(&aGraph);
    let anOrientation = aGraph.refs().vertices().entry(aVertexRefId).orientation;
    lockItem(&aGraph, reference(aVertexRefId));

    expect_program_error(
        aGraph
            .editor()
            .vertices()
            .set_ref_orientation(aVertexRefId, ParityOrientation::REVERSED),
    );
    assert_eq!(
        aGraph.refs().vertices().entry(aVertexRefId).orientation,
        anOrientation
    );
}

#[test]
fn BRepGraph_LockTest__LockedRepresentationRejectsDirectSetterWithoutMutation() {
    let aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aSurface = aGraph.topo().faces().surface(FaceId::start());
    assert!(!aSurface.is_null());

    assert_eq!(
        aGraph
            .topo()
            .faces()
            .surface(FaceId::start())
            .object_address(),
        aSurface.object_address()
    );
}

#[test]
fn BRepGraph_LockTest__LockLayerOwnerControlsLockFlag() {
    let aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aVertexId = VertexId::start();
    let aLayer = lock_layer(&aGraph);
    let anOwnerId = guid("2ee13474-4cc6-4f6f-bde2-f0e4dcf9e5f1");

    aLayer
        .set_owner(&aGraph, node(aVertexId), &anOwnerId)
        .unwrap();

    assert!(isOwned(&aGraph, node(aVertexId)));
    assert!(aLayer.has_owner(&aGraph, node(aVertexId)));
    let aFoundOwnerId = aLayer.find_owner_id(&aGraph, node(aVertexId)).unwrap();
    assert!(aFoundOwnerId.is_some());
    assert_eq!(aFoundOwnerId.unwrap(), anOwnerId);

    aLayer.unset_owner(&aGraph, node(aVertexId)).unwrap();
    assert!(!isOwned(&aGraph, node(aVertexId)));
    assert!(!aLayer.has_owner(&aGraph, node(aVertexId)));
}

#[test]
fn BRepGraph_LockTest__RejectsDifferentOwnerWhenAncestorOwnsNode() {
    let aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aSolidId = SolidId::start();
    let aFaceId = FaceId::start();
    let aLayer = lock_layer(&aGraph);

    assert!(
        aLayer
            .set_owner_with_revision(&aGraph, node(aSolidId), &testOwnerId(), false)
            .unwrap()
    );
    assert!(aLayer.has_owner(&aGraph, node(aFaceId)));
    assert!(
        !aLayer
            .set_owner_with_revision(&aGraph, node(aFaceId), &otherOwnerId(), false)
            .unwrap()
    );

    let aFoundOwnerId = aLayer.find_owner_id(&aGraph, node(aFaceId)).unwrap();
    assert!(aFoundOwnerId.is_some());
    assert_eq!(aFoundOwnerId.unwrap(), testOwnerId());
}

#[test]
fn BRepGraph_LockTest__ParentOwnerRejectsDifferentOwnedDescendant() {
    let aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aSolidId = SolidId::start();
    let aFaceId = FaceId::start();
    let aLayer = lock_layer(&aGraph);

    assert!(
        aLayer
            .set_owner_with_revision(&aGraph, node(aFaceId), &testOwnerId(), false)
            .unwrap()
    );
    assert!(
        !aLayer
            .set_owner_with_revision(&aGraph, node(aSolidId), &otherOwnerId(), false)
            .unwrap()
    );
    assert!(aLayer.has_owner(&aGraph, node(aFaceId)));
    assert!(!aLayer.has_owner(&aGraph, node(aSolidId)));
}

#[test]
fn BRepGraph_LockTest__ParentOwnerCollapsesSameOwnedDescendant() {
    let aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aSolidId = SolidId::start();
    let aFaceId = FaceId::start();
    let aLayer = lock_layer(&aGraph);

    assert!(
        aLayer
            .set_owner_with_revision(&aGraph, node(aFaceId), &testOwnerId(), false)
            .unwrap()
    );
    assert!(
        aLayer
            .set_owner_with_revision(&aGraph, node(aSolidId), &testOwnerId(), false)
            .unwrap()
    );
    assert!(aLayer.has_owner(&aGraph, node(aFaceId)));
    assert!(aLayer.has_owner(&aGraph, node(aSolidId)));

    aLayer.unset_owner(&aGraph, node(aSolidId)).unwrap();
    assert!(!aLayer.has_owner(&aGraph, node(aSolidId)));
    assert!(!aLayer.has_owner(&aGraph, node(aFaceId)));
}

#[test]
fn BRepGraph_LockTest__RemovedRootOwnerCallbackClearsDescendantOwnedFlags() {
    let aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aSolidId = SolidId::start();
    let aFaceId = FaceId::start();
    let aFaceRel = aGraph.topo().faces().relations(aFaceId);
    assert!(!aFaceRel.wire_ref_ids.is_empty());
    let aWireRefId = aFaceRel.wire_ref_ids[0];

    let aLayer = lock_layer(&aGraph);
    assert!(
        aLayer
            .set_owner_with_revision(&aGraph, node(aSolidId), &testOwnerId(), false)
            .unwrap()
    );
    assert!(aLayer.has_owner(&aGraph, node(aSolidId)));
    assert!(aLayer.has_owner(&aGraph, node(aFaceId)));
    assert!(aLayer.has_owner(&aGraph, reference(aWireRefId)));

    aGraph
        .layer_registry()
        .dispatch_on_node_removed(Some(&aGraph), aSolidId.into());

    assert!(!aLayer.has_owner(&aGraph, node(aSolidId)));
    assert!(!aLayer.has_owner(&aGraph, node(aFaceId)));
    assert!(!aLayer.has_owner(&aGraph, reference(aWireRefId)));
}

fn deferred_layer(theGraph: &BRepGraph) -> Handle<LayerDeferred> {
    theGraph.layer_registry().ensure::<LayerDeferred>().unwrap()
}

fn register(
    theGraph: &BRepGraph,
    theLayer: &LayerDeferred,
    theItem: ItemId,
    theKey: &str,
    theKind: RepresentationKind,
    theName: &str,
    theIndex: u32,
) {
    theLayer
        .register_deferred(
            theGraph,
            theItem,
            &AsciiString::new("TestProvider"),
            &AsciiString::new(theKey),
            theKind,
            &AsciiString::new(theName),
            theIndex,
        )
        .unwrap();
}

#[test]
fn BRepGraph_LockTest__DeferredLayerRegistersMultipleRepresentationsAndLocksItem() {
    let aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aFaceId = FaceId::start();
    let aLayer = deferred_layer(&aGraph);

    let aFace = node(aFaceId);
    register(
        &aGraph,
        &aLayer,
        aFace,
        "test-source",
        RepresentationKind::Geometry,
        "surface",
        42,
    );
    register(
        &aGraph,
        &aLayer,
        aFace,
        "test-source",
        RepresentationKind::Mesh,
        "triangulation",
        7,
    );

    assert!(isOwned(&aGraph, aFace));
    let anEntry = aLayer.find_deferred(aFace);
    assert!(anEntry.is_some());
    let anEntry = anEntry.unwrap();
    assert_eq!(anEntry.provider, AsciiString::new("TestProvider"));
    assert_eq!(anEntry.source_key, AsciiString::new("test-source"));
    assert_eq!(anEntry.representations.size(), 2);

    aLayer.unregister_deferred(&aGraph, aFace);
    assert!(!isOwned(&aGraph, aFace));
    assert!(!aLayer.has_deferred(aFace));
}

#[test]
fn BRepGraph_LockTest__DeferredLayerClearsOwnerOnNodeRemoved() {
    let aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aFaceId = FaceId::start();
    let aLayer = deferred_layer(&aGraph);

    let aFace = node(aFaceId);
    register(
        &aGraph,
        &aLayer,
        aFace,
        "test-source",
        RepresentationKind::Geometry,
        "surface",
        42,
    );
    assert!(isOwned(&aGraph, aFace));
    assert!(aLayer.has_deferred(aFace));

    aGraph
        .layer_registry()
        .dispatch_on_node_removed(Some(&aGraph), aFaceId.into());

    assert!(!isOwned(&aGraph, aFace));
    assert!(!aLayer.has_deferred(aFace));
}

#[test]
fn BRepGraph_LockTest__ItemDispatchDelegatesToTypedCallbacks() {
    let aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aLayer = Handle::new(BRepGraph_LayerItemDispatchProbe::new());
    aGraph
        .layer_registry()
        .register_layer(&as_layer(&aLayer))
        .unwrap();

    let aFaceId = FaceId::start();
    let aVertexRefId = start_vertex_ref(&aGraph);

    let aRegistry = aGraph.layer_registry();
    aRegistry.dispatch_on_item_removed(Some(&aGraph), node(aFaceId));
    aRegistry.dispatch_on_item_removed(Some(&aGraph), reference(aVertexRefId));

    let aRemoved = aLayer.RemovedItems();
    assert_eq!(aRemoved.len(), 2);
    assert!(aRemoved[0].is_node());
    assert_eq!(aRemoved[0].node_id(), NodeId::from(aFaceId));
    assert!(aRemoved[1].is_reference());
    assert_eq!(aRemoved[1].ref_id(), RefId::from(aVertexRefId));

    aRegistry.dispatch_item_modified(Some(&aGraph), node(aFaceId));
    aRegistry.dispatch_item_modified(Some(&aGraph), reference(aVertexRefId));

    let aModified = aLayer.ModifiedItems();
    assert_eq!(aModified.len(), 2);
    assert!(aModified[0].is_node());
    assert_eq!(aModified[0].node_id(), NodeId::from(aFaceId));
    assert!(aModified[1].is_reference());
    assert_eq!(aModified[1].ref_id(), RefId::from(aVertexRefId));
}

#[test]
fn BRepGraph_LockTest__Compact_PreservesDeferredRefEntry() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aRemovedFace = FaceId::start();
    let aKeptFace = FaceId::new(1);
    let aFaceRelations = aGraph.topo().faces().relations(aKeptFace);
    assert!(!aFaceRelations.wire_ref_ids.is_empty());
    let aWireRefId = aFaceRelations.wire_ref_ids[0];

    let mut aLayer = deferred_layer(&aGraph);
    let aWireRef = reference(aWireRefId);
    register(
        &aGraph,
        &aLayer,
        aWireRef,
        "ref-source",
        RepresentationKind::Topology,
        "wire-ref",
        11,
    );
    assert!(aLayer.has_deferred(aWireRef));
    assert!(isOwned(&aGraph, aWireRef));

    aGraph
        .editor()
        .generic()
        .remove_node(aRemovedFace.into())
        .unwrap();
    let _ = compact::perform(&mut aGraph).unwrap();

    // Re-acquire layer after compact (CopyLayersTo replaces old instance).
    aLayer = deferred_layer(&aGraph);

    assert!(aGraph.topo().faces().nb() >= 1);
    let aCompactedFaceRelations = aGraph.topo().faces().relations(FaceId::start());
    assert!(!aCompactedFaceRelations.wire_ref_ids.is_empty());
    let aCompactedWireRefId = aCompactedFaceRelations.wire_ref_ids[0];

    assert!(aLayer.has_deferred(reference(aCompactedWireRefId)));
    assert!(isOwned(&aGraph, reference(aCompactedWireRefId)));
}

#[test]
fn BRepGraph_LockTest__Compact_PreservesLockOnExactGeometryRep() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aRemovedFace = FaceId::start();
    let aKeptFace = FaceId::new(1);
    let aSurfaceRepId = aGraph.topo().faces().definition(aKeptFace).surface_rep_id;
    assert!(aSurfaceRepId.is_valid());

    let mut aLayer = lock_layer(&aGraph);
    aLayer
        .set_owner(&aGraph, node(aKeptFace), &testOwnerId())
        .unwrap();
    assert!(aLayer.has_owner(&aGraph, node(aKeptFace)));

    aGraph
        .editor()
        .generic()
        .remove_node(aRemovedFace.into())
        .unwrap();
    let _ = compact::perform(&mut aGraph).unwrap();

    // Re-acquire layer after compact (CopyLayersTo replaces old instance).
    aLayer = lock_layer(&aGraph);

    let aCompactedSurfaceRepId = aGraph
        .topo()
        .faces()
        .definition(FaceId::start())
        .surface_rep_id;
    assert!(aCompactedSurfaceRepId.is_valid());
    assert!(aLayer.has_owner(&aGraph, node(FaceId::start())));
}
