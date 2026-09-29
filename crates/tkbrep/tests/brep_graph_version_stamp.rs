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

//! Direct translation of BRepGraph_VersionStamp_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::tool::face as tool_face;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkernel::standard::Guid;
use support::make_box;

/// The `BRepGraph_VersionStampTest` fixture: a graph of a 10x20x30 box.
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

fn stamp(aGraph: &BRepGraph, aNode: impl Into<NodeId>) -> VersionStamp {
    aGraph.uids().stamp_of_node(aNode.into())
}

fn bumpFaceTolerance(myGraph: &mut BRepGraph) {
    let aTolerance = tool_face::tolerance(myGraph, FaceId::start()) + 0.01;
    myGraph
        .editor()
        .faces()
        .set_tolerance(FaceId::start(), aTolerance)
        .unwrap();
}

fn rebuild(myGraph: &mut BRepGraph) {
    myGraph.clear().unwrap();
    let _aBuildRes = myGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    assert!(!myGraph.is_empty());
}

// --- VersionStamp struct tests ---

#[test]
fn BRepGraph_VersionStampBasicTest__DefaultStamp_IsInvalid() {
    let aStamp = VersionStamp::default();
    assert!(!aStamp.is_valid());
}

#[test]
fn BRepGraph_VersionStampTest__StampOf_ValidNode_ReturnsValidStamp() {
    let myGraph = SetUp();
    let aFaceId = FaceId::new(0);
    assert!(aFaceId.is_valid_for(myGraph.topo().faces().nb()));

    let aStamp = stamp(&myGraph, aFaceId);
    assert!(aStamp.is_valid());
    assert!(aStamp.node_uid.is_valid());
    assert_eq!(aStamp.mutation_gen, 0);
    assert_eq!(aStamp.generation, myGraph.uids().generation());
}

#[test]
fn BRepGraph_VersionStampTest__StampOf_InvalidNode_ReturnsInvalid() {
    let myGraph = SetUp();
    let aStamp = stamp(&myGraph, NodeId::default());
    assert!(!aStamp.is_valid());
}

// --- IsStale tests ---

#[test]
fn BRepGraph_VersionStampTest__IsStale_UnmutatedNode_ReturnsFalse() {
    let myGraph = SetUp();
    let aStamp = stamp(&myGraph, FaceId::start());
    assert!(!myGraph.uids().is_stale(&aStamp));
}

#[test]
fn BRepGraph_VersionStampTest__IsStale_MutatedNode_ReturnsTrue() {
    let mut myGraph = SetUp();
    let aStamp = stamp(&myGraph, FaceId::start());

    // Mutate the face.
    bumpFaceTolerance(&mut myGraph);

    assert!(myGraph.uids().is_stale(&aStamp));
}

#[test]
fn BRepGraph_VersionStampTest__IsStale_RemovedNode_ReturnsTrue() {
    let mut myGraph = SetUp();
    let aStamp = stamp(&myGraph, FaceId::start());

    myGraph
        .editor()
        .generic()
        .remove_node(FaceId::start().into())
        .unwrap();

    assert!(myGraph.uids().is_stale(&aStamp));
}

#[test]
fn BRepGraph_VersionStampTest__IsStale_DifferentGeneration_ReturnsTrue() {
    let mut myGraph = SetUp();
    let aStamp = stamp(&myGraph, FaceId::start());

    // Rebuild the graph - generation changes.
    rebuild(&mut myGraph);

    assert!(myGraph.uids().is_stale(&aStamp));
}

#[test]
fn BRepGraph_VersionStampTest__IsStale_DeferredMode_TracksCorrectly() {
    let mut myGraph = SetUp();
    let aStamp = stamp(&myGraph, EdgeId::start());

    // Mutate in deferred mode.
    myGraph.editor().begin_deferred_invalidation();
    myGraph
        .editor()
        .edges()
        .set_tolerance(EdgeId::start(), 0.5)
        .unwrap();
    myGraph.editor().end_deferred_invalidation();

    assert!(myGraph.uids().is_stale(&aStamp));
}

