fn scenario_root_product(graph: &mut BRepGraph) -> ProductId {
    let id = graph.editor().products().add_empty();
    graph.editor().products().append_document_root(id).unwrap();
    id
}
fn scenario_link(
    graph: &mut BRepGraph,
    parent: ProductId,
    child: ProductId,
    xyz: [f64; 3],
    context: OccurrenceId,
) -> OccurrenceId {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(xyz[0], xyz[1], xyz[2]));
    let location = Location::new(transform);
    let id = graph
        .editor()
        .products()
        .append(parent, child, &location, context)
        .unwrap()
        .0;
    assert!(id.is_valid());
    id
}
fn scenario_part_round_trip(graph: &BRepGraph, faces: u32) {
    let reconstructed = scenario_reconstruct(graph, SolidId::start());
    assert_eq!(scenario_count(&reconstructed, ShapeEnum::Face), faces);
    let populated = scenario_populated(&reconstructed);
    assert_eq!(populated.topo().solids().nb(), 1);
    assert_eq!(populated.topo().faces().nb(), faces);
    scenario_valid(&populated);
}
fn scenario_two_occurrences(graph: &mut BRepGraph) -> [OccurrenceId; 2] {
    let root = scenario_root_product(graph);
    assert!(root.is_valid());
    let part = ProductId::start();
    let first = scenario_link(
        graph,
        root,
        part,
        [100.0, 0.0, 0.0],
        OccurrenceId::default(),
    );
    let second = scenario_link(
        graph,
        root,
        part,
        [0.0, 200.0, 0.0],
        OccurrenceId::default(),
    );
    [first, second]
}
fn scenario_occurrence_pair_queries(graph: &BRepGraph, ids: [OccurrenceId; 2]) {
    let result = validate::perform_with_options(graph, &Options::lightweight());
    assert!(result.is_valid());
    assert_eq!(result.nb_issues(Severity::Error), 0);
    let occurrences = graph.topo().occurrences();
    assert_eq!(occurrences.product(ids[0]), ProductId::start());
    assert_eq!(occurrences.product(ids[1]), ProductId::start());
    let first = occurrences.occurrence_location(ids[0]);
    let second = occurrences.occurrence_location(ids[1]);
    scenario_near(
        first.transformation().translation_part().x(),
        100.0,
        precision::confusion(),
    );
    scenario_near(
        second.transformation().translation_part().y(),
        200.0,
        precision::confusion(),
    );
}
#[test]
fn BRepGraph_ScenarioMatrix__Assembly_TwoOccurrences_ValidateDAGReconstructPartPopulate() {
    let mut graph = scenario_graph(&support::make_box(8.0, 8.0, 8.0));
    let original = graph.shapes().original(SolidId::start().into());
    assert!(!original.is_null());
    let faces = scenario_count(&original, ShapeEnum::Face);
    let occurrences = scenario_two_occurrences(&mut graph);
    scenario_occurrence_pair_queries(&graph, occurrences);
    scenario_part_round_trip(&graph, faces);
}
fn scenario_four_level_chain(graph: &mut BRepGraph) {
    let middle = scenario_root_product(graph);
    let root = scenario_root_product(graph);
    let top = scenario_root_product(graph);
    let first = scenario_link(graph, top, root, [0.0, 0.0, 3.0], OccurrenceId::default());
    let second = scenario_link(graph, root, middle, [0.0, 2.0, 0.0], first);
    scenario_link(graph, middle, ProductId::start(), [1.0, 0.0, 0.0], second);
    assert_eq!(graph.topo().products().nb_components(top), 1);
    assert_eq!(graph.topo().products().nb_components(root), 1);
    assert_eq!(graph.topo().products().nb_components(middle), 1);
}
fn scenario_add_cycle(graph: &mut BRepGraph) {
    let first = scenario_root_product(graph);
    let second = scenario_root_product(graph);
    let location = Location::new(Transform3::default());
    let context = OccurrenceId::default();
    let forward = graph
        .editor()
        .products()
        .append(first, second, &location, context)
        .unwrap()
        .0;
    assert!(forward.is_valid());
    let back = graph
        .editor()
        .products()
        .append(second, first, &location, context)
        .unwrap()
        .0;
    assert!(back.is_valid());
}
#[test]
fn BRepGraph_ScenarioMatrix__Assembly_ThreeLevelNesting_CleanAudit_CycleDetection() {
    let mut graph = scenario_graph(&support::make_box(10.0, 10.0, 10.0));
    scenario_four_level_chain(&mut graph);
    scenario_valid(&graph);
    scenario_add_cycle(&mut graph);
    let result = validate::perform_with_options(&graph, &Options::audit());
    assert!(!result.is_valid());
    // Preserve the native >= 0 predicate, including its not-found value.
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.description.search("Assembly cycle") >= 0)
    );
}
fn scenario_assembly_shape(graph: &BRepGraph, root: ProductId) -> Shape {
    let shape = graph.shapes().shape(root.into()).unwrap();
    assert!(!shape.is_null());
    assert_eq!(shape.shape_type().unwrap(), ShapeEnum::Compound);
    assert_eq!(shape.nb_children(), 1);
    shape
}
fn scenario_shared_part_shapes(graph: &BRepGraph, first: ProductId, second: ProductId) {
    let first = scenario_assembly_shape(graph, first);
    let second = scenario_assembly_shape(graph, second);
    let first = ShapeIterator::new(&first, true, true);
    let second = ShapeIterator::new(&second, true, true);
    assert!(first.more() && second.more());
    assert_eq!(
        first.value().unwrap().shape_type().unwrap(),
        second.value().unwrap().shape_type().unwrap()
    );
}
#[test]
fn BRepGraph_ScenarioMatrix__Assembly_SharedPartBetweenTwoRootAssemblies() {
    let mut graph = scenario_graph(&support::make_box(5.0, 5.0, 5.0));
    let solids = graph.topo().solids().nb();
    let faces = graph.topo().faces().nb();
    let [first, second] = scenario_shared_roots(&mut graph);
    assert_eq!(graph.topo().solids().nb(), solids);
    assert_eq!(graph.topo().faces().nb(), faces);
    scenario_valid(&graph);
    scenario_shared_part_shapes(&graph, first, second);
}

fn scenario_shared_roots(graph: &mut BRepGraph) -> [ProductId; 2] {
    let first = scenario_root_product(graph);
    let second = scenario_root_product(graph);
    scenario_link(
        graph,
        first,
        ProductId::start(),
        [100.0, 0.0, 0.0],
        OccurrenceId::default(),
    );
    scenario_link(
        graph,
        second,
        ProductId::start(),
        [0.0, 100.0, 0.0],
        OccurrenceId::default(),
    );
    [first, second]
}
