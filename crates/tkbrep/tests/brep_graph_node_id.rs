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

//! Direct translation of BRepGraph_NodeId_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! `BRepPrimAPI_MakeBox` is the box of the test support module (TKPrim is not ported).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::related_iterator::{RelatedIterator, RelationKind};
use rustycad_tkbrep::brep_graph::{BRepGraph, EdgeId, FaceId, NodeId, NodeKind, VertexId};
use support::make_box;

// ============ TypedNodeId tests ============

#[test]
fn BRepGraph_NodeIdTest__Construction_DefaultInvalid() {
    let aFace = FaceId::default();
    assert!(!aFace.is_valid());
    assert_eq!(aFace.index, FaceId::INVALID_INDEX);
}

#[test]
fn BRepGraph_NodeIdTest__Construction_FromIndex() {
    let aFace = FaceId::new(5);
    assert!(aFace.is_valid());
    assert_eq!(aFace.index, 5);
}

#[test]
fn BRepGraph_NodeIdTest__TypedAliases_ConstructFromIndex() {
    let aFace = FaceId::new(3);
    assert!(aFace.is_valid());
    assert_eq!(aFace.index, 3);

    let anEdge = EdgeId::new(7);
    assert_eq!(anEdge.index, 7);
}

#[test]
fn BRepGraph_NodeIdTest__ImplicitConversion_ToNodeId() {
    let aFace = FaceId::new(5);

    // Implicit conversion to BRepGraph_NodeId.
    let aNodeId: NodeId = aFace.into();
    assert_eq!(aNodeId.kind, NodeKind::FACE);
    assert_eq!(aNodeId.index, 5);
}

#[test]
fn BRepGraph_NodeIdTest__ImplicitConversion_PassToFunction() {
    // Typed ids work with existing APIs that take BRepGraph_NodeId.
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes1 = aGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    assert!(!aGraph.is_empty());

    let aFace = FaceId::new(0);
    // RelatedIterator accepts typed ids converted through BRepGraph_NodeId.
    let mut aNbAdjacentFaces = 0;
    let mut anIt = RelatedIterator::new(&aGraph, NodeId::from(aFace)).unwrap();
    while anIt.more() {
        if anIt.current_relation() == RelationKind::AdjacentFace {
            aNbAdjacentFaces += 1;
        }
        anIt.next().unwrap();
    }
    assert!(aNbAdjacentFaces > 0);
}

#[test]
fn BRepGraph_NodeIdTest__FromNodeId_CorrectKind() {
    let aNodeId = NodeId::new(NodeKind::EDGE, 3);
    let anEdge = EdgeId::from_node_id(aNodeId);
    assert_eq!(anEdge.index, 3);
}

#[test]
fn BRepGraph_NodeIdTest__Comparison_TypedVsTyped() {
    let aFace1 = FaceId::new(3);
    let aFace2 = FaceId::new(3);
    let aFace3 = FaceId::new(5);

    assert_eq!(aFace1, aFace2);
    assert_ne!(aFace1, aFace3);
}

#[test]
fn BRepGraph_NodeIdTest__Comparison_TypedVsNodeId() {
    let aFace = FaceId::new(3);
    let aMatch = NodeId::new(NodeKind::FACE, 3);
    let aDiffKind = NodeId::new(NodeKind::EDGE, 3);
    let aDiffIdx = NodeId::new(NodeKind::FACE, 5);

    assert_eq!(aFace, aMatch);
    assert_eq!(aMatch, aFace);
    assert_ne!(aFace, aDiffKind);
    assert_ne!(aFace, aDiffIdx);
}

#[test]
fn BRepGraph_NodeIdTest__Hash_ConsistentWithNodeId() {
    let aFace = FaceId::new(5);
    let aNodeId: NodeId = aFace.into();

    assert_eq!(aFace.hash_code(), aNodeId.hash_code());
}

