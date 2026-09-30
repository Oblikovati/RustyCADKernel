fn child_translation_x(graph: &BRepGraph, reference: ChildRefId) -> f64 {
    graph
        .refs()
        .children()
        .entry(reference)
        .local_location
        .transformation()
        .value(1, 4)
        .unwrap()
}
fn occurrence_translation_x(graph: &BRepGraph, reference: OccurrenceRefId) -> f64 {
    graph
        .refs()
        .occurrences()
        .entry(reference)
        .local_location
        .transformation()
        .value(1, 4)
        .unwrap()
}
#[test]
fn BRepGraph_TransformTest__MoveRef_ChildRef_ComposesLocation() {
    let mut graph = transform_box(false);
    let compound = graph.editor().compounds().add(&[SolidId::start().into()]);
    assert!(compound.is_valid());
    let references = graph.refs().children().ids_of(compound);
    assert_eq!(references.len(), 1);
    let reference = references[0];
    assert!(
        graph
            .refs()
            .children()
            .entry(reference)
            .local_location
            .is_identity()
    );
    let transform = graph_translation(42.0, 0.0, 0.0);
    assert!(transform::move_child_ref(&mut graph, reference, &transform).unwrap());
    transform_near(child_translation_x(&graph, reference), 42.0);
    transform::move_child_ref(&mut graph, reference, &transform).unwrap();
    transform_near(child_translation_x(&graph, reference), 84.0);
}
#[test]
fn BRepGraph_TransformTest__MoveRef_OccurrenceRef_ComposesLocationAndRejectsScale() {
    let mut graph = transform_box(false);
    let reference = first_transform_occurrence(&graph);
    let translation = graph_translation(1.0, 2.0, 3.0);
    assert!(transform::move_occurrence_ref(&mut graph, reference, &translation).unwrap());
    transform_near(occurrence_translation_x(&graph, reference), 1.0);
    let mut scale = Transform3::default();
    scale.set_scale(Point3::default(), 2.0).unwrap();
    assert!(!transform::move_occurrence_ref(&mut graph, reference, &scale).unwrap());
    transform_near(occurrence_translation_x(&graph, reference), 1.0);
}
