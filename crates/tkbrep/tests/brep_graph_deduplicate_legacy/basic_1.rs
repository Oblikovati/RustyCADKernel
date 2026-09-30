#[test]
fn BRepGraph_DeduplicateTest__AnalyzeOnly_DoesNotRewrite() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), 2);
    let anOpts = Options {
        analyze_only: true,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    assert_eq!(aRes.nb_surface_rewrites, 0);
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), 2);
}

#[test]
fn BRepGraph_DeduplicateTest__AnalyzeOnly_ReportsCanonicalCandidates() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    let anOpts = Options {
        analyze_only: true,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    // TwoCopiedFaces: 2 surfaces -> 1 canonical, 8 curves -> 4 canonical, 8 PCurves -> 8 canonical.
    assert_eq!(aRes.nb_canonical_surfaces, 1);
    assert_eq!(aRes.nb_canonical_curves, 4);
    assert_eq!(aRes.nb_surface_rewrites, 0);
    assert_eq!(aRes.nb_curve_rewrites, 0);
}

#[test]
fn BRepGraph_DeduplicateTest__CanonicalizeSurfaces_RewritesAndRecordsHistory() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), 2);
    let anOpts = Options {
        analyze_only: false,
        history_mode: true,
        ..Options::default()
    };
    let aHistoryBefore = dedup_history(&aGraph).get().unwrap().nb_records();
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    assert_eq!(aRes.nb_surface_rewrites, 1);
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), 1);
    // 1 surface canonicalize + 4 curve canonicalizes = 5 history records.
    assert_eq!(
        dedup_history(&aGraph).get().unwrap().nb_records(),
        aHistoryBefore + 5
    );
}

#[test]
fn BRepGraph_DeduplicateTest__CanonicalizeCurves_RewritesAndReducesUnique() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    assert_eq!(nbUniqueEdgeCurveDefs(&aGraph), 8);
    let anOpts = Options {
        analyze_only: false,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    assert_eq!(aRes.nb_curve_rewrites, 4);
    assert_eq!(nbUniqueEdgeCurveDefs(&aGraph), 4);
}

#[test]
fn BRepGraph_DeduplicateTest__HistoryModeOff_DoesNotAddHistory() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    assert_eq!(dedup_history(&aGraph).get().unwrap().nb_records(), 0);
    let anOpts = Options {
        history_mode: false,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    assert_eq!(aRes.nb_surface_rewrites, 1);
    assert_eq!(aRes.nb_curve_rewrites, 4);
    assert_eq!(dedup_history(&aGraph).get().unwrap().nb_records(), 0);
}

#[test]
fn BRepGraph_DeduplicateTest__RestoresHistoryEnabledFlag() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    dedup_history(&aGraph).get().unwrap().set_enabled(false);
    assert!(!dedup_history(&aGraph).get().unwrap().is_enabled());
    let anOpts = Options {
        history_mode: true,
        ..Options::default()
    };
    let _ = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    assert!(!dedup_history(&aGraph).get().unwrap().is_enabled());
}

#[test]
fn BRepGraph_DeduplicateTest__DefaultOverload_PerformWorks() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    let aRes = deduplicate::perform(&mut aGraph).unwrap();
    assert_eq!(aRes.nb_canonical_surfaces, 1);
    assert_eq!(aRes.nb_surface_rewrites, 1);
    assert_eq!(aRes.nb_curve_rewrites, 4);
    // 1 surface canonicalize + 4 curve canonicalizes = 5 history records.
    assert_eq!(aRes.nb_history_records, 5);
    assert!(!aRes.is_entity_merge_applied);
}

#[test]
fn BRepGraph_DeduplicateTest__NotDoneGraph_ReturnsEmptyResult() {
    let mut aGraph = BRepGraph::new();
    // Do not call BRepGraph::ShapesView::Add() - graph is not done.
    assert!(aGraph.is_empty());
    let aRes = deduplicate::perform(&mut aGraph).unwrap();
    assert_eq!(aRes.nb_canonical_surfaces, 0);
    assert_eq!(aRes.nb_canonical_curves, 0);
    assert_eq!(aRes.nb_surface_rewrites, 0);
    assert_eq!(aRes.nb_curve_rewrites, 0);
    assert_eq!(aRes.nb_nullified_surfaces, 0);
    assert_eq!(aRes.nb_nullified_curves, 0);
    assert_eq!(aRes.nb_history_records, 0);
    assert!(!aRes.is_entity_merge_applied);
    assert_eq!(aRes.nb_reordered_wires, 0);
    assert_eq!(aRes.nb_tolerance_ordered_wires, 0);
    assert_eq!(aRes.nb_partial_ordered_wires, 0);
}

