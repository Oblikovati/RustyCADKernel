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

//! Direct translation of BRepGraph_BatchOps_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871 (the suite includes BRepGraph_Validate and
//! BRepGraph_Compact but calls neither). The batch `Append` overloads are the
//! `append_many` members; an empty orientation slice is the defaulted native array.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkmath::toploc::Location;

const FORWARD: ParityOrientation = ParityOrientation::FORWARD;

// =======================================================================
// Helper: create a simple face
// =======================================================================

fn createSimpleFace(theGraph: &mut BRepGraph) -> FaceId {
    let aV0 = theGraph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 1.0e-7);
    let aV1 = theGraph
        .editor()
        .vertices()
        .add(Point3::new(10.0, 0.0, 0.0), 1.0e-7);
    let anEdge = theGraph
        .editor()
        .edges()
        .add(aV0, aV1, &Handle::default(), 0.0, 10.0, 1.0e-7);
    let aCoEdges = vec![theGraph.editor().co_edges().add(anEdge, FORWARD)];
    let aWire = theGraph.editor().wires().add(&aCoEdges);
    let aPlane: Handle<dyn GeomSurface> = Handle::new(
        GeomPlane::from_point(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        )
        .unwrap(),
    )
    .into_cast();
    let anEmpty: Vec<WireId> = Vec::new();
    theGraph
        .editor()
        .faces()
        .add(&aPlane, aWire, &anEmpty, 1.0e-7)
}

fn appendFace(theGraph: &mut BRepGraph, theShell: ShellId, theFace: FaceId) -> FaceRefId {
    theGraph
        .editor()
        .shells()
        .append(theShell, theFace, FORWARD)
        .unwrap()
}

fn appendShell(theGraph: &mut BRepGraph, theSolid: SolidId, theShell: ShellId) -> ShellRefId {
    theGraph
        .editor()
        .solids()
        .append(theSolid, theShell, FORWARD)
        .unwrap()
}

// =======================================================================
// ShellOps::Append (batch)
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__ShellOps_AppendBatch_AllForward() {
    let mut aGraph = BRepGraph::new();

    let aFace0 = createSimpleFace(&mut aGraph);
    let aFace1 = createSimpleFace(&mut aGraph);
    let aFace2 = createSimpleFace(&mut aGraph);
    assert!(aFace0.is_valid());
    assert!(aFace1.is_valid());
    assert!(aFace2.is_valid());

    let aShell = aGraph.editor().shells().add();
    assert!(aShell.is_valid());

    let aFaces = [aFace0, aFace1, aFace2];

    let aRefs = aGraph
        .editor()
        .shells()
        .append_many(aShell, &aFaces, &[])
        .unwrap();
    assert_eq!(aRefs.len(), 3);
    assert!(aRefs[0].is_valid());
    assert!(aRefs[1].is_valid());
    assert!(aRefs[2].is_valid());
}

#[test]
fn BRepGraph_BatchOpsTest__ShellOps_AppendBatch_WithOrientations() {
    let mut aGraph = BRepGraph::new();

    let aFace0 = createSimpleFace(&mut aGraph);
    let aFace1 = createSimpleFace(&mut aGraph);
    assert!(aFace0.is_valid());
    assert!(aFace1.is_valid());

    let aShell = aGraph.editor().shells().add();
    assert!(aShell.is_valid());

    let aFaces = [aFace0, aFace1];
    let aOrientations = [ParityOrientation::FORWARD, ParityOrientation::REVERSED];

    let aRefs = aGraph
        .editor()
        .shells()
        .append_many(aShell, &aFaces, &aOrientations)
        .unwrap();
    assert_eq!(aRefs.len(), 2);
    assert!(aRefs[0].is_valid());
    assert!(aRefs[1].is_valid());
}

#[test]
fn BRepGraph_BatchOpsTest__ShellOps_AppendBatch_InvalidFace_ReturnsEmpty() {
    let mut aGraph = BRepGraph::new();

    let aFace0 = createSimpleFace(&mut aGraph);
    assert!(aFace0.is_valid());

    let aShell = aGraph.editor().shells().add();
    assert!(aShell.is_valid());

    let aFaces = [aFace0, FaceId::new(999)]; // invalid

    let aRefs = aGraph
        .editor()
        .shells()
        .append_many(aShell, &aFaces, &[])
        .unwrap();
    assert_eq!(aRefs.len(), 0);
}

