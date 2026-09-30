#[test]
fn BRepGraph_DeduplicateTest__RoundTrip_TwoCopiedFaces_FewerSurfaces() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().faces().nb(), 2);
    deduplicate::perform(&mut graph).unwrap();
    assert_eq!(nbUniqueFaceSurfaceDefs(&graph), 1);
    let rebuilt = dedup_reconstructed_faces(&graph);
    let second = dedup_graph(&rebuilt);
    assert!(!second.is_empty());
    assert_eq!(second.topo().faces().nb(), 2);
    assert_eq!(nbUniqueFaceSurfaceDefs(&second), 1);
}

#[test]
fn BRepGraph_DeduplicateTest__RoundTrip_TwoCopiedFaces_FewerCurves() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().edges().nb(), 8);
    deduplicate::perform(&mut graph).unwrap();
    assert_eq!(nbUniqueEdgeCurveDefs(&graph), 4);
    let rebuilt = dedup_reconstructed_faces(&graph);
    let second = dedup_graph(&rebuilt);
    assert!(!second.is_empty());
    assert_eq!(second.topo().edges().nb(), 8);
    assert_eq!(nbUniqueEdgeCurveDefs(&second), 4);
}

#[test]
fn BRepGraph_DeduplicateTest__RoundTrip_TwoBoxes_GeomReduction() {
    let mut graph = dedup_graph(&makeTwoIdenticalBoxes());
    assert!(!graph.is_empty());
    let surfaces = graph.topo().faces().nb();
    let curves = graph.topo().edges().nb();
    assert_eq!(surfaces, 12);
    assert_eq!(curves, 24);
    deduplicate::perform(&mut graph).unwrap();
    let shape = graph
        .shapes()
        .reconstruct(CompoundId::new(0).into())
        .unwrap();
    assert!(!shape.is_null());
    let second = dedup_graph(&shape);
    assert!(!second.is_empty());
    assert_eq!(second.topo().faces().nb(), surfaces);
    assert_eq!(second.topo().edges().nb(), curves);
    assert_eq!(nbUniqueFaceSurfaceDefs(&second), 6);
    assert_eq!(nbUniqueEdgeCurveDefs(&second), 12);
}

#[test]
fn BRepGraph_DeduplicateTest__FaceMerge_TwoIdenticalSingularFaces_Merge() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().faces().nb(), 2);
    let result = dedup_merge(&mut graph);
    assert_eq!(result.nb_merged_faces, 1);
    compact::perform(&mut graph).unwrap();
    assert_eq!(iterator::FaceIterator::new(&graph).count(), 1);
    assert!(validate::perform(&graph).is_valid());
}

#[test]
fn BRepGraph_DeduplicateTest__DedupMerge_TwoSpheres_DifferentSize_NotMerged() {
    let mut graph = dedup_graph(&dedup_compound(&[dedup_sphere(5.), dedup_sphere(10.)]));
    assert!(!graph.is_empty());
    let result = dedup_merge(&mut graph);
    assert_eq!(result.nb_merged_faces, 0);
    assert_eq!(result.nb_merged_edges, 0);
    assert_eq!(iterator::FaceIterator::new(&graph).count(), 2);
}

#[test]
fn BRepGraph_DeduplicateTest__DedupMerge_SelfLoopEdge_OrientationPreserved() {
    let mut graph = dedup_graph(&dedup_sphere(10.));
    assert!(!graph.is_empty());
    let _result = dedup_merge(&mut graph);
    compact::perform(&mut graph).unwrap();
    assert!(validate::perform(&graph).is_valid());
    dedup_assert_reconstructed_root(&graph);
}

#[test]
fn BRepGraph_DeduplicateTest__ThreeCopiedBoxes_DedupMergeCompact_FacesKeepOuterWires() {
    let mut graph = dedup_graph(&makeThreeCopiedBoxes());
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().faces().nb(), 18);
    assert_eq!(graph.topo().solids().nb(), 3);
    let result = dedup_merge(&mut graph);
    dedup_assert_three_box_merges(&result);
    let compacted = compact::perform(&mut graph).unwrap();
    assert_eq!(compacted.nb_removed_faces, 12);
    assert_eq!(compacted.nb_removed_edges, 24);
    dedup_assert_outer_wires(&graph);
    assert!(validate::perform(&graph).is_valid());
    dedup_assert_reconstructed_solids(&graph, true);
}

