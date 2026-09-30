fn validation_graph(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph.shapes_mut().add(shape).unwrap();
    assert!(!graph.is_empty());
    graph
}
fn validation_box() -> BRepGraph {
    validation_graph(&support::make_box(10.0, 20.0, 30.0))
}
fn audit_graph(graph: &BRepGraph) -> ValidationResult {
    validate::perform_with_options(graph, &Options::audit())
}
fn contains_validation_issue(result: &ValidationResult, severity: Severity, text: &str) -> bool {
    result
        .issues
        .iter()
        .any(|issue| issue.sev == severity && issue.description.search(text) >= 1)
}
fn assert_audit_errors(graph: &BRepGraph) -> ValidationResult {
    let result = audit_graph(graph);
    assert!(!result.is_valid());
    assert!(result.nb_issues(Severity::Error) > 0);
    result
}
fn assert_valid_no_errors(result: &ValidationResult) {
    assert!(
        result.is_valid(),
        "expected valid graph, received {:?}",
        result.issues
    );
    assert_eq!(result.nb_issues(Severity::Error), 0);
}
fn validation_edge_with_curve(graph: &BRepGraph, skip: EdgeId) -> EdgeId {
    let mut edges = EdgeIterator::new(graph);
    while edges.more() {
        let id = edges.current_id();
        if id != skip
            && graph
                .topo()
                .edges()
                .definition(id)
                .curve3d_rep_id
                .is_valid()
        {
            return id;
        }
        edges.next();
    }
    EdgeId::default()
}
fn validation_first_vertex_ref(graph: &BRepGraph) -> VertexRefId {
    for i in 0..graph.topo().edges().nb() {
        let reference =
            rustycad_tkbrep::brep_graph::tool::edge::start_vertex_id(graph, EdgeId::new(i));
        if rustycad_tkbrep::brep_graph::tool::vertex::usage(graph, reference).is_valid() {
            return reference;
        }
    }
    VertexRefId::default()
}
fn removed_loose_vertex(graph: &mut BRepGraph) -> VertexId {
    let id = graph
        .editor()
        .vertices()
        .add(Point3::new(100.0, 0.0, 0.0), precision::confusion());
    graph.editor().generic().remove_node(id.into()).unwrap();
    id
}
fn validation_translation(x: f64) -> Location {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(x, 0.0, 0.0));
    Location::new(transform)
}
fn validation_assembly(graph: &mut BRepGraph) -> ProductId {
    let root = graph.editor().products().add_empty();
    graph
        .editor()
        .products()
        .append_document_root(root)
        .unwrap();
    assert!(root.is_valid());
    root
}
fn validation_occurrence(
    graph: &mut BRepGraph,
    root: ProductId,
    part: ProductId,
    location: &Location,
) -> OccurrenceId {
    graph
        .editor()
        .products()
        .append(root, part, location, OccurrenceId::default())
        .unwrap()
        .0
}

use rustycad_tkbrep::brep_graph::tool::vertex as validation_vertex;
fn corrupt_owned_curve(graph: &mut BRepGraph, edge: EdgeId, rep: EdgeCurve3DRepId) {
    let mut guard = graph.editor().edges().mutate(edge).unwrap();
    guard.internal().curve3d_rep_id = rep;
}
fn assert_specific_audit_error(graph: &BRepGraph, text: &str) {
    let result = audit_graph(graph);
    assert!(!result.is_valid());
    assert!(contains_validation_issue(&result, Severity::Error, text));
}
fn validation_reference_vertex(graph: &BRepGraph, reference: VertexRefId) -> VertexId {
    graph.refs().vertices().entry(reference).child_vertex_id
}
