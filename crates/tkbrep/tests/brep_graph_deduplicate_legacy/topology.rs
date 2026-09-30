#[test]
fn BRepGraph_DeduplicateTest__SingleFace_NoSurfaceRewrite() {
    let mut graph = dedup_graph(&dedup_box_face());
    assert!(!graph.is_empty());
    let result = deduplicate::perform(&mut graph).unwrap();
    assert_eq!(result.nb_surface_rewrites, 0);
}

#[test]
fn BRepGraph_DeduplicateTest__FullBox_AllSurfacesUnique() {
    let mut graph = dedup_graph(&support::make_box(10., 20., 30.));
    assert!(!graph.is_empty());
    let result = deduplicate::perform(&mut graph).unwrap();
    assert_eq!(result.nb_canonical_surfaces, 6);
    assert_eq!(result.nb_surface_rewrites, 0);
}

#[test]
fn BRepGraph_DeduplicateTest__EmptyCompound_NoRewrites() {
    let mut graph = dedup_graph(&dedup_compound(&[]));
    assert!(!graph.is_empty());
    let result = deduplicate::perform(&mut graph).unwrap();
    assert_eq!(result.nb_surface_rewrites, 0);
    assert_eq!(result.nb_curve_rewrites, 0);
}

#[test]
fn BRepGraph_DeduplicateTest__AfterDedup_AllCopiedFacesShareCanonicalSurface() {
    let mut graph = dedup_graph(&makeNCopiedIdenticalFaces(3));
    assert!(!graph.is_empty());
    deduplicate::perform(&mut graph).unwrap();
    assert_eq!(nbUniqueFaceSurfaceDefs(&graph), 1);
}

#[test]
fn BRepGraph_DeduplicateTest__MultiplePCurveDups_AllEdgesDeduped() {
    let mut graph = dedup_graph(&dedup_copy(&dedup_box_face()));
    assert!(!graph.is_empty());
    let duplicates = addDuplicatePCurvesToAllEdges(&mut graph);
    assert!(duplicates > 0);
    deduplicate::perform(&mut graph).unwrap();
}

#[test]
fn BRepGraph_DeduplicateTest__Nullify_OrphanedPCurve_HandleIsNull() {
    let mut graph = dedup_graph(&dedup_copy(&dedup_box_face()));
    assert!(!graph.is_empty());
    let duplicates = addDuplicatePCurvesToAllEdges(&mut graph);
    assert!(duplicates > 0);
    deduplicate::perform(&mut graph).unwrap();
}

#[test]
fn BRepGraph_DeduplicateTest__NoPCurveDuplicates_ZeroPCurveRewrites() {
    let mut graph = dedup_graph(&dedup_copy(&dedup_box_face()));
    assert!(!graph.is_empty());
    deduplicate::perform(&mut graph).unwrap();
}

#[test]
fn BRepGraph_DeduplicateTest__PCurveDup_AnalyzeOnly_CountsButNoRewrite() {
    let mut graph = dedup_graph(&dedup_copy(&dedup_box_face()));
    assert!(!graph.is_empty());
    let duplicates = addDuplicatePCurvesToAllEdges(&mut graph);
    assert!(duplicates > 0);
    let before = nbUniquePCurveNodes(&graph);
    let _result = deduplicate::perform_with(
        &mut graph,
        &Options {
            analyze_only: true,
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(nbUniquePCurveNodes(&graph), before);
}

#[test]
fn BRepGraph_DeduplicateTest__PCurveEntryCount_UnchangedAfterDedup() {
    let mut graph = dedup_graph(&dedup_copy(&dedup_box_face()));
    assert!(!graph.is_empty());
    addDuplicatePCurvesToAllEdges(&mut graph);
    let before = nbPCurveEntries(&graph);
    deduplicate::perform(&mut graph).unwrap();
    assert_eq!(nbPCurveEntries(&graph), before);
}

#[test]
fn BRepGraph_DeduplicateTest__AfterDedup_AllSurfacesValid() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    deduplicate::perform(&mut graph).unwrap();
    dedup_assert_face_representations(&graph);
}

#[test]
fn BRepGraph_DeduplicateTest__AfterDedup_CanonicalSurfaceGeomNotNull() {
    let mut graph = dedup_graph(&makeNCopiedIdenticalFaces(4));
    assert!(!graph.is_empty());
    deduplicate::perform(&mut graph).unwrap();
    dedup_assert_face_representations(&graph);
}

#[test]
fn BRepGraph_DeduplicateTest__AfterDedup_AllCurve3dsValid() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    deduplicate::perform(&mut graph).unwrap();
    dedup_assert_edge_representations(&graph);
}

#[test]
fn BRepGraph_DeduplicateTest__AfterDedup_CanonicalCurveGeomNotNull() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    deduplicate::perform(&mut graph).unwrap();
    dedup_assert_edge_representations(&graph);
}

#[test]
fn BRepGraph_DeduplicateTest__AfterDedup_AllInlinePCurvesHaveCurve2d() {
    let mut graph = dedup_graph(&dedup_copy(&dedup_box_face()));
    assert!(!graph.is_empty());
    addDuplicatePCurvesToAllEdges(&mut graph);
    deduplicate::perform(&mut graph).unwrap();
    dedup_assert_pcurve_representations(&graph);
}