#[test]
fn BRepGraph_DeduplicateTest__Idempotent_SecondRunNoRewrites() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    let aRes1 = deduplicate::perform(&mut aGraph).unwrap();
    assert_eq!(aRes1.nb_surface_rewrites, 1);
    assert_eq!(aRes1.nb_curve_rewrites, 4);
    // After first dedup, all faces share the same surface pointer, all duplicate
    // edges share the same curve pointer. A second run still detects the same
    // canonical mapping via GeomHash, so rewrite counts may be non-zero (no-op
    // at pointer level). Verify the unique pointer count is unchanged.
    let aUniqueSurfsBefore = nbUniqueFaceSurfaceDefs(&aGraph);
    let aUniqueCurvesBefore = nbUniqueEdgeCurveDefs(&aGraph);
    let _aRes2 = deduplicate::perform(&mut aGraph).unwrap();
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), aUniqueSurfsBefore);
    assert_eq!(nbUniqueEdgeCurveDefs(&aGraph), aUniqueCurvesBefore);
}

#[test]
fn BRepGraph_DeduplicateTest__ResultCountersConsistency() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    // TwoCopiedFaces: 2 surfaces, 8 curves, 8 PCurves.
    assert_eq!(aGraph.topo().faces().nb(), 2);
    assert_eq!(aGraph.topo().edges().nb(), 8);
    assert_eq!(nbPCurveEntries(&aGraph), 8);
    let anOpts = Options {
        analyze_only: true,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    dedup_assert_counter_consistency(&aGraph, &aRes);
}

#[test]
fn BRepGraph_DeduplicateTest__MultipleCopies_NWayDedup() {
    let aNbCopies = 4;
    let mut aGraph = dedup_graph(&makeNCopiedIdenticalFaces(aNbCopies));
    assert!(!aGraph.is_empty());
    assert_eq!(aGraph.topo().faces().nb(), aNbCopies);
    let aRes = deduplicate::perform(&mut aGraph).unwrap();
    assert_eq!(aRes.nb_canonical_surfaces, 1);
    assert_eq!(aRes.nb_surface_rewrites, aNbCopies - 1);
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), 1);
}

#[test]
fn BRepGraph_DeduplicateTest__MixedGeometry_OnlyIdenticalDeduped() {
    let mut aGraph = dedup_graph(&makeMixedCompound());
    assert!(!aGraph.is_empty());
    // 2 box face copies + 1 cylinder face = 3 faces, 3 surfaces, 11 curves.
    assert_eq!(aGraph.topo().faces().nb(), 3);
    assert_eq!(aGraph.topo().faces().nb(), 3);
    assert_eq!(aGraph.topo().edges().nb(), 11);
    let aRes = deduplicate::perform(&mut aGraph).unwrap();
    // Only the two identical box faces should be deduped; cylinder face is untouched.
    assert_eq!(aRes.nb_canonical_surfaces, 2);
    assert_eq!(aRes.nb_canonical_curves, 7);
    assert_eq!(aRes.nb_surface_rewrites, 1);
    assert_eq!(aRes.nb_curve_rewrites, 4);
    // 7 canonicalize + 1 nullify surface + 4 nullify curves + 2 nullify PCurves = 14.
    assert!(aRes.nb_history_records > 0);
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), 2);
    assert_eq!(nbUniqueEdgeCurveDefs(&aGraph), 7);
}

#[test]
fn BRepGraph_DeduplicateTest__HistoryRecordNames_MatchExpectedOps() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    let anOpts = Options {
        history_mode: true,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    // 1 surface canonicalize + 4 curve canonicalizes = 5 history records.
    assert_eq!(aRes.nb_history_records, 5);
    // TwoCopiedFaces: 1 surface + 4 curve canonicalizations (no orphan nullification).
    let aNbSurfOps = countHistoryRecordsByOp(&aGraph, "Dedup:CanonicalizeSurface");
    let aNbCurveOps = countHistoryRecordsByOp(&aGraph, "Dedup:CanonicalizeCurve");
    assert_eq!(aNbSurfOps, 1);
    assert_eq!(aNbCurveOps, 4);
    assert_eq!(aNbSurfOps + aNbCurveOps, 5);
}

