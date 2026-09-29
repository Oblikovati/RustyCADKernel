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

//! Direct translation of BRepGraph_RelatedIterator_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported; the suite includes
//! BRepPrimAPI_MakeCylinder but calls no cylinder).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_graph::iterator::CompoundIterator;
use rustycad_tkbrep::brep_graph::related_iterator::{RelatedIterator, RelationKind};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::topods::Compound;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use support::make_box;

fn hasRelatedNode(
    theGraph: &BRepGraph,
    theSource: NodeId,
    theNode: NodeId,
    theRelation: RelationKind,
) -> bool {
    let mut anIt = RelatedIterator::new(theGraph, theSource).unwrap();
    while anIt.more() {
        if anIt.current() == theNode && anIt.current_relation() == theRelation {
            return true;
        }
        anIt.next().unwrap();
    }
    false
}

fn countRelations(theGraph: &BRepGraph, theNode: NodeId, theRelation: RelationKind) -> u32 {
    let mut aCount = 0;
    let mut anIt = RelatedIterator::new(theGraph, theNode).unwrap();
    while anIt.more() {
        if anIt.current_relation() == theRelation {
            aCount += 1;
        }
        anIt.next().unwrap();
    }
    aCount
}

// The nodes of one relation kind, in iteration order.
fn relatedNodes(theGraph: &BRepGraph, theNode: NodeId, theRelation: RelationKind) -> Vec<NodeId> {
    let mut aNodes = Vec::new();
    let mut anIt = RelatedIterator::new(theGraph, theNode).unwrap();
    while anIt.more() {
        if anIt.current_relation() == theRelation {
            aNodes.push(anIt.current());
        }
        anIt.next().unwrap();
    }
    aNodes
}

/// The `BRepGraph_RelatedIteratorTest` fixture: a graph of a 10x20x30 box.
fn SetUp() -> BRepGraph {
    let mut myGraph = BRepGraph::new();
    myGraph.clear().unwrap();
    let _aBuildRes1 = myGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    myGraph
}

#[test]
fn BRepGraph_RelatedIteratorTest__FaceOfBox_ReturnsBoundaryEdgesAndOuterWire() {
    let myGraph = SetUp();
    let aFaceId = FaceId::new(0);

    let aBoundaryEdgeCount = countRelations(&myGraph, aFaceId.into(), RelationKind::BoundaryEdge);
    let anAdjacentFaceCount = countRelations(&myGraph, aFaceId.into(), RelationKind::AdjacentFace);
    let anOuterWireCount = countRelations(&myGraph, aFaceId.into(), RelationKind::OuterWire);

    assert_eq!(aBoundaryEdgeCount, 4);
    assert_eq!(anAdjacentFaceCount, 4);
    assert_eq!(anOuterWireCount, 1);
    assert!(hasRelatedNode(
        &myGraph,
        aFaceId.into(),
        WireId::start().into(),
        RelationKind::OuterWire
    ));
}

#[test]
fn BRepGraph_RelatedIteratorTest__EdgeOfBox_ReturnsIncidentVerticesAndFaces() {
    let myGraph = SetUp();
    let anEdgeId = EdgeId::new(0);

    let aVertexCount = countRelations(&myGraph, anEdgeId.into(), RelationKind::IncidentVertex);
    let aFaceCount = countRelations(&myGraph, anEdgeId.into(), RelationKind::ReferencedByFace);

    assert_eq!(aVertexCount, 2);
    assert_eq!(aFaceCount, 2);
}

#[test]
fn BRepGraph_RelatedIteratorTest__AssemblyNodes_YieldNoRelations() {
    let mut myGraph = SetUp();
    // Assembly/container nodes have no topological relations - use ChildExplorer instead.
    let aPartProduct = myGraph
        .editor()
        .products()
        .add(SolidId::start().into(), &Location::default());
    myGraph
        .editor()
        .products()
        .append_document_root(aPartProduct)
        .unwrap();
    let aRootAssembly = myGraph.editor().products().add_empty();
    myGraph
        .editor()
        .products()
        .append_document_root(aRootAssembly)
        .unwrap();

    let aTrsf = Transform3::translation_vector(Vector3::new(1.0, 2.0, 3.0));
    let (anOccurrenceId, _) = myGraph
        .editor()
        .products()
        .append(
            aRootAssembly,
            aPartProduct,
            &Location::new(aTrsf),
            OccurrenceId::default(),
        )
        .unwrap();
    assert!(anOccurrenceId.is_valid());

    let aProductIt = RelatedIterator::new(&myGraph, aRootAssembly.into()).unwrap();
    assert!(!aProductIt.more());

    let anOccIt = RelatedIterator::new(&myGraph, anOccurrenceId.into()).unwrap();
    assert!(!anOccIt.more());
}

