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

//! Direct translation of BRepGraph_EventBus_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported); the suite includes BRepGraph_Compact
//! but calls none. The test layers keep their records behind a lock, as a Rust layer
//! is shared through a handle; their no-op `CopyTo` overrides are no-op `copy_to`s.
#![allow(non_snake_case, non_camel_case_types)]
mod support;
use rustycad_tkbrep::brep_graph::LayerIterator;
use rustycad_tkbrep::brep_graph::copy_remap::CopyRemap;
use rustycad_tkbrep::brep_graph::deferred_scope::DeferredScope;
use rustycad_tkbrep::brep_graph::editor::EdgeOps;
use rustycad_tkbrep::brep_graph::layer::{
    AttachedGraph, BRepGraphLayer, LayerBase, as_layer, kind_bit, ref_kind_bit,
};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Guid, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::{Mutex, MutexGuard};
use support::make_box;

fn guid(text: &str) -> Guid {
    Guid::from_ascii(text.as_bytes()).unwrap()
}

/// The recorded events of `BRepGraph_LayerModTracking`.
#[derive(Default)]
struct Tracking {
    myImmediateEvents: Vec<NodeId>,
    myBatchEvents: Vec<NodeId>,
    myBatchCallCount: i32,
    myRemoveCallCount: i32,
    myReplaceCallCount: i32,
    myLastRemovedNode: NodeId,
    myLastReplacement: NodeId,
    myRefRemovedEvents: Vec<RefId>,
    myRefImmediateEvents: Vec<RefId>,
    myRefBatchEvents: Vec<RefId>,
    myRefBatchCallCount: i32,
    myRefRemoveCallCount: i32,
}

// Test layer that records modification events.
struct BRepGraph_LayerModTracking {
    transient: StandardTransient,
    base: LayerBase,
    myName: AsciiString,
    mySubscribedKinds: i32,
    mySubscribedRefKinds: i32,
    myId: Guid,
    state: Mutex<Tracking>,
}
impl_transient!(BRepGraph_LayerModTracking, "BRepGraph_LayerModTracking", dyn BRepGraphLayer,
    transient, interfaces: [dyn BRepGraphLayer]);

impl BRepGraph_LayerModTracking {
    fn new(theName: &str, theSubscribedKinds: i32) -> Self {
        Self::with_id(
            theName,
            theSubscribedKinds,
            "2f9b6a5c-1f2d-4a88-9c1c-7a0c16a10004",
            0,
        )
    }

    fn with_id(
        theName: &str,
        theSubscribedKinds: i32,
        theId: &str,
        theSubscribedRefKinds: i32,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: LayerBase::new(),
            myName: AsciiString::new(theName),
            mySubscribedKinds: theSubscribedKinds,
            mySubscribedRefKinds: theSubscribedRefKinds,
            myId: guid(theId),
            state: Mutex::new(Tracking::default()),
        }
    }

    fn s(&self) -> MutexGuard<'_, Tracking> {
        self.state.lock().unwrap()
    }

    fn HasImmediateEventFor(&self, theNode: impl Into<NodeId>) -> bool {
        self.s().myImmediateEvents.contains(&theNode.into())
    }

    fn HasBatchEventFor(&self, theNode: impl Into<NodeId>) -> bool {
        self.s().myBatchEvents.contains(&theNode.into())
    }

    fn CountImmediateEventsOfKind(&self, theKind: NodeKind) -> i32 {
        self.s()
            .myImmediateEvents
            .iter()
            .filter(|anEvent| anEvent.kind == theKind)
            .count() as i32
    }
}