#[test]
fn BRepGraph_VersionStampTest__IsStale_InvalidStamp_ReturnsTrue() {
    let myGraph = SetUp();
    let aStamp = VersionStamp::default();
    assert!(myGraph.uids().is_stale(&aStamp));
}

// --- Identity and version comparison ---

#[test]
fn BRepGraph_VersionStampTest__StampIdentity_SameNode_Equal() {
    let myGraph = SetUp();
    let aStamp1 = stamp(&myGraph, FaceId::start());
    let aStamp2 = stamp(&myGraph, FaceId::start());
    assert_eq!(aStamp1, aStamp2);
}

#[test]
fn BRepGraph_VersionStampTest__StampIdentity_DifferentNodes_NotEqual() {
    let myGraph = SetUp();
    let aStamp1 = stamp(&myGraph, FaceId::start());
    let aStamp2 = stamp(&myGraph, FaceId::new(1));
    assert_ne!(aStamp1, aStamp2);
}

#[test]
fn BRepGraph_VersionStampTest__IsSameItem_SameVersion_ReturnsTrue() {
    let myGraph = SetUp();
    let aStamp1 = stamp(&myGraph, FaceId::start());
    let aStamp2 = stamp(&myGraph, FaceId::start());
    assert!(aStamp1.is_same_item(&aStamp2));
}

#[test]
fn BRepGraph_VersionStampTest__IsSameItem_DifferentVersion_StillSameItem() {
    let mut myGraph = SetUp();
    let aStampBefore = stamp(&myGraph, FaceId::start());

    bumpFaceTolerance(&mut myGraph);

    let aStampAfter = stamp(&myGraph, FaceId::start());

    // Full equality fails (different MutationGen).
    assert_ne!(aStampBefore, aStampAfter);
    // But they refer to the same item identity.
    assert!(aStampBefore.is_same_item(&aStampAfter));
}

#[test]
fn BRepGraph_VersionStampTest__StampOf_AssemblyNodes_WorksForProductsAndOccurrences() {
    let myGraph = SetUp();
    // Box graph auto-creates a root Product.
    assert!(myGraph.topo().products().nb() > 0);

    let aProdStamp = stamp(&myGraph, ProductId::start());
    assert!(aProdStamp.is_valid());
    assert!(!myGraph.uids().is_stale(&aProdStamp));
}

#[test]
fn BRepGraph_VersionStampTest__GenericItemUID_NodeAndReferenceItems_RoundTrip() {
    let myGraph = SetUp();
    let aFaceId = FaceId::start();
    let aFaceItem = ItemId::from(NodeId::from(aFaceId));
    let aFaceUID = myGraph.uids().of_item(aFaceItem);
    assert!(aFaceUID.is_valid());
    assert!(aFaceUID.is_node());
    assert_eq!(aFaceUID.node_kind(), NodeKind::FACE);
    assert_eq!(myGraph.uids().item_id_from(&aFaceUID), aFaceItem);
    assert!(myGraph.uids().has_item(&aFaceUID));

    let aFaceStamp = myGraph.uids().stamp_of_item(aFaceItem);
    assert!(aFaceStamp.is_valid());
    assert!(aFaceStamp.is_node_stamp());
    assert_eq!(aFaceStamp.item_uid(), aFaceUID);

    let aFaceRefId = FaceRefId::start();
    let aFaceRefItem = ItemId::from(RefId::from(aFaceRefId));
    let aFaceRefUID = myGraph.uids().of_item(aFaceRefItem);
    assert!(aFaceRefUID.is_valid());
    assert!(aFaceRefUID.is_reference());
    assert_eq!(aFaceRefUID.ref_kind(), RefKind::FACE);
    assert_eq!(myGraph.uids().item_id_from(&aFaceRefUID), aFaceRefItem);
    assert!(myGraph.uids().has_item(&aFaceRefUID));

    let aFaceRefStamp = myGraph.uids().stamp_of_item(aFaceRefItem);
    assert!(aFaceRefStamp.is_valid());
    assert!(aFaceRefStamp.is_ref_stamp());
    assert_eq!(aFaceRefStamp.item_uid(), aFaceRefUID);
}