#[test]
fn BRepGraph_RelatedIteratorTest__InvalidNode_ReturnsEmpty() {
    let myGraph = SetUp();
    let anIt = RelatedIterator::new(&myGraph, NodeId::default()).unwrap();
    assert!(!anIt.more());
}

// The two faces of an edge (the iterator must yield exactly two).
fn twoFacesOf(theGraph: &BRepGraph, theEdge: EdgeId) -> (FaceId, FaceId) {
    let mut aFaceIt = theGraph.topo().edges().faces_of(theEdge);
    assert!(aFaceIt.more());
    let aFirst = aFaceIt.current_id();
    aFaceIt.next();
    assert!(aFaceIt.more());
    let aSecond = aFaceIt.current_id();
    aFaceIt.next();
    assert!(!aFaceIt.more());
    (aFirst, aSecond)
}

#[test]
fn BRepGraph_RelatedIteratorTest__EdgeReferencedByFace_RemovedFaceIsSkipped() {
    let mut myGraph = SetUp();
    // A box edge is shared by exactly 2 faces. Removing one face should leave
    // exactly 1 ReferencedByFace relation for the edge. This specifically
    // tests that removed faces are not skipped incorrectly (no double-increment).
    let anEdgeId = EdgeId::new(0);

    // Find the two faces that reference this edge.
    let (aRemovedFace, aSurvivor) = twoFacesOf(&myGraph, anEdgeId);

    // Remove the first face.
    myGraph
        .editor()
        .generic()
        .remove_node(aRemovedFace.into())
        .unwrap();

    // The edge should report exactly 1 ReferencedByFace (the surviving face).
    let aFaceCount = countRelations(&myGraph, anEdgeId.into(), RelationKind::ReferencedByFace);
    assert_eq!(aFaceCount, 1);
    assert!(hasRelatedNode(
        &myGraph,
        anEdgeId.into(),
        aSurvivor.into(),
        RelationKind::ReferencedByFace
    ));
}

#[test]
fn BRepGraph_RelatedIteratorTest__ContainerNodes_YieldNoRelations() {
    let myGraph = SetUp();
    // Solid, Shell are containers - no topological relations.
    let aSolidIt = RelatedIterator::new(&myGraph, SolidId::start().into()).unwrap();
    assert!(!aSolidIt.more());

    let aShellIt = RelatedIterator::new(&myGraph, ShellId::start().into()).unwrap();
    assert!(!aShellIt.more());
}

#[test]
fn BRepGraph_RelatedIteratorTest__WireOfBox_ReturnsCoEdges() {
    let myGraph = SetUp();
    let aWireId = WireId::new(0);
    let aCount = countRelations(&myGraph, aWireId.into(), RelationKind::WireCoEdge);
    assert_eq!(aCount, 4);
}

#[test]
fn BRepGraph_RelatedIteratorTest__WireOfBox_ReturnsOwningFace() {
    let myGraph = SetUp();
    let aWireId = WireId::new(0);
    let aCount = countRelations(&myGraph, aWireId.into(), RelationKind::OwningFace);
    assert_eq!(aCount, 1);
}

#[test]
fn BRepGraph_RelatedIteratorTest__VertexOfBox_ReturnsIncidentEdges() {
    let myGraph = SetUp();
    let aVertexId = VertexId::new(0);
    let aCount = countRelations(&myGraph, aVertexId.into(), RelationKind::IncidentEdge);
    // Each vertex of a box touches exactly 3 edges.
    assert_eq!(aCount, 3);
}