#[test]
fn BRepGraph_DeduplicateTest__DedupMerge_ThreeBoxes_AllEdgesMergeCorrectly() {
    let mut graph = dedup_graph(&makeThreeCopiedBoxes());
    assert!(!graph.is_empty());
    let result = dedup_merge(&mut graph);
    dedup_assert_three_box_merges(&result);
    let compacted = compact::perform(&mut graph).unwrap();
    assert_eq!(compacted.nb_removed_faces, 12);
    assert_eq!(compacted.nb_removed_edges, 24);
    assert!(validate::perform(&graph).is_valid());
    dedup_assert_outer_wires(&graph);
}

#[test]
fn BRepGraph_DeduplicateTest__SelfLoopEdge_AfterVertexMerge_OrientationCorrect() {
    let mut graph = dedup_graph(&dedup_compound(&[
        support::make_box(10., 10., 10.).into(),
        support::make_box(10., 10., 10.).into(),
    ]));
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().solids().nb(), 2);
    let _result = dedup_merge(&mut graph);
    compact::perform(&mut graph).unwrap();
    assert!(validate::perform(&graph).is_valid());
    dedup_assert_reconstructed_solids(&graph, false);
}
#[test]
fn BRepGraph_DeduplicateTest__Sphere_SeamPCurve_SurvivesDedupCompactReconstruct() {
    let mut graph = dedup_graph(&dedup_sphere(10.));
    assert!(!graph.is_empty());
    let pcurves = countValidCoEdgePCurves(&graph);
    let seams = countSeamCoEdges(&graph);
    assert_eq!(pcurves, 4);
    assert_eq!(seams, 2);
    let options = Options {
        merge_entities_when_safe: true,
        ..Options::default()
    };
    deduplicate::perform_with(&mut graph, &options).unwrap();
    dedup_assert_pcurve_handles(&graph);
    compact::perform(&mut graph).unwrap();
    assert_eq!(countValidCoEdgePCurves(&graph), pcurves);
    assert_eq!(countSeamCoEdges(&graph), seams);
    assert!(validate::perform(&graph).is_valid());
    dedup_assert_reconstructed_root(&graph);
}

#[test]
fn BRepGraph_DeduplicateTest__ThreeCopiedCylinderFaces_PCurvesSurviveFullPipeline() {
    let mut graph = dedup_graph(&dedup_copied_curved_faces(&dedup_cylinder(5., 20.), 3));
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().faces().nb(), 3);
    let edges = graph.topo().edges().nb();
    assert_eq!(edges, 9);
    assert_eq!(countValidCoEdgePCurves(&graph), 12);
    let options = Options {
        merge_entities_when_safe: true,
        ..Options::default()
    };
    deduplicate::perform_with(&mut graph, &options).unwrap();
    dedup_assert_pcurve_handles(&graph);
    let result = compact::perform(&mut graph).unwrap();
    assert_eq!(result.nb_removed_edges, 6);
    dedup_assert_pcurve_handles(&graph);
    assert!(validate::perform(&graph).is_valid());
    dedup_assert_reconstructed_faces(&graph);
}

#[test]
fn BRepGraph_DeduplicateTest__DedupMerge_TwoBoxesWithSharedEdge_EdgeFaceCountCorrect() {
    let mut graph = dedup_graph(&makeTwoFaceTouchingBoxes());
    assert!(!graph.is_empty());
    let options = Options {
        merge_entities_when_safe: true,
        ..Options::default()
    };
    deduplicate::perform_with(&mut graph, &options).unwrap();
    let count = (0..graph.topo().edges().nb())
        .map(EdgeId::new)
        .filter(|id| !id.is_removed(&graph))
        .filter(|&edge| dedup_distinct_edge_faces(&graph, edge) == 2)
        .count();
    assert_eq!(count, 16);
    compact::perform(&mut graph).unwrap();
    assert!(validate::perform(&graph).is_valid());
}

#[test]
fn BRepGraph_DeduplicateTest__MergeFaces_SameSurfaceDifferentWires_NotMerged() {
    let first = dedup_rectangle_on_plane(0., 0., 10., 20.);
    let second = dedup_rectangle_on_plane(30., 0., 40., 10.);
    assert!(!first.is_null());
    assert!(!second.is_null());
    let mut graph = dedup_graph(&dedup_compound(&[first, second]));
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().faces().nb(), 2);
    let options = Options {
        merge_entities_when_safe: true,
        ..Options::default()
    };
    let result = deduplicate::perform_with(&mut graph, &options).unwrap();
    assert_eq!(
        dedup_surface_pointer(&graph, 0),
        dedup_surface_pointer(&graph, 1)
    );
    assert_eq!(iterator::FaceIterator::new(&graph).count(), 2);
    assert_eq!(result.nb_merged_faces, 0);
}
