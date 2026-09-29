// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.

//! Direct translation of BRepGraph_CacheMesh_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported).
//!
//! Regression coverage for BRepGraph_CacheMesh freshness: verifies that a
//! cached triangulation becomes stale when the owning Face's OwnGen bumps,
//! whether the bump comes from a direct FaceDef mutation or from a face-owned
//! surface use mutation.
//!
//! The cache reads return entry copies, so `nullptr` is `None`; a guarded setter is
//! the `*_guarded` associated function (docs/topology-design.md section 11.4), and
//! `Needs(theGraph, theRoot)` passes its default slot explicitly.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep_graph::cache::BRepGraphCache;
use rustycad_tkbrep::brep_graph::cache_mesh::{CacheMesh, DEFAULT_DISPLAY_SLOT};
use rustycad_tkbrep::brep_graph::editor::{CoEdgeOps, EdgeOps, FaceOps, VertexOps};
use rustycad_tkbrep::brep_graph::iterator::{EdgeIterator, FaceIterator, VertexIterator};
use rustycad_tkbrep::brep_graph::refs_iterator::{CoEdgesOfWire, RefsWireOfFace};
use rustycad_tkbrep::brep_graph::{BRepGraph, CoEdgeId, EdgeId, FaceId, NodeId, VertexId};
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};
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

fn newTriangulation(theNbNodes: i32) -> Handle<Triangulation> {
    Handle::new(Triangulation::with_sizes(theNbNodes, 1, false, false).unwrap())
}

fn makeTrivialTriangulation() -> Handle<Triangulation> {
    newTriangulation(3)
}

fn firstFaceId(theGraph: &BRepGraph) -> FaceId {
    let aFaceIt = FaceIterator::new(theGraph);
    if aFaceIt.more() {
        aFaceIt.current_id()
    } else {
        FaceId::invalid()
    }
}

fn firstEdgeId(theGraph: &BRepGraph) -> EdgeId {
    let anEdgeIt = EdgeIterator::new(theGraph);
    if anEdgeIt.more() {
        anEdgeIt.current_id()
    } else {
        EdgeId::invalid()
    }
}

fn firstVertexId(theGraph: &BRepGraph) -> VertexId {
    let aVtxIt = VertexIterator::new(theGraph);
    if aVtxIt.more() {
        aVtxIt.current_id()
    } else {
        VertexId::invalid()
    }
}

fn firstCoEdgeOfFace(theGraph: &BRepGraph, theFaceId: FaceId) -> CoEdgeId {
    let mut aWireRefIt = RefsWireOfFace::new(theGraph, theFaceId);
    while aWireRefIt.more() {
        let aWireId = theGraph
            .refs()
            .wires()
            .entry(aWireRefIt.current_id())
            .child_wire_id;
        let mut aCoEdgeIt = CoEdgesOfWire::new(theGraph, aWireId);
        while aCoEdgeIt.more() {
            let aCoEdgeId = aCoEdgeIt.current_id();
            if theGraph.topo().co_edges().definition(aCoEdgeId).face_id == theFaceId {
                return aCoEdgeId;
            }
            aCoEdgeIt.next();
        }
        aWireRefIt.next();
    }
    CoEdgeId::invalid()
}

fn writeFaceMesh(theGraph: &mut BRepGraph, theFaceId: FaceId) {
    let aTri = makeTrivialTriangulation();
    theGraph
        .editor()
        .faces()
        .set_persistent_triangulation(theFaceId, &aTri)
        .unwrap();
    theGraph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(theFaceId, &aTri)
        .unwrap();
}

fn writeCoEdgeMesh(theGraph: &mut BRepGraph, theCoEdgeId: CoEdgeId, theFaceId: FaceId) {
    let aPoly2D = Handle::new(Polygon2D::new(2).unwrap());
    let anEditor = theGraph.mesh().editor();
    anEditor
        .co_edges()
        .set_cached_polygon2d(theCoEdgeId, &aPoly2D)
        .unwrap();
    let aTri = makeTrivialTriangulation();
    anEditor
        .faces()
        .set_cached_triangulation(theFaceId, &aTri)
        .unwrap();
    let aPolyOnTri = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    anEditor
        .co_edges()
        .append_cached_polygon_on_tri(theCoEdgeId, &aPolyOnTri)
        .unwrap();
}