#[test]
fn BRepGraph_RelatedIteratorTest__CoEdgeOfBox_ReturnsParentEdgeAndOwningFace() {
    let myGraph = SetUp();
    let aCoEdgeId = CoEdgeId::new(0);
    let aParentEdgeCount = countRelations(&myGraph, aCoEdgeId.into(), RelationKind::ParentEdge);
    let aOwningFaceCount = countRelations(&myGraph, aCoEdgeId.into(), RelationKind::OwningFace);
    assert_eq!(aParentEdgeCount, 1);
    assert_eq!(aOwningFaceCount, 1);
}

#[test]
fn BRepGraph_RelatedIteratorStandalone__Compound_YieldsNoRelations() {
    let aBuilder = BRepBuilder::default();
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);
    aBuilder
        .add(&mut aCompound, &make_box(10.0, 10.0, 10.0))
        .unwrap();
    aBuilder
        .add(&mut aCompound, &make_box(20.0, 20.0, 20.0))
        .unwrap();

    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes2 = aGraph.shapes_mut().add(&aCompound).unwrap();
    assert!(!aGraph.is_empty());

    let mut aCompoundId = CompoundId::default();
    let anIt = CompoundIterator::new(&aGraph);
    if anIt.more() {
        aCompoundId = anIt.current_id();
    }
    assert!(aCompoundId.is_valid());

    let anIt = RelatedIterator::new(&aGraph, aCompoundId.into()).unwrap();
    assert!(!anIt.more());
}

#[test]
fn BRepGraph_RelatedIteratorTest__RangeFor_WorksCorrectly() {
    let myGraph = SetUp();
    let aFaceId = FaceId::new(0);
    let mut aCount: u32 = 0;
    for aNode in RelatedIterator::new(&myGraph, aFaceId.into()).unwrap() {
        assert!(aNode.unwrap().is_valid());
        aCount += 1;
    }
    // Face has 4 boundary edges + adjacent faces + 1 outer wire = at least 9.
    assert!(aCount >= 9);
}

#[test]
fn BRepGraph_RelatedIteratorTest__EdgeOfBox_AllRelationsSequential() {
    let myGraph = SetUp();
    // Edge 0 of a box is shared by 2 faces and has 2 vertices.
    let anEdgeId = EdgeId::new(0);
    let anEdgeNode = NodeId::from(anEdgeId);

    let aFaces = relatedNodes(&myGraph, anEdgeNode, RelationKind::ReferencedByFace);
    let aVertices = relatedNodes(&myGraph, anEdgeNode, RelationKind::IncidentVertex);

    assert_eq!(aFaces.len(), 2);
    assert_eq!(aVertices.len(), 2);
    // All yielded nodes must be distinct.
    if aFaces.len() == 2 {
        assert_ne!(aFaces[0], aFaces[1]);
    }
    if aVertices.len() == 2 {
        assert_ne!(aVertices[0], aVertices[1]);
    }
}

#[test]
fn BRepGraph_RelatedIteratorTest__EdgeOfBox_RemovedFace_CorrectTransition() {
    let mut myGraph = SetUp();
    let anEdgeId = EdgeId::new(0);
    let anEdgeNode = NodeId::from(anEdgeId);

    // Remove one parent face.
    let (aRemovedFace, _) = twoFacesOf(&myGraph, anEdgeId);
    myGraph
        .editor()
        .generic()
        .remove_node(aRemovedFace.into())
        .unwrap();

    // Iterate - expect 1 face + 2 vertices, confirming correct stage transition.
    let aFaceCount = countRelations(&myGraph, anEdgeNode, RelationKind::ReferencedByFace);
    let aVertexCount = countRelations(&myGraph, anEdgeNode, RelationKind::IncidentVertex);
    assert_eq!(aFaceCount, 1);
    assert_eq!(aVertexCount, 2);
}

#[test]
fn BRepGraph_RelatedIteratorTest__VertexOfBox_AllParentEdgesYielded() {
    let myGraph = SetUp();
    // Vertex 0 of a box is touched by 3 edges.
    let aVertexId = VertexId::new(0);
    let aVertexNode = NodeId::from(aVertexId);

    let anEdges = relatedNodes(&myGraph, aVertexNode, RelationKind::IncidentEdge);

    assert_eq!(anEdges.len(), 3);
    // All yielded edges must be distinct.
    for i in 0..anEdges.len() {
        for j in (i + 1)..anEdges.len() {
            assert_ne!(anEdges[i], anEdges[j]);
        }
    }
}
