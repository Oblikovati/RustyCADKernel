#[test]
fn BRepGraph_ValidateTest__DetectsRemovedNodeReference() {
    let mut graph = validation_box();
    assert!(graph.topo().vertices().nb() > 0);
    let reference = validation_first_vertex_ref(&graph);
    let removed = removed_loose_vertex(&mut graph);
    assert!(removed.is_valid());
    assert!(reference.is_valid());
    let mut guard = graph.editor().vertices().mutate_ref(reference).unwrap();
    guard.internal().child_vertex_id = removed;
    assert!(!validate::perform(guard.graph()).is_valid());
    assert_audit_errors(guard.graph());
}
#[test]
fn BRepGraph_ValidateTest__BoundsCheck_InvalidIndex() {
    let mut graph = validation_box();
    assert!(graph.topo().edges().nb() > 0);
    let mut guard = graph.editor().edges().mutate(EdgeId::start()).unwrap();
    guard.internal().curve3d_rep_id = EdgeCurve3DRepId::default();
    assert_audit_errors(guard.graph());
}
#[test]
fn BRepGraph_ValidateTest__CorruptedPCurve_FaceIdOutOfBounds() {
    let mut graph = validation_box();
    assert!(graph.topo().edges().nb() > 0);
    assert!(graph.topo().co_edges().nb() > 0);
    let invalid = FaceId::new(graph.topo().faces().nb() + 999);
    let mut guard = graph.editor().co_edges().mutate(CoEdgeId::start()).unwrap();
    guard.internal().face_id = invalid;
    assert!(!validate::perform(guard.graph()).is_valid());
    assert!(!validate::perform_with_options(guard.graph(), &Options::lightweight()).is_valid());
    assert!(!audit_graph(guard.graph()).is_valid());
}
#[test]
fn BRepGraph_ValidateTest__LightweightVsAudit_RemovedVertexReference_Differential() {
    let mut graph = validation_box();
    assert!(graph.topo().vertices().nb() > 0);
    let found = validation_first_vertex_ref(&graph);
    assert!(validation_vertex::usage(&graph, found).def_id.is_valid());
    let reference = validation_edge::start_vertex_id(&graph, EdgeId::start());
    assert!(reference.is_valid());
    let removed = removed_loose_vertex(&mut graph);
    let mut guard = graph.editor().vertices().mutate_ref(reference).unwrap();
    guard.internal().child_vertex_id = removed;
    assert!(!validate::perform_with_options(guard.graph(), &Options::lightweight()).is_valid());
    let result = assert_audit_errors(guard.graph());
    assert!(contains_validation_issue(
        &result,
        Severity::Error,
        "references removed"
    ));
}
#[test]
fn BRepGraph_ValidateTest__Audit_DetectsSharedOwnedUse() {
    let mut graph = validation_box();
    let first = validation_edge_with_curve(&graph, EdgeId::default());
    assert!(first.is_valid());
    let second = validation_edge_with_curve(&graph, first);
    assert!(second.is_valid());
    let rep = graph.topo().edges().definition(first).curve3d_rep_id;
    assert!(rep.is_valid());
    corrupt_owned_curve(&mut graph, second, rep);
    assert_specific_audit_error(&graph, "Active EdgeCurve3DRep has multiple owners");
}
#[test]
fn BRepGraph_ValidateTest__Audit_DetectsActiveOwnedUseWithoutOwner() {
    let mut graph = validation_box();
    let edge = validation_edge_with_curve(&graph, EdgeId::default());
    assert!(edge.is_valid());
    let rep = graph.topo().edges().definition(edge).curve3d_rep_id;
    assert!(rep.is_valid());
    corrupt_owned_curve(&mut graph, edge, EdgeCurve3DRepId::default());
    assert_specific_audit_error(&graph, "Active EdgeCurve3DRep has no owner");
}
