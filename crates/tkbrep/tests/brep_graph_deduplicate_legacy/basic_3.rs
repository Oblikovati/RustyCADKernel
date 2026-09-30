#[test]
fn BRepGraph_DeduplicateTest__HistoryRecords_MergePhases() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    let anOpts = Options {
        merge_entities_when_safe: true,
        history_mode: true,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    let aNbVertexMerge = countHistoryRecordsByOp(&aGraph, "Dedup:MergeVertex");
    let aNbEdgeMerge = countHistoryRecordsByOp(&aGraph, "Dedup:MergeEdge");
    let aNbWireMerge = countHistoryRecordsByOp(&aGraph, "Dedup:MergeWire");
    let aNbFaceMerge = countHistoryRecordsByOp(&aGraph, "Dedup:MergeFace");
    assert_eq!(aNbVertexMerge, aRes.nb_merged_vertices);
    assert_eq!(aNbEdgeMerge, aRes.nb_merged_edges);
    assert_eq!(aNbWireMerge, aRes.nb_merged_wires);
    assert_eq!(aNbFaceMerge, aRes.nb_merged_faces);
}

#[test]
fn BRepGraph_DeduplicateTest__AfterMerge_Validate_NoIssues() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    let anOpts = Options {
        merge_entities_when_safe: true,
        ..Options::default()
    };
    let _ = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    // After merge + compact, graph should be structurally valid.
    // Merge alone may leave stale back-references on geometry nodes
    // that are cleaned up by compaction.
    let _ = compact::perform(&mut aGraph).unwrap();
    let aValResult = validate::perform(&aGraph);
    assert!(aValResult.is_valid());
}

#[test]
fn BRepGraph_DeduplicateTest__EdgeMerge_ThreeCopiedEdges_GraphValidates() {
    let mut aGraph = dedup_graph(&makeThreeCopiedIdenticalEdges());
    assert!(!aGraph.is_empty());
    assert_eq!(aGraph.topo().edges().nb(), 3);
    let anOpts = Options {
        merge_entities_when_safe: true,
        ..Options::default()
    };
    let aDedupRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    assert_eq!(aDedupRes.nb_merged_edges, 2);
    let _ = compact::perform(&mut aGraph).unwrap();
    let aVal = validate::perform(&aGraph);
    assert!(aVal.is_valid());
}