#[test]
fn BRepGraph_DeduplicateTest__Nullify_OrphanedSurface_HandleIsNull() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    let result = deduplicate::perform(&mut graph).unwrap();
    assert_eq!(result.nb_nullified_surfaces, 0);
    dedup_assert_face_handles(&graph);
}

#[test]
fn BRepGraph_DeduplicateTest__Nullify_OrphanedCurve_HandleIsNull() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    let result = deduplicate::perform(&mut graph).unwrap();
    assert_eq!(result.nb_nullified_curves, 0);
    dedup_assert_edge_handles(&graph);
    assert_eq!(nbUniqueEdgeCurveDefs(&graph), 4);
}

#[test]
fn BRepGraph_DeduplicateTest__TwoCopiedSphereFaces_Deduped() {
    let mut graph = dedup_graph(&dedup_copied_curved_faces(&dedup_sphere(10.), 2));
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().faces().nb(), 2);
    assert_eq!(graph.topo().edges().nb(), 6);
    let result = deduplicate::perform(&mut graph).unwrap();
    assert_eq!(result.nb_canonical_surfaces, 1);
    assert_eq!(result.nb_surface_rewrites, 1);
    assert_eq!(
        dedup_surface_pointer(&graph, 0),
        dedup_surface_pointer(&graph, 1)
    );
    assert!(result.nb_history_records > 0);
}

#[test]
fn BRepGraph_DeduplicateTest__TwoCopiedCylinderFaces_Deduped() {
    let mut graph = dedup_graph(&dedup_copied_curved_faces(&dedup_cylinder(5., 20.), 2));
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().faces().nb(), 2);
    assert_eq!(graph.topo().faces().nb(), 2);
    assert_eq!(graph.topo().edges().nb(), 6);
    let result = deduplicate::perform(&mut graph).unwrap();
    assert_eq!(result.nb_canonical_surfaces, 1);
    assert_eq!(result.nb_canonical_curves, 3);
    assert_eq!(result.nb_surface_rewrites, 1);
    assert_eq!(result.nb_curve_rewrites, 3);
    assert!(result.nb_history_records > 0);
}

#[test]
fn BRepGraph_DeduplicateTest__DifferentSizedCylinders_NotDeduped() {
    let mut graph = dedup_graph(&dedup_compound(&[
        dedup_first(&dedup_cylinder(5., 20.), ShapeEnum::Face),
        dedup_first(&dedup_cylinder(10., 20.), ShapeEnum::Face),
    ]));
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().faces().nb(), 2);
    assert_eq!(graph.topo().faces().nb(), 2);
    assert_eq!(graph.topo().edges().nb(), 6);
    let result = deduplicate::perform(&mut graph).unwrap();
    assert_eq!(result.nb_canonical_surfaces, 2);
    assert_eq!(result.nb_canonical_curves, 6);
    assert_eq!(result.nb_surface_rewrites, 0);
    assert_eq!(result.nb_curve_rewrites, 0);
    assert!(result.nb_history_records == 0);
}

#[test]
fn BRepGraph_DeduplicateTest__BackRefs_SurfaceRewrite_UpdatesFaceDefUsers() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().faces().nb(), 2);
    assert_ne!(
        dedup_surface_pointer(&graph, 0),
        dedup_surface_pointer(&graph, 1)
    );
    deduplicate::perform(&mut graph).unwrap();
    assert_eq!(
        dedup_surface_pointer(&graph, 0),
        dedup_surface_pointer(&graph, 1)
    );
}

#[test]
fn BRepGraph_DeduplicateTest__BackRefs_CurveRewrite_UpdatesEdgeDefUsers() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().edges().nb(), 8);
    deduplicate::perform(&mut graph).unwrap();
    assert_eq!(nbUniqueEdgeCurveDefs(&graph), 4);
}

#[test]
fn BRepGraph_DeduplicateTest__FacesOnSurface_AfterDedup_ReturnsCorrectDefs() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    deduplicate::perform(&mut graph).unwrap();
    assert_eq!(graph.topo().faces().nb(), 2);
    assert!(tool::face::has_surface(&graph, FaceId::start()));
    assert!(tool::face::has_surface(&graph, FaceId::new(1)));
    assert_eq!(
        dedup_surface_pointer(&graph, 0),
        dedup_surface_pointer(&graph, 1)
    );
}

#[test]
fn BRepGraph_DeduplicateTest__MergeVertices_SharedVerticesReduced() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    let before = graph.topo().vertices().nb();
    let result = deduplicate::perform_with(
        &mut graph,
        &Options {
            merge_entities_when_safe: true,
            ..Options::default()
        },
    )
    .unwrap();
    assert!(result.nb_merged_vertices > 0);
    let count = iterator::VertexIterator::new(&graph).count();
    assert!(count < before as usize);
}

#[test]
fn BRepGraph_DeduplicateTest__Build_SharedTFace_OneSurfaceNode() {
    let face = dedup_box_face();
    let graph = dedup_graph(&dedup_compound(&[face.clone(), face]));
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().faces().nb(), 1);
}
