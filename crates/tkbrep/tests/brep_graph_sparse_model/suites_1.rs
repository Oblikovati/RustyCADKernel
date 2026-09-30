// BRepGraph_SparseModel_Test.cxx, Groups 1 and 2.

// =======================================================================
// Group 1 - Compact on sparse models (no removed nodes -> no-op or valid pass)
// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Compact_BareVertices_NoOp() {
    let mut aGraph = BRepGraph::new();
    assert!(addVertex(&mut aGraph, 0.0, 0.0, 0.0).is_valid());
    assert!(addVertex(&mut aGraph, 1.0, 0.0, 0.0).is_valid());
    assert!(addVertex(&mut aGraph, 2.0, 0.0, 0.0).is_valid());

    let aVtxBefore = aGraph.topo().vertices().nb_active();
    assert_eq!(aVtxBefore, 3);

    let aRes = compactGraph(&mut aGraph);
    assert_eq!(aRes.nb_nodes_before, aRes.nb_nodes_after);
    assert_eq!(aGraph.topo().vertices().nb_active(), 3);
    let aValidateResult = validate::perform(&aGraph);
    assert!(
        aValidateResult.is_valid(),
        "{:?}",
        aValidateResult.issues.first().map(|anIssue| &anIssue.description)
    );
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Compact_FreeEdgeNoCurve_NoOp() {
    let mut aGraph = BRepGraph::new();
    let aV0 = addVertex(&mut aGraph, 0.0, 0.0, 0.0);
    let aV1 = addVertex(&mut aGraph, 10.0, 0.0, 0.0);
    assert!(aV0.is_valid() && aV1.is_valid());
    let anEdge = addEdge(&mut aGraph, aV0, aV1, &noCurve());
    assert!(anEdge.is_valid());

    assert_eq!(aGraph.topo().vertices().nb_active(), 2);
    assert_eq!(aGraph.topo().edges().nb_active(), 1);

    let aRes = compactGraph(&mut aGraph);
    assert_eq!(aRes.nb_nodes_before, aRes.nb_nodes_after);
    assert_eq!(aGraph.topo().vertices().nb_active(), 2);
    assert_eq!(aGraph.topo().edges().nb_active(), 1);
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Compact_FreeWireSingleCoEdge_NoOp() {
    let mut aGraph = BRepGraph::new();
    let aV0 = addVertex(&mut aGraph, 0.0, 0.0, 0.0);
    let aV1 = addVertex(&mut aGraph, 10.0, 0.0, 0.0);
    assert!(aV0.is_valid() && aV1.is_valid());
    let anEdge = addEdge(&mut aGraph, aV0, aV1, &noCurve());
    assert!(anEdge.is_valid());

    assert!(addWire(&mut aGraph, anEdge).is_valid());

    assert_eq!(aGraph.topo().vertices().nb_active(), 2);
    assert_eq!(aGraph.topo().edges().nb_active(), 1);
    assert_eq!(aGraph.topo().wires().nb_active(), 1);
    assert!(aGraph.topo().co_edges().nb_active() >= 1);

    let aRes = compactGraph(&mut aGraph);
    assert_eq!(aRes.nb_nodes_before, aRes.nb_nodes_after);
    assert_eq!(aGraph.topo().vertices().nb_active(), 2);
    assert_eq!(aGraph.topo().edges().nb_active(), 1);
    assert_eq!(aGraph.topo().wires().nb_active(), 1);
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Compact_EmptyCompound_NoOp() {
    let mut aGraph = BRepGraph::new();
    assert!(aGraph.editor().compounds().add(&[]).is_valid());

    assert_eq!(aGraph.topo().compounds().nb_active(), 1);

    let aRes = compactGraph(&mut aGraph);
    assert_eq!(aRes.nb_nodes_before, aRes.nb_nodes_after);
    assert_eq!(aGraph.topo().compounds().nb_active(), 1);
    assert!(isValid(&aGraph));
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Compact_EmptyShell_NoOp() {
    let mut aGraph = BRepGraph::new();
    assert!(aGraph.editor().shells().add().is_valid());

    assert_eq!(aGraph.topo().shells().nb_active(), 1);

    let aRes = compactGraph(&mut aGraph);
    assert_eq!(aRes.nb_nodes_before, aRes.nb_nodes_after);
    assert_eq!(aGraph.topo().shells().nb_active(), 1);
    assert!(isValid(&aGraph));
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Compact_EmptySolid_NoOp() {
    let mut aGraph = BRepGraph::new();
    assert!(aGraph.editor().solids().add().is_valid());

    assert_eq!(aGraph.topo().solids().nb_active(), 1);

    let aRes = compactGraph(&mut aGraph);
    assert_eq!(aRes.nb_nodes_before, aRes.nb_nodes_after);
    assert_eq!(aGraph.topo().solids().nb_active(), 1);
    assert!(isValid(&aGraph));
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Compact_WirelessSurfacelessFace_NoOp() {
    let mut aGraph = BRepGraph::new();
    let aNoSurface: Handle<dyn GeomSurface> = Handle::default();
    let aFace = aGraph
        .editor()
        .faces()
        .add(&aNoSurface, WireId::default(), &[], 1.0e-7);
    assert!(aFace.is_valid());

    assert_eq!(aGraph.topo().faces().nb_active(), 1);

    let aRes = compactGraph(&mut aGraph);
    assert_eq!(aRes.nb_nodes_before, aRes.nb_nodes_after);
    assert_eq!(aGraph.topo().faces().nb_active(), 1);
    assert!(isValid(&aGraph));
}

// =======================================================================
// Group 2 - Compact after RemoveNode on sparse models
// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Compact_RemoveVertexFromFreeEdge_RemapsEdgeRefs() {
    let mut aGraph = BRepGraph::new();
    let aV0 = addVertex(&mut aGraph, 0.0, 0.0, 0.0);
    let aV1 = addVertex(&mut aGraph, 10.0, 0.0, 0.0);
    assert!(aV0.is_valid() && aV1.is_valid());
    let aRvEdge = addEdge(&mut aGraph, aV0, aV1, &noCurve());
    assert!(aRvEdge.is_valid());

    assert_eq!(aGraph.topo().vertices().nb_active(), 2);
    assert_eq!(aGraph.topo().edges().nb_active(), 1);

    removeNode(&mut aGraph, aV0);
    cleanup(&mut aGraph);

    assert_eq!(aGraph.topo().vertices().nb_active(), 1);
    assert_eq!(aGraph.topo().edges().nb_active(), 1);

    let _aRes = compactGraph(&mut aGraph);
    assert_eq!(aGraph.topo().vertices().nb_active(), 1);
    assert_eq!(aGraph.topo().edges().nb_active(), 1);
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Compact_RemoveEdgeFromFreeWire_RetiresCoEdgeAndPrunesRef() {
    let mut aGraph = BRepGraph::new();
    let (anEdge, aWire) = freeWire(&mut aGraph);
    assert!(anEdge.is_valid());
    assert!(aWire.is_valid());

    let aCoEdgesBefore = aGraph.topo().co_edges().nb_active();
    assert!(aCoEdgesBefore >= 1);
    assert_eq!(aGraph.topo().wires().nb_active(), 1);
    assert_eq!(aGraph.topo().edges().nb_active(), 1);

    removeNode(&mut aGraph, anEdge);
    cleanup(&mut aGraph);

    assert_eq!(aGraph.topo().edges().nb_active(), 0);
    assert_eq!(
        aGraph.topo().co_edges().nb_active(),
        aCoEdgesBefore - 1,
        "Phase 4a should retire orphaned CoEdge"
    );

    let _aRes = compactGraph(&mut aGraph);
    assert_eq!(aGraph.topo().co_edges().nb_active(), 0);
    assert_eq!(aGraph.topo().edges().nb_active(), 0);
    assert_eq!(aGraph.topo().wires().nb_active(), 1);
    assert_eq!(aGraph.topo().vertices().nb_active(), 2);
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Compact_RemoveFace_FreeWireCoEdgesSurvive() {
    let mut aGraph = BRepGraph::new();
    let (_anEdge, aWire) = freeWire(&mut aGraph);
    let aFace = addPlaneFace(&mut aGraph, aWire);
    assert!(aFace.is_valid());

    let aCoEdgesBefore = aGraph.topo().co_edges().nb_active();
    let aWiresBefore = aGraph.topo().wires().nb_active();
    assert!(aCoEdgesBefore >= 1);
    assert_eq!(aWiresBefore, 1);
    assert_eq!(aGraph.topo().faces().nb_active(), 1);

    removeNode(&mut aGraph, aFace);
    cleanup(&mut aGraph);

    assert_eq!(aGraph.topo().faces().nb_active(), 0);
    assert_eq!(
        aGraph.topo().co_edges().nb_active(),
        aCoEdgesBefore,
        "Free-wire CoEdges must survive face removal (FaceId cleared, ChildEdgeId valid)"
    );
    assert_eq!(
        aGraph.topo().wires().nb_active(),
        aWiresBefore,
        "Wire must survive face removal"
    );

    let _aRes = compactGraph(&mut aGraph);
    assert_eq!(aGraph.topo().faces().nb_active(), 0);
    assert_eq!(aGraph.topo().co_edges().nb_active(), aCoEdgesBefore);
    assert_eq!(aGraph.topo().wires().nb_active(), aWiresBefore);
    assert_eq!(aGraph.topo().edges().nb_active(), 1);
    assert_eq!(aGraph.topo().vertices().nb_active(), 2);
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Compact_RemoveTopologyChild_OccurrenceRetired() {
    let mut aGraph = BRepGraph::new();
    let (_anEdge, aWire) = freeWire(&mut aGraph);
    let (_aFace, aSolid, aProduct) = canonicalSolid(&mut aGraph, aWire);
    assert!(aProduct.is_valid());
    aGraph
        .editor()
        .products()
        .append_document_root(aProduct)
        .unwrap();

    assert_eq!(aGraph.topo().products().nb_active(), 1);
    assert_eq!(aGraph.topo().occurrences().nb_active(), 1);
    assert_eq!(aGraph.topo().solids().nb_active(), 1);

    removeNode(&mut aGraph, aSolid);
    cleanup(&mut aGraph);

    assert_eq!(aGraph.topo().solids().nb_active(), 0);
    assert_eq!(
        aGraph.topo().occurrences().nb_active(),
        0,
        "Phase 10 should retire occurrence whose ChildNodeId was removed"
    );
    assert_eq!(aGraph.topo().products().nb_active(), 1);

    let _aRes = compactGraph(&mut aGraph);
    assert_eq!(aGraph.topo().solids().nb_active(), 0);
    assert_eq!(aGraph.topo().occurrences().nb_active(), 0);
    assert_eq!(aGraph.topo().products().nb_active(), 1);
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Compact_MultipleRemovals_VertexEdgeFace_Validates() {
    let mut aGraph = BRepGraph::new();

    let aExtraVtx = addVertex(&mut aGraph, 100.0, 0.0, 0.0);
    assert!(aExtraVtx.is_valid());

    let aV0f = addVertex(&mut aGraph, 0.0, 0.0, 0.0);
    let aV1f = addVertex(&mut aGraph, 10.0, 0.0, 0.0);
    let aFreeEdge = addEdge(&mut aGraph, aV0f, aV1f, &noCurve());

    let aV0 = addVertex(&mut aGraph, 0.0, 1.0, 0.0);
    let aV1 = addVertex(&mut aGraph, 10.0, 1.0, 0.0);
    let anEdge = addEdge(&mut aGraph, aV0, aV1, &noCurve());
    let aWire = addWire(&mut aGraph, anEdge);
    let aFace = addPlaneFace(&mut aGraph, aWire);
    assert!(aFace.is_valid());
    assert!(aFreeEdge.is_valid());

    let aVtxBefore = aGraph.topo().vertices().nb_active();

    removeNode(&mut aGraph, aExtraVtx);
    removeNode(&mut aGraph, aFreeEdge);
    removeNode(&mut aGraph, aFace);
    cleanup(&mut aGraph);

    assert_eq!(aGraph.topo().vertices().nb_active(), aVtxBefore - 1);
    assert_eq!(aGraph.topo().edges().nb_active(), 1);
    assert_eq!(aGraph.topo().faces().nb_active(), 0);

    let _aRes = compactGraph(&mut aGraph);
}
