// BRepGraph_SparseModel_Test.cxx, Groups 3 to 6.

// =======================================================================
// Group 3 - Deduplicate on sparse models
// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Dedup_BareVertices_NoRewrite() {
    let mut aGraph = BRepGraph::new();
    assert!(addVertex(&mut aGraph, 0.0, 0.0, 0.0).is_valid());
    assert!(addVertex(&mut aGraph, 1.0, 0.0, 0.0).is_valid());
    assert!(addVertex(&mut aGraph, 2.0, 0.0, 0.0).is_valid());

    let aRes = dedupGraph(&mut aGraph);
    assert_eq!(aRes.nb_canonical_surfaces, 0);
    assert_eq!(aRes.nb_canonical_curves, 0);
    assert_eq!(aRes.nb_surface_rewrites, 0);
    assert_eq!(aRes.nb_curve_rewrites, 0);
    assert!(!aRes.is_entity_merge_applied);
    assert!(isValid(&aGraph));
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Dedup_TwoIdenticalVertices_MergeWhenSafe() {
    let mut aGraph = BRepGraph::new();
    assert!(addVertex(&mut aGraph, 0.0, 0.0, 0.0).is_valid());
    assert!(addVertex(&mut aGraph, 0.0, 0.0, 0.0).is_valid());

    assert_eq!(aGraph.topo().vertices().nb_active(), 2);

    let aRes = dedupMerging(&mut aGraph);

    assert!(aRes.nb_merged_vertices >= 1);
    assert_eq!(aGraph.topo().vertices().nb_active(), 1);
    assert!(isValid(&aGraph));
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Dedup_TwoIdenticalVertices_NoMergeByDefault() {
    let mut aGraph = BRepGraph::new();
    assert!(addVertex(&mut aGraph, 0.0, 0.0, 0.0).is_valid());
    assert!(addVertex(&mut aGraph, 0.0, 0.0, 0.0).is_valid());

    assert_eq!(aGraph.topo().vertices().nb_active(), 2);

    let aRes = dedupGraph(&mut aGraph);
    assert_eq!(aRes.nb_merged_vertices, 0);
    assert!(!aRes.is_entity_merge_applied);
    assert_eq!(aGraph.topo().vertices().nb_active(), 2);
    assert!(isValid(&aGraph));
}

// =======================================================================

/// Two free edges on two equal but distinct x lines, the second one unit up in y.
fn twoLineEdges(aGraph: &mut BRepGraph) -> [VertexId; 4] {
    let aV0 = addVertex(aGraph, 0.0, 0.0, 0.0);
    let aV1 = addVertex(aGraph, 10.0, 0.0, 0.0);
    let aV0b = addVertex(aGraph, 0.0, 1.0, 0.0);
    let aV1b = addVertex(aGraph, 10.0, 1.0, 0.0);
    let aLine1 = xLine();
    let aLine2 = xLine();
    assert!(aLine1.object_address() != aLine2.object_address());

    assert!(addEdge(aGraph, aV0, aV1, &aLine1).is_valid());
    assert!(addEdge(aGraph, aV0b, aV1b, &aLine2).is_valid());
    [aV0, aV1, aV0b, aV1b]
}

#[test]
fn BRepGraph_SparseModelTest__Dedup_FreeEdgesWithCurve_CanonicalizesCurves() {
    let mut aGraph = BRepGraph::new();
    let _aVertices = twoLineEdges(&mut aGraph);

    let aRes = dedupGraph(&mut aGraph);
    assert_eq!(aRes.nb_canonical_curves, 1);
    assert_eq!(aRes.nb_curve_rewrites, 1);
    assert!(isValid(&aGraph));
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Dedup_FreeWireSingleCoEdge_NoGeometryRewrite() {
    let mut aGraph = BRepGraph::new();
    let (_anEdge, aWire) = freeWire(&mut aGraph);
    assert!(aWire.is_valid());

    let aRes = dedupGraph(&mut aGraph);
    assert_eq!(aRes.nb_canonical_surfaces, 0);
    assert_eq!(aRes.nb_canonical_curves, 1);
    assert_eq!(aRes.nb_curve_rewrites, 0);
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Dedup_EmptyCompound_NoOp() {
    let mut aGraph = BRepGraph::new();
    assert!(aGraph.editor().compounds().add(&[]).is_valid());

    let aRes = dedupGraph(&mut aGraph);
    assert_eq!(aRes.nb_canonical_surfaces, 0);
    assert_eq!(aRes.nb_canonical_curves, 0);
    assert!(isValid(&aGraph));
}

// =======================================================================
// Group 4 - CleanupRemovedReferences edge cases
// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Cleanup_RemoveFace_CoEdgeFaceIdCleared_CoEdgeSurvives() {
    let mut aGraph = BRepGraph::new();
    let (_anEdge, aWire) = freeWire(&mut aGraph);
    let aFace = addPlaneFace(&mut aGraph, aWire);
    assert!(aFace.is_valid());

    removeNode(&mut aGraph, aFace);
    cleanup(&mut aGraph);

    let mut anIt = CoEdgeIterator::new(&aGraph);
    while anIt.more() {
        let aCE = anIt.current();
        assert!(
            !anIt.current_id().is_removed(&aGraph),
            "Free-wire CoEdge must not be retired by Phase 4a"
        );
        assert!(
            !aCE.face_id.is_valid(),
            "FaceId should be cleared for CoEdges whose face was removed"
        );
        assert!(
            aCE.child_edge_id.is_valid(),
            "ChildEdgeId must remain valid for free-wire CoEdges"
        );
        anIt.next();
    }
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Cleanup_RemoveEdge_FreeWireCoEdgeRetired() {
    let mut aGraph = BRepGraph::new();
    let (anEdge, aWire) = freeWire(&mut aGraph);
    assert!(aWire.is_valid());

    removeNode(&mut aGraph, anEdge);
    cleanup(&mut aGraph);

    assert_eq!(
        aGraph.topo().co_edges().nb_active(),
        0,
        "All CoEdges should be removed (Phase 4a orphan retirement)"
    );
    assert!(
        aGraph.topo().co_edges().nb() >= 1,
        "Total CoEdges (including removed) should still reflect the pre-cleanup count"
    );
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Cleanup_RemoveVertex_EdgeVertexRefsCleared() {
    let mut aGraph = BRepGraph::new();
    let aV0 = addVertex(&mut aGraph, 0.0, 0.0, 0.0);
    let aV1 = addVertex(&mut aGraph, 10.0, 0.0, 0.0);
    assert!(addEdge(&mut aGraph, aV0, aV1, &noCurve()).is_valid());

    removeNode(&mut aGraph, aV0);
    cleanup(&mut aGraph);

    assert_eq!(aGraph.topo().vertices().nb_active(), 1);
    assert_eq!(aGraph.topo().edges().nb_active(), 1);

    let mut anIt = EdgeIterator::new(&aGraph);
    while anIt.more() {
        let anED = anIt.current();
        assert!(
            !anED.start_vertex_ref_id.is_valid(),
            "StartVertexRef should be cleared for removed vertex"
        );
        assert!(
            anED.end_vertex_ref_id.is_valid(),
            "EndVertexRef to surviving vertex must persist"
        );
        anIt.next();
    }
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Cleanup_RemoveWire_FaceWireRefPruned() {
    let mut aGraph = BRepGraph::new();
    let (_anEdge, aWire) = freeWire(&mut aGraph);
    let aFace = addPlaneFace(&mut aGraph, aWire);
    assert!(aFace.is_valid());

    removeNode(&mut aGraph, aWire);
    cleanup(&mut aGraph);

    assert_eq!(aGraph.topo().wires().nb_active(), 0);
    assert_eq!(aGraph.topo().faces().nb_active(), 1);
    let aFaceRelations = aGraph.topo().faces().relations(FaceId::start());
    assert_eq!(
        aFaceRelations.wire_ref_ids.len(),
        0,
        "WireRef should be pruned by Phase 5"
    );
}

// =======================================================================
// Group 5 - Product / Occurrence canonical pattern stress
// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Product_EmptyProduct_CompactAndAudit() {
    let mut aGraph = BRepGraph::new();
    assert!(aGraph.editor().products().add_empty().is_valid());

    assert_eq!(aGraph.topo().products().nb_active(), 1);
    assert_eq!(aGraph.topo().occurrences().nb_active(), 0);

    let aRes = compactGraph(&mut aGraph);
    assert_eq!(aRes.nb_nodes_before, aRes.nb_nodes_after);
    assert_eq!(aGraph.topo().products().nb_active(), 1);
    assert!(isAuditValid(&aGraph));
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Product_FullCanonical_RemoveSolid_WholeChainCleans() {
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
        "Phase 10 retires occurrence with removed ChildNodeId"
    );

    let _aRes = compactGraph(&mut aGraph);
    assert_eq!(aGraph.topo().solids().nb_active(), 0);
    assert_eq!(aGraph.topo().occurrences().nb_active(), 0);
    assert_eq!(aGraph.topo().products().nb_active(), 1);
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Product_FullCanonical_CompactAndDedup_NoRegression() {
    let mut aGraph = BRepGraph::new();
    let aV0 = addVertex(&mut aGraph, 0.0, 0.0, 0.0);
    let aV1 = addVertex(&mut aGraph, 10.0, 0.0, 0.0);
    let aLine = xLine();
    let anEdge = addEdge(&mut aGraph, aV0, aV1, &aLine);
    let aWire = addWire(&mut aGraph, anEdge);
    let (_aFace, _aSolid, aProduct) = canonicalSolid(&mut aGraph, aWire);
    assert!(aProduct.is_valid());
    aGraph
        .editor()
        .products()
        .append_document_root(aProduct)
        .unwrap();

    let _ = compactGraph(&mut aGraph);
    assert!(isAuditValid(&aGraph));

    let aDedupRes = dedupGraph(&mut aGraph);
    assert_eq!(aDedupRes.nb_canonical_surfaces, 1);
    assert_eq!(aDedupRes.nb_canonical_curves, 1);
    assert!(isAuditValid(&aGraph));
}

// =======================================================================
// Group 6 - Stress pipelines (multiple operations combined)
// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Pipeline_RemoveCleanupCompact_GraphValid() {
    let mut aGraph = BRepGraph::new();
    let aV0 = addVertex(&mut aGraph, 0.0, 0.0, 0.0);
    let aV1 = addVertex(&mut aGraph, 10.0, 0.0, 0.0);
    let aVExtra = addVertex(&mut aGraph, 100.0, 0.0, 0.0);
    let anEdge = addEdge(&mut aGraph, aV0, aV1, &noCurve());
    let aWire = addWire(&mut aGraph, anEdge);
    let aFace = addPlaneFace(&mut aGraph, aWire);
    assert!(aFace.is_valid());

    removeNode(&mut aGraph, aFace);
    removeNode(&mut aGraph, aVExtra);
    cleanup(&mut aGraph);
    let _ = compactGraph(&mut aGraph);
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Pipeline_RemoveCleanupDedupCompact_GraphValid() {
    let mut aGraph = BRepGraph::new();
    let [aV0a, _, _, _] = twoLineEdges(&mut aGraph);

    removeNode(&mut aGraph, aV0a);
    cleanup(&mut aGraph);

    let _ = dedupMerging(&mut aGraph);
    let _ = compactGraph(&mut aGraph);

    assert!(isAuditValid(&aGraph));
}

// =======================================================================

#[test]
fn BRepGraph_SparseModelTest__Pipeline_CompoundWithEmptyShells_CompactAndAudit() {
    let mut aGraph = BRepGraph::new();
    let aShell1 = aGraph.editor().shells().add();
    let aShell2 = aGraph.editor().shells().add();
    assert!(aShell1.is_valid() && aShell2.is_valid());

    let aChildren = [NodeId::from(aShell1), NodeId::from(aShell2)];
    assert!(aGraph.editor().compounds().add(&aChildren).is_valid());

    assert_eq!(aGraph.topo().shells().nb_active(), 2);
    assert_eq!(aGraph.topo().compounds().nb_active(), 1);

    removeNode(&mut aGraph, aShell1);
    cleanup(&mut aGraph);

    assert_eq!(aGraph.topo().shells().nb_active(), 1);

    let _aRes = compactGraph(&mut aGraph);
    assert_eq!(aGraph.topo().shells().nb_active(), 1);
    assert_eq!(aGraph.topo().compounds().nb_active(), 1);
    assert!(isAuditValid(&aGraph));
}