#[test]
fn BRepGraph_BatchOpsTest__ShellOps_AppendBatch_InvalidShell_ReturnsEmpty() {
    let mut aGraph = BRepGraph::new();

    let aFace0 = createSimpleFace(&mut aGraph);
    assert!(aFace0.is_valid());

    let aFaces = [aFace0];

    let aRefs = aGraph
        .editor()
        .shells()
        .append_many(ShellId::new(999), &aFaces, &[])
        .unwrap();
    assert_eq!(aRefs.len(), 0);
}

// =======================================================================
// ShellOps::RemoveFaces (batch, all-or-nothing)
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__ShellOps_RemoveFaces_AllRemoved() {
    let mut aGraph = BRepGraph::new();

    let aFace0 = createSimpleFace(&mut aGraph);
    let aFace1 = createSimpleFace(&mut aGraph);
    assert!(aFace0.is_valid());
    assert!(aFace1.is_valid());

    let aShell = aGraph.editor().shells().add();
    assert!(aShell.is_valid());

    let aRef0 = appendFace(&mut aGraph, aShell, aFace0);
    let aRef1 = appendFace(&mut aGraph, aShell, aFace1);
    assert!(aRef0.is_valid());
    assert!(aRef1.is_valid());

    let aRefs = [aRef0, aRef1];

    assert!(
        aGraph
            .editor()
            .shells()
            .remove_faces(aShell, &aRefs)
            .unwrap()
    );
}

#[test]
fn BRepGraph_BatchOpsTest__ShellOps_RemoveFaces_InvalidRef_ReturnsFalse() {
    let mut aGraph = BRepGraph::new();

    let aFace0 = createSimpleFace(&mut aGraph);
    assert!(aFace0.is_valid());

    let aShell = aGraph.editor().shells().add();
    assert!(aShell.is_valid());

    let aRef0 = appendFace(&mut aGraph, aShell, aFace0);
    assert!(aRef0.is_valid());

    let aRefs = [aRef0, FaceRefId::new(999)]; // invalid

    assert!(
        !aGraph
            .editor()
            .shells()
            .remove_faces(aShell, &aRefs)
            .unwrap()
    );
}

// =======================================================================
// SolidOps::Append (batch)
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__SolidOps_AppendBatch_AllForward() {
    let mut aGraph = BRepGraph::new();

    let aFace0 = createSimpleFace(&mut aGraph);
    let aFace1 = createSimpleFace(&mut aGraph);
    assert!(aFace0.is_valid());
    assert!(aFace1.is_valid());

    let aShell0 = aGraph.editor().shells().add();
    let aShell1 = aGraph.editor().shells().add();
    assert!(aShell0.is_valid());
    assert!(aShell1.is_valid());

    appendFace(&mut aGraph, aShell0, aFace0);
    appendFace(&mut aGraph, aShell1, aFace1);

    let aSolid = aGraph.editor().solids().add();
    assert!(aSolid.is_valid());

    let aShells = [aShell0, aShell1];

    let aRefs = aGraph
        .editor()
        .solids()
        .append_many(aSolid, &aShells, &[])
        .unwrap();
    assert_eq!(aRefs.len(), 2);
    assert!(aRefs[0].is_valid());
    assert!(aRefs[1].is_valid());
}

// =======================================================================
// SolidOps::RemoveShells (batch, all-or-nothing)
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__SolidOps_RemoveShells_AllRemoved() {
    let mut aGraph = BRepGraph::new();

    let aFace0 = createSimpleFace(&mut aGraph);
    assert!(aFace0.is_valid());

    let aShell0 = aGraph.editor().shells().add();
    assert!(aShell0.is_valid());
    appendFace(&mut aGraph, aShell0, aFace0);

    let aSolid = aGraph.editor().solids().add();
    assert!(aSolid.is_valid());

    let aRef0 = appendShell(&mut aGraph, aSolid, aShell0);
    assert!(aRef0.is_valid());

    let aRefs = [aRef0];

    assert!(
        aGraph
            .editor()
            .solids()
            .remove_shells(aSolid, &aRefs)
            .unwrap()
    );
}

// =======================================================================
// CompoundOps::Append (batch)
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__CompoundOps_AppendBatch_AllForward() {
    let mut aGraph = BRepGraph::new();

    let aFace0 = createSimpleFace(&mut aGraph);
    let aFace1 = createSimpleFace(&mut aGraph);
    assert!(aFace0.is_valid());
    assert!(aFace1.is_valid());

    let anEmptyChildren: Vec<NodeId> = Vec::new();
    let aCompound = aGraph.editor().compounds().add(&anEmptyChildren);
    assert!(aCompound.is_valid());

    let aChildren: [NodeId; 2] = [aFace0.into(), aFace1.into()];

    let aRefs = aGraph
        .editor()
        .compounds()
        .append_many(aCompound, &aChildren, &[])
        .unwrap();
    assert_eq!(aRefs.len(), 2);
    assert!(aRefs[0].is_valid());
    assert!(aRefs[1].is_valid());
}

