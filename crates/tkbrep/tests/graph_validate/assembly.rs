#[test]
fn BRepGraph_ValidateTest__AssemblyGraph_ValidProduct_NoIssuesInAudit() {
    let mut graph = validation_box();
    assert!(graph.topo().products().nb() >= 1);
    let part = ProductId::start();
    assert!(graph.topo().products().is_part(part));
    let root = validation_assembly(&mut graph);
    let first = validation_occurrence(&mut graph, root, part, &Location::default());
    let second = validation_occurrence(&mut graph, root, part, &validation_translation(20.0));
    assert!(first.is_valid());
    assert!(second.is_valid());
    assert!(graph.topo().products().is_assembly(root));
    assert_eq!(graph.topo().products().nb_components(root), 2);
    graph.editor().commit_mutation();
    assert!(audit_graph(&graph).is_valid());
}
#[test]
fn BRepGraph_ValidateTest__DocumentRootReferencedByOccurrence_Detected() {
    let mut graph = validation_box();
    let part = ProductId::start();
    assert!(graph.topo().products().is_part(part));
    let root = validation_assembly(&mut graph);
    assert!(validation_occurrence(&mut graph, root, part, &Location::default()).is_valid());
    assert_eq!(graph.root_product_ids(), &[root]);
    graph
        .editor()
        .products()
        .append_document_root(part)
        .unwrap();
    assert!(!validate::perform_with_options(&graph, &Options::lightweight()).is_valid());
    assert_document_root_error(&graph, part);
}
#[test]
fn BRepGraph_ValidateTest__AssemblyGraph_CorruptedOccurrenceChildNodeId_DetectedByAudit() {
    let mut graph = validation_box();
    assert!(graph.topo().products().nb() >= 1);
    assert!(graph.topo().occurrences().nb() >= 1);
    let invalid = NodeId::new(NodeKind::SOLID, graph.topo().solids().nb() + 999);
    let mut guard = graph
        .editor()
        .occurrences()
        .mutate(OccurrenceId::start())
        .unwrap();
    guard.internal().child_node_id = invalid;
    let result = assert_audit_errors(guard.graph());
    assert!(contains_validation_issue(
        &result,
        Severity::Error,
        "ChildNodeId invalid"
    ));
}
#[test]
fn BRepGraph_ValidateTest__AssemblyGraph_CorruptedOccurrenceChildNodeId_ProductIndex_DetectedByAudit()
 {
    let mut graph = validation_box();
    let root = validation_assembly(&mut graph);
    let part = ProductId::start();
    assert!(graph.topo().products().is_part(part));
    assert!(part.is_valid());
    let occurrence = validation_occurrence(&mut graph, root, part, &Location::default());
    assert!(occurrence.is_valid());
    let invalid = NodeId::new(NodeKind::PRODUCT, graph.topo().products().nb() + 999);
    let mut guard = graph.editor().occurrences().mutate(occurrence).unwrap();
    guard.internal().child_node_id = invalid;
    let result = assert_audit_errors(guard.graph());
    assert!(contains_validation_issue(
        &result,
        Severity::Error,
        "ChildNodeId invalid"
    ));
}
#[test]
fn BRepGraph_ValidateTest__AssemblyGraph_OccurrenceChildRefersToOccurrence_DetectedByAudit() {
    let mut graph = validation_box();
    assert!(graph.topo().occurrences().nb() > 0);
    let mut guard = graph
        .editor()
        .occurrences()
        .mutate(OccurrenceId::start())
        .unwrap();
    guard.internal().child_node_id = OccurrenceId::start().into();
    let result = audit_graph(guard.graph());
    assert!(!result.is_valid());
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.sev == Severity::Error
                && issue.description.search("cannot reference an Occurrence") >= 0)
    );
}

fn assert_document_root_error(graph: &BRepGraph, part: ProductId) {
    let result = audit_graph(graph);
    assert!(!result.is_valid());
    // Preserve the original >= 0 substring check, including its zero sentinel behavior.
    assert!(result.issues.iter().any(|issue| {
        issue.sev == Severity::Error
            && issue.node_id == NodeId::from(part)
            && issue
                .description
                .search("Document root Product is referenced")
                >= 0
    }));
}