fn writeEdgeMesh(theGraph: &mut BRepGraph, theEdgeId: EdgeId) {
    let aPoly3D = Handle::new(Polygon3D::new(2, false).unwrap());
    theGraph
        .mesh()
        .editor()
        .edges()
        .set_cached_polygon3d(theEdgeId, &aPoly3D)
        .unwrap();
}

/// `aGraph.CacheRegistry().Find<BRepGraph_CacheMesh>()`.
fn cacheMesh(theGraph: &BRepGraph) -> Handle<CacheMesh> {
    theGraph.cache_registry().find::<CacheMesh>()
}

/// The `{ MutGuard aGuard = Faces().Mut(theFace); SetTolerance(aGuard, +1e-6); }` scope.
fn bumpFaceTolerance(theGraph: &mut BRepGraph, theFaceId: FaceId) {
    let mut aGuard = theGraph.editor().faces().mutate(theFaceId).unwrap();
    let aTolerance = aGuard.tolerance + 1.0e-6;
    FaceOps::set_tolerance_guarded(&mut aGuard, aTolerance);
}

fn bumpEdgeTolerance(theGraph: &mut BRepGraph, theEdgeId: EdgeId) {
    let mut aGuard = theGraph.editor().edges().mutate(theEdgeId).unwrap();
    let aTolerance = aGuard.tolerance + 1.0e-6;
    EdgeOps::set_tolerance_guarded(&mut aGuard, aTolerance);
}

fn bumpVertexTolerance(theGraph: &mut BRepGraph, theVertexId: VertexId) {
    let mut aGuard = theGraph.editor().vertices().mutate(theVertexId).unwrap();
    let aTolerance = aGuard.tolerance + 1.0e-6;
    VertexOps::set_tolerance_guarded(&mut aGuard, aTolerance);
}

fn reverseCoEdge(theGraph: &mut BRepGraph, theCoEdgeId: CoEdgeId) {
    let mut aGuard = theGraph.editor().co_edges().mutate(theCoEdgeId).unwrap();
    CoEdgeOps::set_orientation_guarded(&mut aGuard, ParityOrientation::REVERSED);
}

/// The common prologue: the first face and its first coedge, both valid.
fn faceAndCoEdge(aGraph: &BRepGraph) -> (FaceId, CoEdgeId) {
    let aFaceId = firstFaceId(aGraph);
    assert!(aFaceId.is_valid_for(aGraph.topo().faces().nb()));
    let aCoEdgeId = firstCoEdgeOfFace(aGraph, aFaceId);
    assert!(aCoEdgeId.is_valid_for(aGraph.topo().co_edges().nb()));
    (aFaceId, aCoEdgeId)
}