// =======================================================================
// CompoundOps::RemoveChildren (batch, all-or-nothing)
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__CompoundOps_RemoveChildren_AllRemoved() {
    let mut aGraph = BRepGraph::new();

    let aFace0 = createSimpleFace(&mut aGraph);
    assert!(aFace0.is_valid());

    let anEmptyChildren: Vec<NodeId> = Vec::new();
    let aCompound = aGraph.editor().compounds().add(&anEmptyChildren);
    assert!(aCompound.is_valid());

    let aRef0 = aGraph
        .editor()
        .compounds()
        .append(aCompound, aFace0.into(), FORWARD)
        .unwrap();
    assert!(aRef0.is_valid());

    let aRefs = [aRef0];

    assert!(
        aGraph
            .editor()
            .compounds()
            .remove_children(aCompound, &aRefs)
            .unwrap()
    );
}

// =======================================================================
// CompoundOps::ReplaceChild
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__CompoundOps_ReplaceChild() {
    let mut aGraph = BRepGraph::new();

    let aFace0 = createSimpleFace(&mut aGraph);
    let aFace1 = createSimpleFace(&mut aGraph);
    assert!(aFace0.is_valid());
    assert!(aFace1.is_valid());

    let anEmptyChildren: Vec<NodeId> = Vec::new();
    let aCompound = aGraph.editor().compounds().add(&anEmptyChildren);
    assert!(aCompound.is_valid());

    let aRef0 = aGraph
        .editor()
        .compounds()
        .append(aCompound, aFace0.into(), FORWARD)
        .unwrap();
    assert!(aRef0.is_valid());

    aGraph
        .editor()
        .compounds()
        .replace_child(aRef0, aFace1.into())
        .unwrap();
}

// Two solids, each with one shell of one simple face.
fn twoSolids(aGraph: &mut BRepGraph) -> (SolidId, SolidId) {
    let aFace0 = createSimpleFace(aGraph);
    let aFace1 = createSimpleFace(aGraph);
    assert!(aFace0.is_valid());
    assert!(aFace1.is_valid());

    let aShell0 = aGraph.editor().shells().add();
    let aShell1 = aGraph.editor().shells().add();
    assert!(aShell0.is_valid());
    assert!(aShell1.is_valid());

    appendFace(aGraph, aShell0, aFace0);
    appendFace(aGraph, aShell1, aFace1);

    let aSolid0 = aGraph.editor().solids().add();
    let aSolid1 = aGraph.editor().solids().add();
    assert!(aSolid0.is_valid());
    assert!(aSolid1.is_valid());

    appendShell(aGraph, aSolid0, aShell0);
    appendShell(aGraph, aSolid1, aShell1);
    (aSolid0, aSolid1)
}

// One solid with one shell of one simple face.
fn oneSolid(aGraph: &mut BRepGraph) -> SolidId {
    let aFace0 = createSimpleFace(aGraph);
    assert!(aFace0.is_valid());

    let aShell0 = aGraph.editor().shells().add();
    assert!(aShell0.is_valid());
    appendFace(aGraph, aShell0, aFace0);

    let aSolid0 = aGraph.editor().solids().add();
    assert!(aSolid0.is_valid());
    appendShell(aGraph, aSolid0, aShell0);
    aSolid0
}

// =======================================================================
// CompSolidOps::Append (batch)
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__CompSolidOps_AppendBatch_AllForward() {
    let mut aGraph = BRepGraph::new();

    let (aSolid0, aSolid1) = twoSolids(&mut aGraph);

    let anEmptySolids: Vec<SolidId> = Vec::new();
    let aCompSolid = aGraph.editor().comp_solids().add(&anEmptySolids);
    assert!(aCompSolid.is_valid());

    let aSolids = [aSolid0, aSolid1];

    let aRefs = aGraph
        .editor()
        .comp_solids()
        .append_many(aCompSolid, &aSolids, &[])
        .unwrap();
    assert_eq!(aRefs.len(), 2);
    assert!(aRefs[0].is_valid());
    assert!(aRefs[1].is_valid());
}

