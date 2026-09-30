#[test]
fn BRepGraph_ValidateTest__CleanGraph_NoIssues() {
    let graph = validation_box();
    let result = validate::perform(&graph);
    assert_valid_no_errors(&result);
    assert_eq!(result.nb_issues(Severity::Warning), 0);
    assert_valid_no_errors(&audit_graph(&graph));
    assert_valid_no_errors(&validate::perform_with_options(
        &graph,
        &Options::lightweight(),
    ));
}
#[test]
fn BRepGraph_ValidateTest__AfterGeomDeduplicate_NoIssues() {
    let shape = support::make_box(10.0, 20.0, 30.0);
    let explorer = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let first = validation_copy_face(explorer.current());
    let second = validation_copy_face(explorer.current());
    let mut compound = Compound::default();
    let builder = BRepBuilder::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &first).unwrap();
    builder.add(&mut compound, &second).unwrap();
    let mut graph = validation_graph(&compound);
    deduplicate::perform(&mut graph).unwrap();
    assert_valid_no_errors(&validate::perform(&graph));
}
#[test]
fn BRepGraph_ValidateTest__RemoveNodeMaintainsActiveCount() {
    let mut graph = validation_box();
    let before = graph.topo().faces().nb_active();
    assert!(before > 0);
    graph
        .editor()
        .generic()
        .remove_node(FaceId::start().into())
        .unwrap();
    let result = audit_graph(&graph);
    assert!(
        !result
            .issues
            .iter()
            .any(|issue| issue.description.search("NbActiveFaces mismatch") > 0)
    );
    assert_eq!(graph.topo().faces().nb_active(), before - 1);
}
#[test]
fn BRepGraph_ValidateTest__Audit_ValidatesCoEdgeUIDsFromBuilderWireCreation() {
    let mut graph = validation_box();
    assert!(graph.topo().edges().nb() > 0);
    let coedge = graph
        .editor()
        .co_edges()
        .add(EdgeId::start(), ParityOrientation::FORWARD);
    let wire = graph.editor().wires().add(&[coedge]);
    assert!(wire.is_valid());
    let coedges = &graph.topo().wires().relations(wire).co_edge_ids;
    assert_eq!(coedges.len(), 1);
    assert!(graph.uids().of_node(coedges[0].into()).is_valid());
    assert_valid_no_errors(&audit_graph(&graph));
}
#[test]
fn BRepGraph_ValidateTest__Audit_WarnsSurfacedFaceWithNoWireRefs() {
    let mut graph = validation_graph(&support::make_box(10.0, 10.0, 10.0));
    assert!(remove_first_face_wire(&mut graph));
    let result = audit_graph(&graph);
    assert!(result.is_valid());
    assert!(!result.issues.is_empty());
    assert_eq!(result.issues[0].sev, Severity::Warning);
    assert_ne!(result.issues[0].description.search("no wire refs"), -1);
}
fn remove_first_face_wire(graph: &mut BRepGraph) -> bool {
    for i in 0..graph.topo().faces().nb() {
        let face = FaceId::new(i);
        let Some(reference) = graph
            .topo()
            .faces()
            .relations(face)
            .wire_ref_ids
            .last()
            .copied()
        else {
            continue;
        };
        assert!(graph.editor().faces().remove_wire(face, reference).unwrap());
        return true;
    }
    false
}

fn validation_copy_face(face: &Shape) -> Shape {
    BRepBuilderAPICopy::with_shape(face, true, false)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}
