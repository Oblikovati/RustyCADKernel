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

//! Direct translation of BRepGraph_ReplaceVertex_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported).
//!
//! Coverage for EditorView::EdgeOps::ReplaceVertex - boundary/internal vertex
//! remap without a full edge rebuild.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::validate::{self, Options};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkmath::gp::Point3;
use support::make_box;

fn makeBoxGraph() -> BRepGraph {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes1 = aGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    aGraph
}

fn replaceVertex(
    aGraph: &mut BRepGraph,
    theEdge: EdgeId,
    theOldRef: VertexRefId,
    theNewVertex: VertexId,
) -> VertexRefId {
    aGraph
        .editor()
        .edges()
        .replace_vertex(theEdge, theOldRef, theNewVertex)
        .unwrap()
}

fn isLightweightValid(aGraph: &BRepGraph) -> bool {
    validate::perform_with_options(aGraph, &Options::lightweight()).is_valid()
}

#[test]
fn BRepGraph_ReplaceVertexTest__StartVertex_SwappedToFreshVertex_AuditClean() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());
    assert!(aGraph.topo().edges().nb() > 0);

    let anEdgeId = EdgeId::new(0);
    let anOldStartRefId = aGraph.topo().edges().definition(anEdgeId).start_vertex_ref_id;
    assert!(anOldStartRefId.is_valid());

    let aNewVertex = aGraph
        .editor()
        .vertices()
        .add(Point3::new(99.0, 99.0, 99.0), 1.0e-7);
    assert!(aNewVertex.is_valid());

    let aNewRefId = replaceVertex(&mut aGraph, anEdgeId, anOldStartRefId, aNewVertex);
    assert!(aNewRefId.is_valid());
    assert_ne!(aNewRefId, anOldStartRefId);

    // Edge slot must now point at the new ref.
    assert_eq!(
        aGraph.topo().edges().definition(anEdgeId).start_vertex_ref_id,
        aNewRefId
    );

    // Old ref is retired, new ref points at the replacement vertex.
    assert!(anOldStartRefId.is_removed(&aGraph));
    assert_eq!(
        aGraph.refs().vertices().entry(aNewRefId).child_vertex_id,
        aNewVertex
    );

    // Lightweight only; a single-edge replacement breaks wire connectivity.
    assert!(isLightweightValid(&aGraph));
}

#[test]
fn BRepGraph_ReplaceVertexTest__EndVertex_SwappedToFreshVertex_PreservesOrientation() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let anEdgeId = EdgeId::new(0);
    let anOldEndRefId = aGraph.topo().edges().definition(anEdgeId).end_vertex_ref_id;
    assert!(anOldEndRefId.is_valid());
    let aExpectedOri = aGraph.refs().vertices().entry(anOldEndRefId).orientation;

    let aNewVertex = aGraph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 2.0, 3.0), 1.0e-7);
    let aNewRefId = replaceVertex(&mut aGraph, anEdgeId, anOldEndRefId, aNewVertex);
    assert!(aNewRefId.is_valid());

    assert_eq!(
        aGraph.refs().vertices().entry(aNewRefId).orientation,
        aExpectedOri,
        "ReplaceVertex must preserve orientation of the retired ref"
    );
    // Lightweight validate covers structural invariants (relation tables, active
    // counts) but skips wire-connectivity. A single-edge vertex replacement
    // deliberately breaks wire connectivity at the shared corner; the full
    // Audit would reject it. Callers that want a connectivity-safe replace
    // must update every edge at that corner in the same mutation batch.
    assert!(isLightweightValid(&aGraph));
}

#[test]
fn BRepGraph_ReplaceVertexTest__SameVertex_Idempotent() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let anEdgeId = EdgeId::new(0);
    let anOldStartRefId = aGraph.topo().edges().definition(anEdgeId).start_vertex_ref_id;
    let aSameVertex = aGraph.refs().vertices().entry(anOldStartRefId).child_vertex_id;

    let aResult = replaceVertex(&mut aGraph, anEdgeId, anOldStartRefId, aSameVertex);
    assert_eq!(
        aResult, anOldStartRefId,
        "Replacing with the same vertex must be a no-op returning the same ref"
    );
    assert!(!anOldStartRefId.is_removed(&aGraph));
}

#[test]
fn BRepGraph_ReplaceVertexTest__InactiveEdge_Rejected() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let anEdgeId = EdgeId::new(0);
    let anOldRefId = aGraph.topo().edges().definition(anEdgeId).start_vertex_ref_id;

    let aNewVertex = aGraph
        .editor()
        .vertices()
        .add(Point3::new(5.0, 5.0, 5.0), 1.0e-7);

    aGraph
        .editor()
        .generic()
        .remove_node(NodeId::from(anEdgeId))
        .unwrap();

    let aResult = replaceVertex(&mut aGraph, anEdgeId, anOldRefId, aNewVertex);
    assert!(
        !aResult.is_valid(),
        "Replacing on a removed edge must return an invalid ref id"
    );
}

#[test]
fn BRepGraph_ReplaceVertexTest__WrongParent_Rejected() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());
    assert!(aGraph.topo().edges().nb() > 1);

    // Vertex ref from edge 1 passed to an API call targeting edge 0 must fail.
    let anEdge0 = EdgeId::new(0);
    let aEdge1StartRefId = aGraph
        .topo()
        .edges()
        .definition(EdgeId::new(1))
        .start_vertex_ref_id;
    assert!(aEdge1StartRefId.is_valid());

    let aNewVertex = aGraph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 100.0), 1.0e-7);

    let aResult = replaceVertex(&mut aGraph, anEdge0, aEdge1StartRefId, aNewVertex);
    assert!(
        !aResult.is_valid(),
        "ReplaceVertex must refuse a ref that is not in the requested edge slots"
    );
}

#[test]
fn BRepGraph_ReplaceVertexTest__Relations_Rebuilt_VertexToEdge() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let anEdgeId = EdgeId::new(0);
    let anOldRefId = aGraph.topo().edges().definition(anEdgeId).start_vertex_ref_id;
    let aOldVertexId = aGraph.refs().vertices().entry(anOldRefId).child_vertex_id;

    let aNewVertex = aGraph
        .editor()
        .vertices()
        .add(Point3::new(1000.0, 0.0, 0.0), 1.0e-7);

    assert!(replaceVertex(&mut aGraph, anEdgeId, anOldRefId, aNewVertex).is_valid());

    // New vertex must now see anEdgeId in its vertex-to-edge reverse map.
    let aNewEdges = aGraph.topo().vertices().edges(aNewVertex);
    let aFoundNew = aNewEdges.iter().any(|anE| *anE == anEdgeId);
    assert!(
        aFoundNew,
        "Vertex->Edge relation tables must include the replacement edge"
    );

    // Old vertex may still be referenced by other edges of the box (shared
    // corners); but it should no longer see anEdgeId through this slot.
    let aOldEdges = aGraph.topo().vertices().edges(aOldVertexId);
    let aFoundOld = aOldEdges.iter().any(|anE| *anE == anEdgeId);
    assert!(
        !aFoundOld,
        "Vertex->Edge relation tables must drop the replaced edge"
    );

    // Lightweight validate covers structural invariants (relation tables, active
    // counts) but skips wire-connectivity. A single-edge vertex replacement
    // deliberately breaks wire connectivity at the shared corner; the full
    // Audit would reject it. Callers that want a connectivity-safe replace
    // must update every edge at that corner in the same mutation batch.
    assert!(isLightweightValid(&aGraph));
}
