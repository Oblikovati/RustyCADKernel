#[test]
fn BRepGraph_SeamRedesignTest__Reconstruct_CylinderWire_TopoDSIteratorOrder() {
    let original = seam_cylinder();
    let graph = seam_graph(&original);
    let reconstructed = graph
        .shapes()
        .reconstruct(NodeId::new(NodeKind::SOLID, 0))
        .unwrap();
    assert!(!reconstructed.is_null());
    // Upstream checks the ordered wire-edge counts, not TShape identity.
    let original_counts = shape_wire_counts(&original);
    let reconstructed_counts = shape_wire_counts(&reconstructed);
    assert_eq!(original_counts.len(), reconstructed_counts.len());
    assert_eq!(original_counts, reconstructed_counts);
}
#[test]
fn BRepGraph_SeamRedesignTest__Validate_DetectsAsymmetricSeamPair() {
    let mut graph = seam_graph(&seam_cylinder());
    let seam = first_seam_coedge(&graph);
    assert!(seam.is_valid());
    let mate = co_edge::seam_pair(&graph, seam);
    assert!(mate.is_valid());
    // Break the opposite-orientation invariant, as in the original regression.
    let orientation = graph.topo().co_edges().definition(seam).orientation;
    graph
        .editor()
        .co_edges()
        .set_orientation(mate, orientation)
        .unwrap();
    assert_invalid_seam_orientation(&graph, seam);
}
fn assert_invalid_seam_orientation(graph: &BRepGraph, seam: CoEdgeId) {
    let definition = graph.topo().co_edges().definition(seam);
    assert!(!edge::is_seam_on_face(
        graph,
        definition.child_edge_id,
        definition.face_id
    ));
    let audit = validate::perform_with_options(graph, &validate::Options::audit());
    assert!(!audit.is_valid());
    assert!(audit.nb_issues(validate::Severity::Error) > 0);
    assert!(
        audit
            .issues
            .iter()
            .any(|issue| issue.sev == validate::Severity::Error
                && issue.description.search("same Orientation") > 0)
    );
}