impl BRepGraphLayer for BRepGraph_LayerModTracking {
    fn layer_base(&self) -> &LayerBase {
        &self.base
    }
    fn id(&self) -> Guid {
        self.myId
    }
    fn name(&self) -> &AsciiString {
        &self.myName
    }
    fn subscribed_kinds(&self) -> i32 {
        self.mySubscribedKinds
    }
    fn subscribed_ref_kinds(&self) -> i32 {
        self.mySubscribedRefKinds
    }
    fn copy_to(&self, _copy: &CopyRemap<'_>) -> Result<(), Failure> {
        Ok(())
    }
    fn on_ref_removed(&self, _attached: AttachedGraph<'_>, theRef: RefId) {
        let mut s = self.s();
        s.myRefRemovedEvents.push(theRef);
        s.myRefRemoveCallCount += 1;
    }
    fn on_ref_modified(&self, _attached: AttachedGraph<'_>, theRef: RefId) {
        self.s().myRefImmediateEvents.push(theRef);
    }
    fn on_refs_modified(&self, _attached: AttachedGraph<'_>, theRefs: &[RefId]) {
        let mut s = self.s();
        s.myRefBatchEvents = theRefs.to_vec();
        s.myRefBatchCallCount += 1;
    }
    fn on_node_modified(&self, _attached: AttachedGraph<'_>, theNode: NodeId) {
        self.s().myImmediateEvents.push(theNode);
    }
    fn on_nodes_modified(&self, _attached: AttachedGraph<'_>, theNodes: &[NodeId]) {
        let mut s = self.s();
        s.myBatchEvents = theNodes.to_vec();
        s.myBatchCallCount += 1;
    }
    fn on_node_removed(&self, _attached: AttachedGraph<'_>, theNode: NodeId) {
        let mut s = self.s();
        s.myLastRemovedNode = theNode;
        s.myRemoveCallCount += 1;
    }
    fn on_node_replaced(
        &self,
        _attached: AttachedGraph<'_>,
        theOldNode: NodeId,
        theNewNode: NodeId,
    ) {
        let mut s = self.s();
        s.myLastRemovedNode = theOldNode;
        s.myLastReplacement = theNewNode;
        s.myReplaceCallCount += 1;
    }
    fn invalidate_all(&self, _attached: AttachedGraph<'_>) {}
    fn clear(&self, _attached: AttachedGraph<'_>) {
        *self.s() = Tracking::default();
    }
}

// Minimal layer with default SubscribedKinds() behavior from base class.
struct BRepGraph_LayerDefault {
    transient: StandardTransient,
    base: LayerBase,
    myId: Guid,
    myName: AsciiString,
}
impl_transient!(BRepGraph_LayerDefault, "BRepGraph_LayerDefault", dyn BRepGraphLayer,
    transient, interfaces: [dyn BRepGraphLayer]);

impl BRepGraph_LayerDefault {
    fn new() -> Self {
        Self::named("2f9b6a5c-1f2d-4a88-9c1c-7a0c16a10007", "DefaultLayer")
    }

    fn named(theId: &str, theName: &str) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: LayerBase::new(),
            myId: guid(theId),
            myName: AsciiString::new(theName),
        }
    }
}