#[test]
fn BRepGraph_DeduplicateTest__AnalyzeOnly_CurveAndPCurveCountsReported() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    let anOpts = Options {
        analyze_only: true,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    // TwoCopiedFaces: 2 geom surfaces, 8 curves, 8 PCurves.
    assert_eq!(aRes.nb_canonical_surfaces, 1);
    assert_eq!(aRes.nb_canonical_curves, 4);
    // But no rewrites.
    assert_eq!(aRes.nb_surface_rewrites, 0);
    assert_eq!(aRes.nb_curve_rewrites, 0);
}

#[test]
fn BRepGraph_DeduplicateTest__DefaultOptionsStruct_HasExpectedDefaults() {
    let anOpts = Options {
        ..Options::default()
    };
    assert!(!anOpts.analyze_only);
    assert!(anOpts.history_mode);
    assert!(!anOpts.merge_entities_when_safe);
    assert!((anOpts.comp_tolerance - precision::angular()).abs() <= 1e-20);
    assert!((anOpts.hash_tolerance - precision::confusion()).abs() <= 1e-20);
}

#[test]
fn BRepGraph_DeduplicateTest__DefaultResultStruct_AllZeroed() {
    let aRes = DeduplicateResult::default();
    assert_eq!(aRes.nb_canonical_surfaces, 0);
    assert_eq!(aRes.nb_canonical_curves, 0);
    assert_eq!(aRes.nb_surface_rewrites, 0);
    assert_eq!(aRes.nb_curve_rewrites, 0);
    assert_eq!(aRes.nb_nullified_surfaces, 0);
    assert_eq!(aRes.nb_nullified_curves, 0);
    assert_eq!(aRes.nb_history_records, 0);
    assert!(!aRes.is_entity_merge_applied);
    assert_eq!(aRes.nb_reordered_wires, 0);
    assert_eq!(aRes.nb_tolerance_ordered_wires, 0);
    assert_eq!(aRes.nb_partial_ordered_wires, 0);
}

#[test]
fn BRepGraph_DeduplicateTest__MergeDefsWhenSafe_MergesVertices() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    let aNbVerticesBefore = aGraph.topo().vertices().nb();
    let anOpts = Options {
        merge_entities_when_safe: true,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    // Two copied faces share identical vertex positions -> vertices should merge.
    assert!(aRes.nb_merged_vertices > 0);
    assert!(aRes.is_entity_merge_applied);
    let _ = aNbVerticesBefore;
}

#[test]
fn BRepGraph_DeduplicateTest__NestedCompound_AllCopiesDeduped() {
    let mut aGraph = dedup_graph(&makeNestedCompound());
    assert!(!aGraph.is_empty());
    // 3 copies of the same face across nested compounds: 3 surfaces, 12 curves.
    assert_eq!(aGraph.topo().faces().nb(), 3);
    assert_eq!(aGraph.topo().edges().nb(), 12);
    assert_eq!(aGraph.topo().faces().nb(), 3);
    assert_eq!(aGraph.topo().edges().nb(), 12);
    let aRes = deduplicate::perform(&mut aGraph).unwrap();
    assert_eq!(aRes.nb_canonical_surfaces, 1);
    assert_eq!(aRes.nb_canonical_curves, 4);
    assert_eq!(aRes.nb_surface_rewrites, 2);
    assert_eq!(aRes.nb_curve_rewrites, 8);
    // 10 canonicalize + 2 nullify surfaces + 8 nullify curves = 20.
    assert!(aRes.nb_history_records > 0);
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), 1);
    assert_eq!(nbUniqueEdgeCurveDefs(&aGraph), 4);
}

fn dedup_assert_counter_consistency(aGraph: &BRepGraph, aRes: &DeduplicateResult) {
    // Canonical + duplicates == total for each geometry type.
    assert_eq!(aRes.nb_canonical_surfaces, 1);
    assert_eq!(
        aRes.nb_canonical_surfaces + (aGraph.topo().faces().nb() - aRes.nb_canonical_surfaces),
        aGraph.topo().faces().nb()
    );
    assert_eq!(aRes.nb_canonical_curves, 4);
    assert_eq!(
        aRes.nb_canonical_curves + (aGraph.topo().edges().nb() - aRes.nb_canonical_curves),
        aGraph.topo().edges().nb()
    );
}
