#[test]
fn BRepGraph_DeduplicateTest__ThreeDistinctPrimitives_MinimalDedup() {
    let mut aGraph = dedup_graph(&makeThreeDistinctPrimitives());
    assert!(!aGraph.is_empty());
    // Box(6 faces) + Sphere(1 face) + Cone(3 faces) = 10 faces, 18 edges.
    assert_eq!(aGraph.topo().faces().nb(), 10);
    assert_eq!(aGraph.topo().edges().nb(), 18);
    let aRes = deduplicate::perform(&mut aGraph).unwrap();
    // 9 canonical surfaces (1 surface matches between cone bottom and sphere/box plane).
    assert_eq!(aRes.nb_canonical_surfaces, 9);
    assert_eq!(aRes.nb_surface_rewrites, 1);
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), 9);
    // Some curve dedup may occur between the three primitives (shared circle/line geometry).
    assert!(nbUniqueEdgeCurveDefs(&aGraph) <= 18);
}

#[test]
fn BRepGraph_DeduplicateTest__TwoIdenticalBoxes_SurfacesAndCurvesDeduped() {
    let mut aGraph = dedup_graph(&makeTwoIdenticalBoxes());
    assert!(!aGraph.is_empty());
    // Two identical boxes: 12 faces, 24 edges, 12 geom surfaces, 24 geom curves.
    assert_eq!(aGraph.topo().faces().nb(), 12);
    assert_eq!(aGraph.topo().edges().nb(), 24);
    assert_eq!(aGraph.topo().faces().nb(), 12);
    assert_eq!(aGraph.topo().edges().nb(), 24);
    let aRes = deduplicate::perform(&mut aGraph).unwrap();
    assert_eq!(aRes.nb_canonical_surfaces, 6);
    assert_eq!(aRes.nb_canonical_curves, 12);
    assert_eq!(aRes.nb_surface_rewrites, 6);
    assert_eq!(aRes.nb_curve_rewrites, 12);
    // 18 canonicalize + 6 nullify surfaces + 12 nullify curves = 36.
    assert!(aRes.nb_history_records > 0);
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), 6);
    assert_eq!(nbUniqueEdgeCurveDefs(&aGraph), 12);
}

#[test]
fn BRepGraph_DeduplicateTest__CurveRewriteCount_MatchesDuplicateEdgeCurves() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    // TwoCopiedFaces: 8 geom curves, 4 canonical -> 4 duplicates.
    assert_eq!(aGraph.topo().edges().nb(), 8);
    let anAnalyze = Options {
        analyze_only: true,
        ..Options::default()
    };
    let anAnalysis = deduplicate::perform_with(&mut aGraph, &anAnalyze).unwrap();
    assert_eq!(anAnalysis.nb_canonical_curves, 4);
    let aRes = deduplicate::perform(&mut aGraph).unwrap();
    // 4 duplicate curve nodes -> exactly 4 edge curve rewrites.
    assert_eq!(aRes.nb_curve_rewrites, 4);
}

#[test]
fn BRepGraph_DeduplicateTest__AfterDedup_AllCopiedEdgesShareCanonicalCurve() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    assert_eq!(nbUniqueEdgeCurveDefs(&aGraph), 8);
    let _ = deduplicate::perform(&mut aGraph).unwrap();
    // After dedup: 8 unique curves reduced to 4 canonical.
    assert_eq!(nbUniqueEdgeCurveDefs(&aGraph), 4);
}

#[test]
fn BRepGraph_DeduplicateTest__HistoryOff_NbRecordsUnchanged() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    // Run once with history to get 5 records (1 surface + 4 curve canonicalizes).
    let anOpts1 = Options {
        history_mode: true,
        ..Options::default()
    };
    let _ = deduplicate::perform_with(&mut aGraph, &anOpts1).unwrap();
    assert_eq!(dedup_history(&aGraph).get().unwrap().nb_records(), 5);
    dedup_assert_fresh_history_off();
}