// =======================================================================
// CompSolidOps::RemoveSolids (batch, all-or-nothing)
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__CompSolidOps_RemoveSolids_AllRemoved() {
    let mut aGraph = BRepGraph::new();

    let aSolid0 = oneSolid(&mut aGraph);

    let anEmptySolids: Vec<SolidId> = Vec::new();
    let aCompSolid = aGraph.editor().comp_solids().add(&anEmptySolids);
    assert!(aCompSolid.is_valid());

    let aRef0 = aGraph
        .editor()
        .comp_solids()
        .append(aCompSolid, aSolid0, FORWARD)
        .unwrap();
    assert!(aRef0.is_valid());

    let aRefs = [aRef0];

    assert!(
        aGraph
            .editor()
            .comp_solids()
            .remove_solids(aCompSolid, &aRefs)
            .unwrap()
    );
}

// =======================================================================
// CompSolidOps::ReplaceSolid
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__CompSolidOps_ReplaceSolid() {
    let mut aGraph = BRepGraph::new();

    let (aSolid0, aSolid1) = twoSolids(&mut aGraph);

    let anEmptySolids: Vec<SolidId> = Vec::new();
    let aCompSolid = aGraph.editor().comp_solids().add(&anEmptySolids);
    assert!(aCompSolid.is_valid());

    let aRef0 = aGraph
        .editor()
        .comp_solids()
        .append(aCompSolid, aSolid0, FORWARD)
        .unwrap();
    assert!(aRef0.is_valid());

    aGraph
        .editor()
        .comp_solids()
        .replace_solid(aRef0, aSolid1)
        .unwrap();
}

// =======================================================================
// ProductOps::Append (batch)
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__ProductOps_AppendBatch() {
    let mut aGraph = BRepGraph::new();

    let (aSolid0, aSolid1) = twoSolids(&mut aGraph);

    let aChildProduct0 = aGraph
        .editor()
        .products()
        .add(aSolid0.into(), &Location::default());
    let aChildProduct1 = aGraph
        .editor()
        .products()
        .add(aSolid1.into(), &Location::default());
    assert!(aChildProduct0.is_valid());
    assert!(aChildProduct1.is_valid());

    let aParentProduct = aGraph.editor().products().add_empty();
    assert!(aParentProduct.is_valid());

    let aChildren = [aChildProduct0, aChildProduct1];

    let aPlacements = [Location::default(), Location::default()];

    let aRefs = aGraph
        .editor()
        .products()
        .append_many(aParentProduct, &aChildren, &aPlacements)
        .unwrap();
    assert_eq!(aRefs.len(), 2);
    assert!(aRefs[0].is_valid());
    assert!(aRefs[1].is_valid());
}

// =======================================================================
// ProductOps::RemoveOccurrences (batch, all-or-nothing)
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__ProductOps_RemoveOccurrences_AllRemoved() {
    let mut aGraph = BRepGraph::new();

    let aSolid0 = oneSolid(&mut aGraph);

    let aChildProduct = aGraph
        .editor()
        .products()
        .add(aSolid0.into(), &Location::default());
    assert!(aChildProduct.is_valid());

    let aParentProduct = aGraph.editor().products().add_empty();
    assert!(aParentProduct.is_valid());

    let (anOccId, anOutRefId) = aGraph
        .editor()
        .products()
        .append(
            aParentProduct,
            aChildProduct,
            &Location::default(),
            OccurrenceId::default(),
        )
        .unwrap();
    assert!(anOccId.is_valid());
    assert!(anOutRefId.is_valid());

    let aRefs = [anOutRefId];

    assert!(
        aGraph
            .editor()
            .products()
            .remove_occurrences(aParentProduct, &aRefs)
            .unwrap()
    );
}

// =======================================================================
// Validation after batch operations
// =======================================================================

#[test]
fn BRepGraph_BatchOpsTest__Validation_AfterBatchOperations() {
    let mut aGraph = BRepGraph::new();

    let aFace0 = createSimpleFace(&mut aGraph);
    let aFace1 = createSimpleFace(&mut aGraph);
    let aFace2 = createSimpleFace(&mut aGraph);
    assert!(aFace0.is_valid());
    assert!(aFace1.is_valid());
    assert!(aFace2.is_valid());

    let aShell = aGraph.editor().shells().add();
    assert!(aShell.is_valid());

    // Batch append
    let aFaces = [aFace0, aFace1, aFace2];

    let aRefs = aGraph
        .editor()
        .shells()
        .append_many(aShell, &aFaces, &[])
        .unwrap();
    assert_eq!(aRefs.len(), 3);
    assert!(aRefs[0].is_valid());
    assert!(aRefs[1].is_valid());
    assert!(aRefs[2].is_valid());

    // Batch remove
    let aRefsToRemove = [aRefs[0], aRefs[1]];

    assert!(
        aGraph
            .editor()
            .shells()
            .remove_faces(aShell, &aRefsToRemove)
            .unwrap()
    );
}