#[test]
fn BRepGraph_CacheMeshTest__CacheStaleAfterFaceMutation() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aFaceId = firstFaceId(&aGraph);
    assert!(aFaceId.is_valid_for(aGraph.topo().faces().nb()));

    let aTri = makeTrivialTriangulation();
    aGraph
        .editor()
        .faces()
        .set_persistent_triangulation(aFaceId, &aTri)
        .unwrap();

    aGraph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(aFaceId, &aTri)
        .unwrap();

    let aBefore = aGraph.mesh().cache().faces().entry(aFaceId);
    assert!(
        aBefore.is_some(),
        "CachedMesh must be present immediately after write"
    );

    bumpFaceTolerance(&mut aGraph, aFaceId);

    let aAfter = aGraph.mesh().cache().faces().entry(aFaceId);
    assert!(
        aAfter.is_none(),
        "CachedMesh must become null (stale) after Face Mut bumps OwnGen"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__CacheStaleAfterSurfaceRepMutation() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aFaceId = firstFaceId(&aGraph);
    assert!(aFaceId.is_valid_for(aGraph.topo().faces().nb()));

    let aTri = makeTrivialTriangulation();
    aGraph
        .editor()
        .faces()
        .set_persistent_triangulation(aFaceId, &aTri)
        .unwrap();

    aGraph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(aFaceId, &aTri)
        .unwrap();

    assert!(aGraph.mesh().cache().faces().entry(aFaceId).is_some());

    aGraph.editor().faces().clear_surface(aFaceId).unwrap();

    assert!(
        aGraph.mesh().cache().faces().entry(aFaceId).is_none(),
        "CachedMesh must become null after SurfaceRep Set propagates to Face OwnGen"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__CacheSurvivesUnrelatedMutation() {
    let mut aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aFaceId = firstFaceId(&aGraph);
    assert!(aFaceId.is_valid_for(aGraph.topo().faces().nb()));

    let mut anOther = FaceIterator::new(&aGraph);
    anOther.next();
    assert!(anOther.more(), "Box should have >1 face for this test");
    let anOtherFaceId = anOther.current_id();

    let aTri = makeTrivialTriangulation();
    aGraph
        .editor()
        .faces()
        .set_persistent_triangulation(aFaceId, &aTri)
        .unwrap();

    aGraph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(aFaceId, &aTri)
        .unwrap();

    bumpFaceTolerance(&mut aGraph, anOtherFaceId);

    assert!(
        aGraph.mesh().cache().faces().entry(aFaceId).is_some(),
        "Unrelated face mutation must not invalidate this face's cache"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__ClearDropsLargeCacheAndAllowsSmallRegenerate() {
    let aGraph = makeBoxGraph();
    assert!(!aGraph.is_empty());

    let aFaceId = firstFaceId(&aGraph);
    assert!(aFaceId.is_valid_for(aGraph.topo().faces().nb()));

    let aCheckTri = newTriangulation(3);
    aGraph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(aFaceId, &aCheckTri)
        .unwrap();
    assert!(aGraph.mesh().cache().faces().entry(aFaceId).is_some());

    aGraph.cache_registry().clear_all();
    assert!(aGraph.mesh().cache().faces().entry(aFaceId).is_none());

    aGraph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(aFaceId, &aCheckTri)
        .unwrap();
    assert!(aGraph.mesh().cache().faces().entry(aFaceId).is_some());
    assert!(
        aGraph
            .mesh()
            .cache()
            .faces()
            .entry(aFaceId)
            .unwrap()
            .triangulation
            == aCheckTri
    );
}

// ---- Step 15: New tests for CacheMesh freshness redesign ----

#[test]
fn BRepGraph_CacheMeshTest__FaceCache_StaleAfterEdgeMutation() {
    let mut aGraph = makeBoxGraph();
    let aFaceId = firstFaceId(&aGraph);
    assert!(aFaceId.is_valid_for(aGraph.topo().faces().nb()));

    writeFaceMesh(&mut aGraph, aFaceId);
    assert!(aGraph.mesh().cache().faces().entry(aFaceId).is_some());

    let anEdgeId = firstEdgeId(&aGraph);
    assert!(anEdgeId.is_valid_for(aGraph.topo().edges().nb()));

    bumpEdgeTolerance(&mut aGraph, anEdgeId);

    assert!(
        aGraph.mesh().cache().faces().entry(aFaceId).is_none(),
        "Face cache must become stale after edge mutation (SubtreeGen propagation)"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__FaceCache_StaleAfterCoEdgeMutation() {
    let mut aGraph = makeBoxGraph();
    let (aFaceId, aCoEdgeId) = faceAndCoEdge(&aGraph);

    writeFaceMesh(&mut aGraph, aFaceId);
    assert!(aGraph.mesh().cache().faces().entry(aFaceId).is_some());

    reverseCoEdge(&mut aGraph, aCoEdgeId);

    assert!(
        aGraph.mesh().cache().faces().entry(aFaceId).is_none(),
        "Face cache must become stale after coedge mutation (SubtreeGen propagation)"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__FaceCache_StaleAfterVertexMutation() {
    let mut aGraph = makeBoxGraph();
    let aFaceId = firstFaceId(&aGraph);
    assert!(aFaceId.is_valid_for(aGraph.topo().faces().nb()));

    let aVtxId = firstVertexId(&aGraph);
    assert!(aVtxId.is_valid_for(aGraph.topo().vertices().nb()));

    writeFaceMesh(&mut aGraph, aFaceId);
    assert!(aGraph.mesh().cache().faces().entry(aFaceId).is_some());

    bumpVertexTolerance(&mut aGraph, aVtxId);

    assert!(
        aGraph.mesh().cache().faces().entry(aFaceId).is_none(),
        "Face cache must become stale after vertex mutation (SubtreeGen propagation)"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__EdgeCache_StaleAfterVertexMutation() {
    let mut aGraph = makeBoxGraph();
    let anEdgeId = firstEdgeId(&aGraph);
    assert!(anEdgeId.is_valid_for(aGraph.topo().edges().nb()));

    let aVtxId = firstVertexId(&aGraph);
    assert!(aVtxId.is_valid_for(aGraph.topo().vertices().nb()));

    writeEdgeMesh(&mut aGraph, anEdgeId);
    assert!(aGraph.mesh().cache().edges().entry(anEdgeId).is_some());

    bumpVertexTolerance(&mut aGraph, aVtxId);

    assert!(
        aGraph.mesh().cache().edges().entry(anEdgeId).is_none(),
        "Edge cache must become stale after vertex mutation"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__EdgeCache_FreshAfterUnrelatedFaceMutation() {
    let mut aGraph = makeBoxGraph();
    let anEdgeId = firstEdgeId(&aGraph);
    assert!(anEdgeId.is_valid_for(aGraph.topo().edges().nb()));

    writeEdgeMesh(&mut aGraph, anEdgeId);
    assert!(aGraph.mesh().cache().edges().entry(anEdgeId).is_some());

    let aFaceId = firstFaceId(&aGraph);
    bumpFaceTolerance(&mut aGraph, aFaceId);

    assert!(
        aGraph.mesh().cache().edges().entry(anEdgeId).is_some(),
        "Edge cache must survive unrelated face mutation"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__CoEdge_Polygon2DFreshAfterFaceMeshChange() {
    let mut aGraph = makeBoxGraph();
    let (aFaceId, aCoEdgeId) = faceAndCoEdge(&aGraph);

    writeCoEdgeMesh(&mut aGraph, aCoEdgeId, aFaceId);
    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon2d(&aGraph, aCoEdgeId)
            .is_some()
    );

    let aNewTri = newTriangulation(4);
    aGraph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(aFaceId, &aNewTri)
        .unwrap();

    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon2d(&aGraph, aCoEdgeId)
            .is_some(),
        "Polygon2D must stay fresh when only face mesh content changes"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__CoEdge_PolygonOnTriStaleAfterFaceMeshChange() {
    let mut aGraph = makeBoxGraph();
    let (aFaceId, aCoEdgeId) = faceAndCoEdge(&aGraph);

    writeCoEdgeMesh(&mut aGraph, aCoEdgeId, aFaceId);
    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon_on_tri(&aGraph, aCoEdgeId)
            .is_some()
    );

    let aNewTri = newTriangulation(4);
    aGraph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(aFaceId, &aNewTri)
        .unwrap();

    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon_on_tri(&aGraph, aCoEdgeId)
            .is_none(),
        "PolygonOnTri must become stale when face mesh content changes"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__CoEdge_PolygonOnTriStaleAfterFaceTopologyChange() {
    let mut aGraph = makeBoxGraph();
    let (aFaceId, aCoEdgeId) = faceAndCoEdge(&aGraph);

    writeCoEdgeMesh(&mut aGraph, aCoEdgeId, aFaceId);
    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon_on_tri(&aGraph, aCoEdgeId)
            .is_some()
    );

    bumpFaceTolerance(&mut aGraph, aFaceId);

    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon_on_tri(&aGraph, aCoEdgeId)
            .is_none(),
        "PolygonOnTri must become stale when face topology changes"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__CoEdge_PolygonOnTriStaleAfterCoEdgeMutation() {
    let mut aGraph = makeBoxGraph();
    let (aFaceId, aCoEdgeId) = faceAndCoEdge(&aGraph);

    writeCoEdgeMesh(&mut aGraph, aCoEdgeId, aFaceId);
    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon_on_tri(&aGraph, aCoEdgeId)
            .is_some()
    );

    reverseCoEdge(&mut aGraph, aCoEdgeId);

    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon_on_tri(&aGraph, aCoEdgeId)
            .is_none(),
        "PolygonOnTri must become stale when coedge topology changes"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__CoEdge_FreshAfterUnrelatedEdgeMutation() {
    let mut aGraph = makeBoxGraph();
    let (aFaceId, aCoEdgeId) = faceAndCoEdge(&aGraph);

    writeCoEdgeMesh(&mut aGraph, aCoEdgeId, aFaceId);
    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon2d(&aGraph, aCoEdgeId)
            .is_some()
    );
    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon_on_tri(&aGraph, aCoEdgeId)
            .is_some()
    );

    let mut anEdgeIt = EdgeIterator::new(&aGraph);
    anEdgeIt.next();
    let anOtherEdgeId = anEdgeIt.current_id();
    if anOtherEdgeId.is_valid_for(aGraph.topo().edges().nb()) {
        bumpEdgeTolerance(&mut aGraph, anOtherEdgeId);
    }

    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon2d(&aGraph, aCoEdgeId)
            .is_some(),
        "Polygon2D must survive unrelated edge mutation"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__CoEdge_UsesCoEdgeDefFaceId() {
    let mut aGraph = makeBoxGraph();
    let (aFaceId, aCoEdgeId) = faceAndCoEdge(&aGraph);

    writeCoEdgeMesh(&mut aGraph, aCoEdgeId, aFaceId);
    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon_on_tri(&aGraph, aCoEdgeId)
            .is_some()
    );

    let aDefFaceId = aGraph.topo().co_edges().definition(aCoEdgeId).face_id;
    assert!(aDefFaceId.is_valid_for(aGraph.topo().faces().nb()));

    bumpFaceTolerance(&mut aGraph, aDefFaceId);

    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon_on_tri(&aGraph, aCoEdgeId)
            .is_none(),
        "Coedge freshness must use CoEdgeDef::FaceId for face binding"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__CoEdge_Polygon2DStaleAfterSlotRecipeChange() {
    let mut aGraph = makeBoxGraph();
    let (aFaceId, aCoEdgeId) = faceAndCoEdge(&aGraph);

    writeCoEdgeMesh(&mut aGraph, aCoEdgeId, aFaceId);
    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon2d(&aGraph, aCoEdgeId)
            .is_some()
    );

    cacheMesh(&aGraph).clear();

    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon2d(&aGraph, aCoEdgeId)
            .is_none(),
        "Polygon2D must become stale after cache clear (SlotGeneration bump)"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__CoEdge_FaceNotYetMeshed_SnapshotZero() {
    let mut aGraph = makeBoxGraph();
    let (aFaceId, aCoEdgeId) = faceAndCoEdge(&aGraph);

    let aPoly2D = Handle::new(Polygon2D::new(2).unwrap());
    aGraph
        .mesh()
        .editor()
        .co_edges()
        .set_cached_polygon2d(aCoEdgeId, &aPoly2D)
        .unwrap();
    let aPolyOnTri = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    aGraph
        .mesh()
        .editor()
        .co_edges()
        .append_cached_polygon_on_tri(aCoEdgeId, &aPolyOnTri)
        .unwrap();

    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon_on_tri(&aGraph, aCoEdgeId)
            .is_none(),
        "PolygonOnTri must be stale when face has never been meshed (snapshot=0 vs default=0)"
    );

    writeFaceMesh(&mut aGraph, aFaceId);

    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon_on_tri(&aGraph, aCoEdgeId)
            .is_none(),
        "PolygonOnTri must remain stale after face is meshed (MeshGeneration bumped to 1)"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__Needs_DetectsChildDrivenFaceStaleness() {
    let mut aGraph = makeBoxGraph();
    let aFaceId = firstFaceId(&aGraph);
    assert!(aFaceId.is_valid_for(aGraph.topo().faces().nb()));

    writeFaceMesh(&mut aGraph, aFaceId);
    assert!(
        !cacheMesh(&aGraph)
            .needs(&aGraph, NodeId::from(aFaceId), DEFAULT_DISPLAY_SLOT)
            .unwrap()
    );

    let aVtxId = firstVertexId(&aGraph);
    assert!(aVtxId.is_valid_for(aGraph.topo().vertices().nb()));
    bumpVertexTolerance(&mut aGraph, aVtxId);

    assert!(
        cacheMesh(&aGraph)
            .needs(&aGraph, NodeId::from(aFaceId), DEFAULT_DISPLAY_SLOT)
            .unwrap(),
        "Needs(face) must detect staleness driven by child vertex mutation"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__Needs_DetectsStaleCoedgeOnTri() {
    let mut aGraph = makeBoxGraph();
    let (aFaceId, aCoEdgeId) = faceAndCoEdge(&aGraph);

    writeCoEdgeMesh(&mut aGraph, aCoEdgeId, aFaceId);
    assert!(
        !cacheMesh(&aGraph)
            .needs(&aGraph, NodeId::from(aFaceId), DEFAULT_DISPLAY_SLOT)
            .unwrap()
    );

    let aNewTri = newTriangulation(4);
    aGraph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(aFaceId, &aNewTri)
        .unwrap();

    assert!(
        cacheMesh(&aGraph)
            .needs(&aGraph, NodeId::from(aFaceId), DEFAULT_DISPLAY_SLOT)
            .unwrap(),
        "Needs(face) must detect coedge PolygonOnTri staleness from face mesh change"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__VertexPropagation_ReachesFaces() {
    let mut aGraph = makeBoxGraph();
    let aFaceId = firstFaceId(&aGraph);
    assert!(aFaceId.is_valid_for(aGraph.topo().faces().nb()));

    let aVtxId = firstVertexId(&aGraph);
    assert!(aVtxId.is_valid_for(aGraph.topo().vertices().nb()));

    writeFaceMesh(&mut aGraph, aFaceId);
    assert!(aGraph.mesh().cache().faces().entry(aFaceId).is_some());

    bumpVertexTolerance(&mut aGraph, aVtxId);

    assert!(
        aGraph.mesh().cache().faces().entry(aFaceId).is_none(),
        "Vertex mutation must propagate through edge -> wire -> face chain"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__MeshGeneration_MonotonicAcrossClear() {
    let mut aGraph = makeBoxGraph();
    let (aFaceId, aCoEdgeId) = faceAndCoEdge(&aGraph);

    writeCoEdgeMesh(&mut aGraph, aCoEdgeId, aFaceId);
    let aCoEntry = cacheMesh(&aGraph).find_co_edge_polygon_on_tri(&aGraph, aCoEdgeId);
    assert!(aCoEntry.is_some());

    aGraph.mesh().editor().faces().clear(aFaceId).unwrap();
    writeFaceMesh(&mut aGraph, aFaceId);

    let aAfterEntry = cacheMesh(&aGraph).find_co_edge_entry_raw(DEFAULT_DISPLAY_SLOT, aCoEdgeId);
    assert!(aAfterEntry.is_some());
    let aFaceEntry = cacheMesh(&aGraph).find_face_entry_raw(DEFAULT_DISPLAY_SLOT, aFaceId);
    assert!(aFaceEntry.is_some());
    assert!(
        aFaceEntry.unwrap().mesh_generation > aAfterEntry.unwrap().face_mesh_generation,
        "Face MeshGeneration must exceed coedge snapshot after clear+rewrite"
    );
}

#[test]
fn BRepGraph_CacheMeshTest__FaceClear_PreservesCoEdgePolygon2D() {
    let mut aGraph = makeBoxGraph();
    let (aFaceId, aCoEdgeId) = faceAndCoEdge(&aGraph);

    writeCoEdgeMesh(&mut aGraph, aCoEdgeId, aFaceId);
    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon2d(&aGraph, aCoEdgeId)
            .is_some()
    );

    aGraph.mesh().editor().faces().clear(aFaceId).unwrap();

    assert!(
        cacheMesh(&aGraph)
            .find_co_edge_polygon2d(&aGraph, aCoEdgeId)
            .is_some(),
        "FaceOps::Clear must not destroy coedge Polygon2D"
    );
}