#[test]
fn BRepGraph_DeduplicateTest__TenCopies_AllDeduplicatedToOneSurface() {
    let aNbCopies = 10;
    let mut aGraph = dedup_graph(&makeNCopiedIdenticalFaces(aNbCopies));
    assert!(!aGraph.is_empty());
    assert_eq!(aGraph.topo().faces().nb(), aNbCopies);
    let aRes = deduplicate::perform(&mut aGraph).unwrap();
    assert_eq!(aRes.nb_canonical_surfaces, 1);
    assert_eq!(aRes.nb_surface_rewrites, aNbCopies - 1);
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), 1);
}

#[test]
fn BRepGraph_DeduplicateTest__TwoCopies_CurveCanonicalCountLessThanTotal() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    // 8 geom curves, 4 canonical.
    assert_eq!(aGraph.topo().edges().nb(), 8);
    let anOpts = Options {
        analyze_only: true,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    assert_eq!(aRes.nb_canonical_curves, 4);
}

#[test]
fn BRepGraph_DeduplicateTest__Idempotent_MixedCompound_SurfacesAndCurves() {
    let mut aGraph = dedup_graph(&makeMixedCompound());
    assert!(!aGraph.is_empty());
    let aRes1 = deduplicate::perform(&mut aGraph).unwrap();
    assert_eq!(aRes1.nb_surface_rewrites, 1);
    assert_eq!(aRes1.nb_curve_rewrites, 4);
    // After first dedup, unique pointer counts are stable.
    let aUniqueSurfs = nbUniqueFaceSurfaceDefs(&aGraph);
    let aUniqueCurves = nbUniqueEdgeCurveDefs(&aGraph);
    let _aRes2 = deduplicate::perform(&mut aGraph).unwrap();
    // Unique pointer counts unchanged after second run.
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), aUniqueSurfs);
    assert_eq!(nbUniqueEdgeCurveDefs(&aGraph), aUniqueCurves);
}

#[test]
fn BRepGraph_DeduplicateTest__Idempotent_TwoIdenticalBoxes() {
    let mut aGraph = dedup_graph(&makeTwoIdenticalBoxes());
    assert!(!aGraph.is_empty());
    let aRes1 = deduplicate::perform(&mut aGraph).unwrap();
    assert_eq!(aRes1.nb_surface_rewrites, 6);
    assert_eq!(aRes1.nb_curve_rewrites, 12);
    // After first dedup, unique pointer counts are stable.
    let aUniqueSurfs = nbUniqueFaceSurfaceDefs(&aGraph);
    let aUniqueCurves = nbUniqueEdgeCurveDefs(&aGraph);
    let _aRes2 = deduplicate::perform(&mut aGraph).unwrap();
    // Unique pointer counts unchanged after second run.
    assert_eq!(nbUniqueFaceSurfaceDefs(&aGraph), aUniqueSurfs);
    assert_eq!(nbUniqueEdgeCurveDefs(&aGraph), aUniqueCurves);
}

#[test]
fn BRepGraph_DeduplicateTest__RestoresHistoryFlag_WhenHistoryModeOff() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    dedup_history(&aGraph).get().unwrap().set_enabled(true);
    assert!(dedup_history(&aGraph).get().unwrap().is_enabled());
    let anOpts = Options {
        history_mode: false,
        ..Options::default()
    };
    let _ = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    // Should be restored to the original value (true).
    assert!(dedup_history(&aGraph).get().unwrap().is_enabled());
}

#[test]
fn BRepGraph_DeduplicateTest__RestoresHistoryFlag_AnalyzeOnlyPath() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    dedup_history(&aGraph).get().unwrap().set_enabled(true);
    let anOpts = Options {
        analyze_only: true,
        history_mode: false,
        ..Options::default()
    };
    let _ = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    // Restored even when exiting through the AnalyzeOnly early-return.
    assert!(dedup_history(&aGraph).get().unwrap().is_enabled());
}