impl BRepGraphLayer for BRepGraph_LayerDefault {
    fn layer_base(&self) -> &LayerBase {
        &self.base
    }
    fn id(&self) -> Guid {
        self.myId
    }
    fn name(&self) -> &AsciiString {
        &self.myName
    }
    fn on_node_removed(&self, _attached: AttachedGraph<'_>, _node: NodeId) {}
    fn copy_to(&self, _copy: &CopyRemap<'_>) -> Result<(), Failure> {
        Ok(())
    }
    fn invalidate_all(&self, _attached: AttachedGraph<'_>) {}
    fn clear(&self, _attached: AttachedGraph<'_>) {}
}

/// `BRepGraph_LayerGraphProbe`: the default layer under another GUID and name, with the
/// protected `IsAttached` and `Graph` exposed.
fn BRepGraph_LayerGraphProbe() -> BRepGraph_LayerDefault {
    BRepGraph_LayerDefault::named("2f9b6a5c-1f2d-4a88-9c1c-7a0c16a10008", "GraphProbeLayer")
}

impl BRepGraph_LayerDefault {
    fn IsGraphAttached(&self) -> bool {
        self.base.is_attached()
    }

    fn NbFacesFromGraph(&self, theGraph: &BRepGraph) -> Result<u32, Failure> {
        Ok(self.base.graph(theGraph)?.topo().faces().nb())
    }
}

/// The `BRepGraph_EventBusTest` fixture: a graph of a 10x20x30 box.
fn SetUp() -> BRepGraph {
    let mut myGraph = BRepGraph::new();
    myGraph.clear().unwrap();
    let _aBuildRes1 = myGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    assert!(!myGraph.is_empty());
    myGraph
}

fn register<T: BRepGraphLayer + rustycad_tkernel::standard::RuntimeTyped>(
    theGraph: &BRepGraph,
    theLayer: &Handle<T>,
) {
    theGraph
        .layer_registry()
        .register_layer(&as_layer(theLayer))
        .unwrap();
}

/// `{ auto aMut = Edges().Mut(edge); Edges().SetTolerance(aMut, tolerance); }`.
fn setToleranceGuarded(theGraph: &mut BRepGraph, theEdge: EdgeId, theTolerance: f64) {
    let mut aMut = theGraph.editor().edges().mutate(theEdge).unwrap();
    EdgeOps::set_tolerance_guarded(&mut aMut, theTolerance);
}

fn setTolerance(theGraph: &mut BRepGraph, theEdge: EdgeId, theTolerance: f64) {
    theGraph
        .editor()
        .edges()
        .set_tolerance(theEdge, theTolerance)
        .unwrap();
}

fn tracker(theName: &str, theKinds: i32) -> Handle<BRepGraph_LayerModTracking> {
    Handle::new(BRepGraph_LayerModTracking::new(theName, theKinds))
}

fn allFiveKinds() -> i32 {
    kind_bit(NodeKind::EDGE)
        | kind_bit(NodeKind::WIRE)
        | kind_bit(NodeKind::FACE)
        | kind_bit(NodeKind::SHELL)
        | kind_bit(NodeKind::SOLID)
}

#[test]
fn BRepGraph_EventBusTest__ZeroCost_NoSubscribers() {
    let mut myGraph = SetUp();
    // Mutate edge without any subscribing layer - verify no crash.
    setToleranceGuarded(&mut myGraph, EdgeId::start(), 0.5);
    assert!(
        myGraph
            .topo()
            .edges()
            .definition(EdgeId::start())
            .base
            .own_gen
            > 0
    );
}

#[test]
fn BRepGraph_EventBusTest__ImmediateMode_SingleEdge() {
    let mut myGraph = SetUp();
    let aLayer = tracker("Tracker", allFiveKinds());
    register(&myGraph, &aLayer);

    setToleranceGuarded(&mut myGraph, EdgeId::start(), 0.5);

    // Edge(0) should have an immediate event.
    assert!(aLayer.HasImmediateEventFor(EdgeId::start()));
    // At least one event total (edge + propagated parents).
    assert!(!aLayer.s().myImmediateEvents.is_empty());
}

#[test]
fn BRepGraph_EventBusTest__ImmediateMode_UpwardPropagation() {
    let mut myGraph = SetUp();
    let aLayer = tracker("Tracker", allFiveKinds());
    register(&myGraph, &aLayer);

    setToleranceGuarded(&mut myGraph, EdgeId::start(), 0.5);

    // Only directly mutated node gets immediate dispatch.
    // Parents get SubtreeGen incremented but NO dispatch (mutex-free propagation).
    assert!(aLayer.CountImmediateEventsOfKind(NodeKind::EDGE) > 0);
    assert_eq!(aLayer.CountImmediateEventsOfKind(NodeKind::WIRE), 0);
    assert_eq!(aLayer.CountImmediateEventsOfKind(NodeKind::FACE), 0);
    assert_eq!(aLayer.CountImmediateEventsOfKind(NodeKind::SHELL), 0);
    assert_eq!(aLayer.CountImmediateEventsOfKind(NodeKind::SOLID), 0);
    // Verify SubtreeGen was propagated upward despite no dispatch.
    assert!(
        myGraph
            .topo()
            .wires()
            .definition(WireId::start())
            .base
            .subtree_gen
            > 0
    );
    assert!(
        myGraph
            .topo()
            .faces()
            .definition(FaceId::start())
            .base
            .subtree_gen
            > 0
    );
}

#[test]
fn BRepGraph_EventBusTest__ImmediateMode_KindFilter() {
    let mut myGraph = SetUp();
    // Subscribe only to Face.
    let aLayer = tracker("FaceOnly", kind_bit(NodeKind::FACE));
    register(&myGraph, &aLayer);

    setToleranceGuarded(&mut myGraph, EdgeId::start(), 0.5);

    // No face dispatch from upward propagation (mutex-free SubtreeGen only).
    // Edge is directly mutated but Face-only subscription filters it out.
    assert_eq!(aLayer.CountImmediateEventsOfKind(NodeKind::EDGE), 0);
    assert_eq!(aLayer.CountImmediateEventsOfKind(NodeKind::FACE), 0);
    // But SubtreeGen was propagated.
    assert!(
        myGraph
            .topo()
            .faces()
            .definition(FaceId::start())
            .base
            .subtree_gen
            > 0
    );
}

#[test]
fn BRepGraph_EventBusTest__DeferredMode_BatchDispatch() {
    let mut myGraph = SetUp();
    let aLayer = tracker("Tracker", allFiveKinds());
    register(&myGraph, &aLayer);

    myGraph.editor().begin_deferred_invalidation();
    setTolerance(&mut myGraph, EdgeId::start(), 0.5);
    setTolerance(&mut myGraph, EdgeId::new(1), 0.6);
    setTolerance(&mut myGraph, EdgeId::new(2), 0.7);
    myGraph.editor().end_deferred_invalidation();

    // OnNodesModified called exactly once.
    assert_eq!(aLayer.s().myBatchCallCount, 1);
    // Batch contains at least the 3 edges + propagated parents.
    assert!(aLayer.s().myBatchEvents.len() >= 3);
    assert!(aLayer.HasBatchEventFor(EdgeId::start()));
    assert!(aLayer.HasBatchEventFor(EdgeId::new(1)));
    assert!(aLayer.HasBatchEventFor(EdgeId::new(2)));
}

#[test]
fn BRepGraph_EventBusTest__DeferredMode_NoImmediateDispatch() {
    let mut myGraph = SetUp();
    let aLayer = tracker("Tracker", kind_bit(NodeKind::EDGE));
    register(&myGraph, &aLayer);

    myGraph.editor().begin_deferred_invalidation();
    setTolerance(&mut myGraph, EdgeId::start(), 0.5);

    // During deferred mode: OnNodeModified must NOT be called.
    assert_eq!(aLayer.s().myImmediateEvents.len(), 0);

    myGraph.editor().end_deferred_invalidation();

    // Immediate events still empty - only batch was dispatched.
    assert_eq!(aLayer.s().myImmediateEvents.len(), 0);
    assert_eq!(aLayer.s().myBatchCallCount, 1);
}

#[test]
fn BRepGraph_EventBusTest__DeferredMode_RefOnlyMutation_DispatchesRefBatch() {
    let mut myGraph = SetUp();
    let aVertexRefBit = ref_kind_bit(RefKind::VERTEX);
    let aLayer = Handle::new(BRepGraph_LayerModTracking::with_id(
        "RefTracker",
        0,
        "2f9b6a5c-1f2d-4a88-9c1c-7a0c16a10008",
        aVertexRefBit,
    ));
    register(&myGraph, &aLayer);

    let aVertexRef = myGraph
        .topo()
        .edges()
        .definition(EdgeId::start())
        .start_vertex_ref_id;
    assert!(aVertexRef.is_valid());

    myGraph.editor().begin_deferred_invalidation();
    myGraph
        .editor()
        .vertices()
        .set_ref_orientation(aVertexRef, ParityOrientation::REVERSED)
        .unwrap();
    myGraph.editor().end_deferred_invalidation();

    assert_eq!(aLayer.s().myBatchCallCount, 0);
    assert_eq!(aLayer.s().myRefBatchCallCount, 1);
    assert_eq!(aLayer.s().myRefBatchEvents.len(), 1);
    assert_eq!(aLayer.s().myRefBatchEvents[0], RefId::from(aVertexRef));
}

#[test]
fn BRepGraph_EventBusTest__UnregisterLayer_FlagUpdate() {
    let mut myGraph = SetUp();
    let aLayer = tracker("Tracker", kind_bit(NodeKind::EDGE));
    register(&myGraph, &aLayer);

    // Mutate - should dispatch.
    setToleranceGuarded(&mut myGraph, EdgeId::start(), 0.5);
    assert!(!aLayer.s().myImmediateEvents.is_empty());

    // Unregister and clear.
    myGraph.layer_registry().unregister_layer(&aLayer.id());
    aLayer.clear(None);

    // Mutate again - should NOT dispatch (layer unregistered).
    setToleranceGuarded(&mut myGraph, EdgeId::new(1), 0.6);
    assert_eq!(aLayer.s().myImmediateEvents.len(), 0);
}

#[test]
fn BRepGraph_EventBusTest__DetachedLayerGraph_FailsClearly() {
    let myGraph = SetUp();
    let aLayer = Handle::new(BRepGraph_LayerGraphProbe());
    register(&myGraph, &aLayer);

    assert!(aLayer.IsGraphAttached());
    assert_eq!(
        aLayer.NbFacesFromGraph(&myGraph).unwrap(),
        myGraph.topo().faces().nb()
    );

    myGraph.layer_registry().unregister_layer(&aLayer.id());

    assert!(!aLayer.IsGraphAttached());
    assert_eq!(
        aLayer
            .NbFacesFromGraph(&myGraph)
            .unwrap_err()
            .exception_type(),
        "Standard_ProgramError"
    );
}

#[test]
fn BRepGraph_EventBusTest__FindLayer_ByGuid_ReturnsRegisteredLayer() {
    let myGraph = SetUp();
    let aLayer = tracker("Tracker", kind_bit(NodeKind::EDGE));

    register(&myGraph, &aLayer);

    assert!(myGraph.layer_registry().find_layer(&aLayer.id()) == aLayer);
    assert!(myGraph.layer_registry().find_slot(&aLayer.id()).is_some());
}

#[test]
fn BRepGraph_EventBusTest__OnNodeReplaced_DispatchesReplacement() {
    let mut myGraph = SetUp();
    let aLayer = tracker("Tracker", 0);
    register(&myGraph, &aLayer);

    let anOldEdge = NodeId::from(EdgeId::start());
    let aNewEdge = NodeId::from(EdgeId::new(1));
    myGraph
        .editor()
        .generic()
        .replace_node(anOldEdge, aNewEdge)
        .unwrap();

    let s = aLayer.s();
    assert_eq!(s.myRemoveCallCount, 0);
    assert_eq!(s.myReplaceCallCount, 1);
    assert_eq!(s.myLastRemovedNode, anOldEdge);
    assert_eq!(s.myLastReplacement, aNewEdge);
}

#[test]
fn BRepGraph_EventBusTest__OnNodeRemoved_DispatchesPureDeletion() {
    let mut myGraph = SetUp();
    let aLayer = tracker("Tracker", 0);
    register(&myGraph, &aLayer);

    let aFaceId = NodeId::from(FaceId::start());
    myGraph.editor().generic().remove_node(aFaceId).unwrap();

    let s = aLayer.s();
    assert_eq!(s.myRemoveCallCount, 1);
    assert_eq!(s.myReplaceCallCount, 0);
    assert_eq!(s.myLastRemovedNode, aFaceId);
    assert!(!s.myLastReplacement.is_valid());
}

#[test]
fn BRepGraph_EventBusTest__MultipleSubscribers() {
    let mut myGraph = SetUp();
    let aEdgeLayer = Handle::new(BRepGraph_LayerModTracking::with_id(
        "EdgeTracker",
        kind_bit(NodeKind::EDGE),
        "2f9b6a5c-1f2d-4a88-9c1c-7a0c16a10005",
        0,
    ));
    let aFaceLayer = Handle::new(BRepGraph_LayerModTracking::with_id(
        "FaceTracker",
        kind_bit(NodeKind::FACE),
        "2f9b6a5c-1f2d-4a88-9c1c-7a0c16a10006",
        0,
    ));
    register(&myGraph, &aEdgeLayer);
    register(&myGraph, &aFaceLayer);

    setToleranceGuarded(&mut myGraph, EdgeId::start(), 0.5);

    // Edge layer gets edge events (directly mutated), no face events.
    assert!(aEdgeLayer.CountImmediateEventsOfKind(NodeKind::EDGE) > 0);
    assert_eq!(aEdgeLayer.CountImmediateEventsOfKind(NodeKind::FACE), 0);

    // Face layer gets NO events - parents don't get immediate dispatch.
    assert_eq!(aFaceLayer.CountImmediateEventsOfKind(NodeKind::EDGE), 0);
    assert_eq!(aFaceLayer.CountImmediateEventsOfKind(NodeKind::FACE), 0);
}

#[test]
fn BRepGraph_EventBusTest__DefaultSubscribedKinds_Zero() {
    let mut myGraph = SetUp();
    let aDefaultLayer = Handle::new(BRepGraph_LayerDefault::new());
    register(&myGraph, &aDefaultLayer);

    // SubscribedKinds() == 0 in BRepGraph_Layer base default implementation.
    assert_eq!(aDefaultLayer.subscribed_kinds(), 0);

    // Mutate - a layer with default SubscribedKinds() should not receive modification events.
    // This just verifies the default no-subscription path remains a no-op.
    setToleranceGuarded(&mut myGraph, EdgeId::start(), 0.5);
    assert!(
        myGraph
            .topo()
            .edges()
            .definition(EdgeId::start())
            .base
            .own_gen
            > 0
    );
}

#[test]
fn BRepGraph_EventBusTest__DeferredScope_DispatchesOnDestruction() {
    let mut myGraph = SetUp();
    let aLayer = tracker("Tracker", kind_bit(NodeKind::EDGE));
    register(&myGraph, &aLayer);

    {
        let mut aScope = DeferredScope::new(&mut myGraph);
        setTolerance(&mut aScope, EdgeId::start(), 0.5);

        // During guard scope: no batch dispatch yet.
        assert_eq!(aLayer.s().myBatchCallCount, 0);
    }

    // After guard destruction: batch dispatched.
    assert_eq!(aLayer.s().myBatchCallCount, 1);
    assert!(aLayer.HasBatchEventFor(EdgeId::start()));
}

#[test]
fn BRepGraph_EventBusTest__DeferredMode_NoModifications_NoDispatch() {
    let mut myGraph = SetUp();
    let aLayer = tracker("Tracker", kind_bit(NodeKind::EDGE));
    register(&myGraph, &aLayer);

    myGraph.editor().begin_deferred_invalidation();
    // No mutations.
    myGraph.editor().end_deferred_invalidation();

    assert_eq!(aLayer.s().myBatchCallCount, 0);
    assert_eq!(aLayer.s().myBatchEvents.len(), 0);
}

fn bitCount(theValue: i32) -> i32 {
    let mut aBitCount = 0;
    let mut v = theValue;
    while v != 0 {
        aBitCount += v & 1;
        v >>= 1;
    }
    aBitCount
}

#[test]
fn BRepGraph_EventBusTest__KindBit_Helpers() {
    let _myGraph = SetUp();
    // Each kind produces a distinct single-bit value derived from the Kind enum.
    let kinds = [
        NodeKind::SOLID,
        NodeKind::SHELL,
        NodeKind::FACE,
        NodeKind::WIRE,
        NodeKind::EDGE,
        NodeKind::VERTEX,
        NodeKind::COMPOUND,
        NodeKind::COMP_SOLID,
    ];
    for aKind in kinds {
        assert_eq!(kind_bit(aKind), 1 << aKind.0);
    }

    // All kind bits are distinct (no collisions).
    let aAll = kinds
        .iter()
        .fold(0, |anAll, aKind| anAll | kind_bit(*aKind));
    // 8 distinct bits set.
    assert_eq!(bitCount(aAll), 8);
}

#[test]
fn BRepGraph_EventBusTest__RefKindBit_Helpers() {
    let _myGraph = SetUp();
    let kinds = [
        RefKind::SHELL,
        RefKind::FACE,
        RefKind::WIRE,
        RefKind::VERTEX,
        RefKind::SOLID,
        RefKind::CHILD,
        RefKind::OCCURRENCE,
    ];
    for aKind in kinds {
        assert_eq!(ref_kind_bit(aKind), 1 << aKind.0);
    }

    // 7 distinct bits.
    let aAll = kinds
        .iter()
        .fold(0, |anAll, aKind| anAll | ref_kind_bit(*aKind));
    assert_eq!(bitCount(aAll), 7);
}

#[test]
fn BRepGraph_EventBusTest__DefaultSubscribedRefKinds_Zero() {
    let _myGraph = SetUp();
    let aDefaultLayer = Handle::new(BRepGraph_LayerDefault::new());
    assert_eq!(aDefaultLayer.subscribed_ref_kinds(), 0);
}

#[test]
fn BRepGraph_EventBusTest__OnRefRemoved_DispatchedToAllLayers() {
    let mut myGraph = SetUp();
    let aLayer = tracker("Tracker", 0);
    register(&myGraph, &aLayer);

    // A box has FaceRef entries in its shell. Remove one.
    let aFaceRef = FaceRefId::new(0);
    assert!(
        myGraph
            .editor()
            .generic()
            .remove_ref(aFaceRef.into())
            .unwrap()
    );

    // OnRefRemoved must be dispatched regardless of SubscribedRefKinds.
    let s = aLayer.s();
    assert_eq!(s.myRefRemoveCallCount, 1);
    assert_eq!(s.myRefRemovedEvents.len(), 1);
    assert_eq!(s.myRefRemovedEvents[0], RefId::from(aFaceRef));
}

#[test]
fn BRepGraph_EventBusTest__OnRefRemoved_MultipleRefs() {
    let mut myGraph = SetUp();
    let aLayer = tracker("Tracker", 0);
    register(&myGraph, &aLayer);

    assert!(
        myGraph
            .editor()
            .generic()
            .remove_ref(FaceRefId::start().into())
            .unwrap()
    );
    assert!(
        myGraph
            .editor()
            .generic()
            .remove_ref(FaceRefId::new(1).into())
            .unwrap()
    );

    assert_eq!(aLayer.s().myRefRemoveCallCount, 2);
    assert_eq!(aLayer.s().myRefRemovedEvents.len(), 2);
}

#[test]
fn BRepGraph_EventBusTest__OnRefRemoved_AlreadyRemoved_NotDispatched() {
    let mut myGraph = SetUp();
    let aLayer = tracker("Tracker", 0);
    register(&myGraph, &aLayer);

    assert!(
        myGraph
            .editor()
            .generic()
            .remove_ref(FaceRefId::start().into())
            .unwrap()
    );
    assert_eq!(aLayer.s().myRefRemoveCallCount, 1);

    // Second removal returns false and must NOT dispatch again.
    assert!(
        !myGraph
            .editor()
            .generic()
            .remove_ref(FaceRefId::start().into())
            .unwrap()
    );
    assert_eq!(aLayer.s().myRefRemoveCallCount, 1);
}

#[test]
fn BRepGraph_EventBusTest__OverlappingSubscription_EdgeAndFace() {
    let mut myGraph = SetUp();
    // Layer subscribes to both Edge and Face - should receive events for both kinds.
    let aEdgeFace = kind_bit(NodeKind::EDGE) | kind_bit(NodeKind::FACE);
    let aLayer = tracker("EdgeFace", aEdgeFace);
    register(&myGraph, &aLayer);

    setToleranceGuarded(&mut myGraph, EdgeId::start(), 0.5);

    // Edge events from direct mutation; NO face events (no parent dispatch).
    assert!(aLayer.CountImmediateEventsOfKind(NodeKind::EDGE) > 0);
    assert_eq!(aLayer.CountImmediateEventsOfKind(NodeKind::FACE), 0);
    assert_eq!(aLayer.CountImmediateEventsOfKind(NodeKind::WIRE), 0);
}

// The two tracking layers of the iterator declarations.
fn twoLayers(
    myGraph: &BRepGraph,
) -> (
    Handle<BRepGraph_LayerModTracking>,
    Handle<BRepGraph_LayerModTracking>,
) {
    let aLayer1 = Handle::new(BRepGraph_LayerModTracking::with_id(
        "L1",
        0,
        "2f9b6a5c-1f2d-4a88-9c1c-7a0c16a10030",
        0,
    ));
    let aLayer2 = Handle::new(BRepGraph_LayerModTracking::with_id(
        "L2",
        0,
        "2f9b6a5c-1f2d-4a88-9c1c-7a0c16a10031",
        0,
    ));
    register(myGraph, &aLayer1);
    register(myGraph, &aLayer2);
    (aLayer1, aLayer2)
}

#[test]
fn BRepGraph_EventBusTest__LayerIterator_TraditionalLoop() {
    let myGraph = SetUp();
    let aBaseLayerCount = myGraph.layer_registry().nb_layers();
    let _aLayers = twoLayers(&myGraph);

    let mut aCount = 0u32;
    let mut anIt = LayerIterator::new(myGraph.layer_registry());
    while anIt.more() {
        assert!(!anIt.value().is_null());
        // Slot() must match iteration index and Layer(slot) must return the same value.
        assert!(myGraph.layer_registry().layer(anIt.slot()) == anIt.value());
        aCount += 1;
        anIt.next_slot();
    }
    assert_eq!(aCount, aBaseLayerCount + 2);
}

#[test]
fn BRepGraph_EventBusTest__LayerIterator_RangeFor() {
    let myGraph = SetUp();
    let aBaseLayerCount = myGraph.layer_registry().nb_layers();
    let (aLayer1, aLayer2) = twoLayers(&myGraph);

    let mut aCount = 0u32;
    let mut hasL1 = false;
    let mut hasL2 = false;
    for aLayer in LayerIterator::new(myGraph.layer_registry()) {
        if aLayer == aLayer1 {
            hasL1 = true;
        }
        if aLayer == aLayer2 {
            hasL2 = true;
        }
        aCount += 1;
    }
    assert_eq!(aCount, aBaseLayerCount + 2);
    assert!(hasL1);
    assert!(hasL2);
}

#[test]
fn BRepGraph_EventBusTest__LayerIterator_RangeFor_DefaultLayers() {
    let myGraph = SetUp();
    assert!(myGraph.layer_registry().nb_layers() >= 1);
    let mut aCount = 0u32;
    for aLayer in LayerIterator::new(myGraph.layer_registry()) {
        let _ = aLayer;
        aCount += 1;
    }
    assert_eq!(aCount, myGraph.layer_registry().nb_layers());
}