#[test]
fn BRepGraph_VersionStampTest__StampOf_RemovedUse_ReturnsInvalidUntilReused() {
    let mut myGraph = SetUp();
    let aFaceId = FaceId::start();
    let aRepId = myGraph.topo().faces().definition(aFaceId).surface_rep_id;
    assert!(aRepId.is_valid());

    let aSurface = tool_face::surface(&myGraph, aFaceId);
    assert!(!aSurface.is_null());
    assert!(myGraph.uids().stamp_of_rep(aRepId.into()).is_valid());

    myGraph.editor().faces().clear_surface(aFaceId).unwrap();
    assert!(!myGraph.uids().stamp_of_rep(aRepId.into()).is_valid());

    myGraph
        .editor()
        .faces()
        .set_surface(aFaceId, &aSurface)
        .unwrap();
    assert_eq!(
        myGraph.topo().faces().definition(aFaceId).surface_rep_id,
        aRepId
    );
    assert!(myGraph.uids().stamp_of_rep(aRepId.into()).is_valid());
}

// --- Graph GUID tests ---

#[test]
fn BRepGraph_VersionStampTest__GraphGUID_AfterBuild_IsValid() {
    let myGraph = SetUp();
    let aGUID = myGraph.uids().graph_guid();
    // A random GUID should not be all zeros.
    let aZero = Guid::default();
    assert_ne!(aGUID, aZero);
}

#[test]
fn BRepGraph_VersionStampTest__GraphGUID_Rebuild_Changes() {
    let mut myGraph = SetUp();
    let aGUID1 = myGraph.uids().graph_guid();

    rebuild(&mut myGraph);

    let aGUID2 = myGraph.uids().graph_guid();
    // Two random GUIDs should differ (probability of collision is negligible).
    assert_ne!(aGUID1, aGUID2);
}

// --- ToGUID tests ---

#[test]
fn BRepGraph_VersionStampTest__ToGUID_Deterministic_SameInputSameOutput() {
    let myGraph = SetUp();
    let aStamp = stamp(&myGraph, FaceId::start());
    let aGraph = myGraph.uids().graph_guid();
    let aGUID1 = aStamp.to_guid(aGraph);
    let aGUID2 = aStamp.to_guid(aGraph);
    assert_eq!(aGUID1, aGUID2);
}

#[test]
fn BRepGraph_VersionStampTest__ToGUID_DifferentMutationGen_DifferentGUID() {
    let mut myGraph = SetUp();
    let aStampBefore = stamp(&myGraph, FaceId::start());
    let aGraph = myGraph.uids().graph_guid();
    let aGUIDBefore = aStampBefore.to_guid(aGraph);

    bumpFaceTolerance(&mut myGraph);

    let aStampAfter = stamp(&myGraph, FaceId::start());
    let aGUIDAfter = aStampAfter.to_guid(aGraph);

    assert_ne!(aGUIDBefore, aGUIDAfter);
}

#[test]
fn BRepGraph_VersionStampTest__ToGUID_DifferentGraphGUID_DifferentGUID() {
    let myGraph = SetUp();
    let aStamp = stamp(&myGraph, FaceId::start());

    let aGraphA = Guid::from_ascii(b"a1b2c3d4-e5f6-7890-abcd-ef0123456789").unwrap();
    let aGraphB = Guid::from_ascii(b"11223344-5566-7788-99aa-bbccddeeff00").unwrap();

    let aGUIDA = aStamp.to_guid(aGraphA);
    let aGUIDB = aStamp.to_guid(aGraphB);

    assert_ne!(aGUIDA, aGUIDB);
}

#[test]
fn BRepGraph_VersionStampTest__ToGUID_DifferentNodes_DifferentGUID() {
    let myGraph = SetUp();
    let aGraph = myGraph.uids().graph_guid();

    let aStamp0 = stamp(&myGraph, FaceId::start());
    let aStamp1 = stamp(&myGraph, FaceId::new(1));

    assert_ne!(aStamp0.to_guid(aGraph), aStamp1.to_guid(aGraph));
}