#[test]
fn BRepGraph_DeduplicateTest__GeomCountsUnchanged_AfterDedup() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    // TwoCopiedFaces: 2 surfaces, 8 curves, 8 PCurves.
    assert_eq!(aGraph.topo().faces().nb(), 2);
    assert_eq!(aGraph.topo().edges().nb(), 8);
    assert_eq!(nbPCurveEntries(&aGraph), 8);
    let _ = deduplicate::perform(&mut aGraph).unwrap();
    // Dedup rewrites references, but does not remove geometry nodes.
    assert_eq!(aGraph.topo().faces().nb(), 2);
    assert_eq!(aGraph.topo().edges().nb(), 8);
    assert_eq!(nbPCurveEntries(&aGraph), 8);
}

#[test]
fn BRepGraph_DeduplicateTest__DefCountsUnchanged_AfterDedup() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    // TwoCopiedFaces: 2 face defs, 8 edge defs.
    assert_eq!(aGraph.topo().faces().nb(), 2);
    assert_eq!(aGraph.topo().edges().nb(), 8);
    let _ = deduplicate::perform(&mut aGraph).unwrap();
    // Dedup never removes or adds defs.
    assert_eq!(aGraph.topo().faces().nb(), 2);
    assert_eq!(aGraph.topo().edges().nb(), 8);
}

#[test]
fn BRepGraph_DeduplicateTest__MergeEdges_SharedEdgesReduced() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    let anOpts = Options {
        merge_entities_when_safe: true,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    assert!(aRes.nb_merged_edges > 0);
}

// Original OCCT assertion compares an unsigned counter with zero; preserve it verbatim.
#[allow(clippy::absurd_extreme_comparisons, unused_comparisons)]
#[test]
fn BRepGraph_DeduplicateTest__MergeWires_IdenticalWiresMerged() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    let anOpts = Options {
        merge_entities_when_safe: true,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    // After vertex and edge merge, identical wires should also merge.
    assert!(aRes.nb_merged_wires >= 0);
}

// Original OCCT assertion compares an unsigned counter with zero; preserve it verbatim.
#[allow(clippy::absurd_extreme_comparisons, unused_comparisons)]
#[test]
fn BRepGraph_DeduplicateTest__MergeFaces_IdenticalFacesMerged() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    let anOpts = Options {
        merge_entities_when_safe: true,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    // After lower-level merges, faces with same surface+wires should merge.
    assert!(aRes.nb_merged_faces >= 0);
}

#[test]
fn BRepGraph_DeduplicateTest__MergeDefsWhenSafe_False_NoMerge() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    // Default: MergeEntitiesWhenSafe = false.
    let aRes = deduplicate::perform(&mut aGraph).unwrap();
    assert_eq!(aRes.nb_merged_vertices, 0);
    assert_eq!(aRes.nb_merged_edges, 0);
    assert_eq!(aRes.nb_merged_wires, 0);
    assert_eq!(aRes.nb_merged_faces, 0);
    assert!(!aRes.is_entity_merge_applied);
}

#[test]
fn BRepGraph_DeduplicateTest__AnalyzeOnly_MergeDefsWhenSafe_CountsOnly() {
    let mut aGraph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph.is_empty());
    let aNbVerticesBefore = aGraph.topo().vertices().nb();
    let anOpts = Options {
        merge_entities_when_safe: true,
        analyze_only: true,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph, &anOpts).unwrap();
    // Counts should be reported.
    assert!(aRes.nb_merged_vertices > 0);
    // But no mutation: all vertices still present.
    assert_eq!(aGraph.topo().vertices().nb(), aNbVerticesBefore);
    assert!(!aRes.is_entity_merge_applied);
}

fn dedup_assert_fresh_history_off() {
    // Fresh graph, run with history off - no records should be added.
    let mut aGraph2 = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!aGraph2.is_empty());
    let anOpts2 = Options {
        history_mode: false,
        ..Options::default()
    };
    let aRes = deduplicate::perform_with(&mut aGraph2, &anOpts2).unwrap();
    assert_eq!(aRes.nb_surface_rewrites, 1);
    assert_eq!(aRes.nb_curve_rewrites, 4);
    assert_eq!(dedup_history(&aGraph2).get().unwrap().nb_records(), 0);
}