#[test]
fn BRepGraph_NodeIdTest__UntypedArithmetic_PreservesKindAndIndex() {
    let mut aNodeId = NodeId::new(NodeKind::FACE, 2);

    let aPrev = aNodeId.post_increment();
    assert_eq!(aPrev.kind, NodeKind::FACE);
    assert_eq!(aPrev.index, 2);
    assert_eq!(aNodeId.index, 3);

    aNodeId.pre_increment();
    assert_eq!(aNodeId.index, 4);

    let anAdvanced = aNodeId.offset(3);
    assert_eq!(anAdvanced.kind, NodeKind::FACE);
    assert_eq!(anAdvanced.index, 7);

    let aRetreated = anAdvanced.retreat(5);
    assert_eq!(aRetreated.kind, NodeKind::FACE);
    assert_eq!(aRetreated.index, 2);
}

#[test]
fn BRepGraph_NodeIdTest__TypedArithmetic_PreservesKindAndIndex() {
    let mut aFaceId = FaceId::new(2);

    let aPrev = aFaceId.post_increment();
    assert_eq!(aPrev.index, 2);
    assert_eq!(aFaceId.index, 3);

    aFaceId.pre_increment();
    assert_eq!(aFaceId.index, 4);

    let anAdvanced = aFaceId.offset(3);
    assert_eq!(anAdvanced.index, 7);

    let aRetreated = anAdvanced.retreat(5);
    assert_eq!(aRetreated.index, 2);

    // Verify kind is preserved through implicit conversion.
    let aNodeId: NodeId = anAdvanced.into();
    assert_eq!(aNodeId.kind, NodeKind::FACE);
    assert_eq!(aNodeId.index, 7);
}

#[test]
fn BRepGraph_NodeIdTest__TypedArithmetic_IndexZeroBoundary() {
    let mut anEdge = EdgeId::new(0);
    assert!(anEdge.is_valid());

    // Increment from zero.
    anEdge.pre_increment();
    assert_eq!(anEdge.index, 1);

    // Retreat back to zero  still valid.
    let aZero = anEdge.retreat(1);
    assert_eq!(aZero.index, 0);
    assert!(aZero.is_valid());

    // Subtract to the invalid sentinel produces an invalid id.
    let anInvalid = aZero.retreat(1);
    assert_eq!(anInvalid.index, EdgeId::INVALID_INDEX);
    assert!(!anInvalid.is_valid());
}

#[test]
fn BRepGraph_NodeIdTest__InvalidKind_IsRejected() {
    const THE_RESERVED_KIND: u32 = 9;
    let aNode = NodeId::new(NodeKind(THE_RESERVED_KIND as i32), 0);

    assert!(!NodeKind::is_valid_kind(aNode.kind));
    assert!(!aNode.is_valid());
    assert!(!aNode.is_valid_for(10));
    assert!(!NodeKind::is_topology_kind(aNode.kind));
    assert!(!NodeKind::is_assembly_kind(aNode.kind));
}

#[test]
fn BRepGraph_NodeIdTest__FromNodeId_WrongKindReturnsInvalid() {
    let aNode = NodeId::new(NodeKind::FACE, 3);
    let anEdge = EdgeId::from_node_id(aNode);

    assert!(!anEdge.is_valid());
    assert_eq!(anEdge.index, EdgeId::INVALID_INDEX);
}

#[test]
fn BRepGraph_NodeIdTest__OutOfRangeMetadataQueriesReturnFalse() {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes = aGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    assert!(!aGraph.is_empty());

    let anOutOfRangeVertex = VertexId::new(aGraph.topo().vertices().nb());
    assert!(!anOutOfRangeVertex.is_removed(&aGraph));
    assert!(!anOutOfRangeVertex.is_owned(&aGraph));

    let anOutOfRangeFace = NodeId::new(NodeKind::FACE, aGraph.topo().faces().nb());
    assert!(!anOutOfRangeFace.is_removed(&aGraph));
    assert!(!anOutOfRangeFace.is_owned(&aGraph));
}
