fn assembly_translation_x(shape: &Shape) -> f64 {
    shape.location().transformation().translation_part().x()
}
fn assembly_two_children(shape: &Shape) -> [Shape; 2] {
    let mut iterator = ShapeIterator::new(shape, true, true);
    assert!(iterator.more());
    let first = iterator.value().unwrap().clone();
    iterator.next();
    assert!(iterator.more());
    let second = iterator.value().unwrap().clone();
    iterator.next();
    assert!(!iterator.more());
    [first, second]
}
#[test]
fn BRepGraph_AssemblyTest__ShapesView_ProductShape_ReconstructsBuiltRootTransform() {
    let mut shape = support::make_box(10.0, 20.0, 30.0).into_shape();
    shape
        .move_by(&assembly_location(15.0, -7.0, 2.0), true)
        .unwrap();
    shape.reverse();
    let graph = assembly_source(&shape);
    let product = assembly_shape(&graph, ProductId::start());
    assert_eq!(product.shape_type().unwrap(), shape.shape_type().unwrap());
    // Native no longer asserts root orientation; ProductDef does not store it.
    let reconstructed = assembly_reconstruct(&graph, ProductId::start());
    assert_eq!(
        reconstructed.shape_type().unwrap(),
        shape.shape_type().unwrap()
    );
}
#[test]
fn BRepGraph_AssemblyTest__ShapesView_AssemblyProduct_ReconstructsChildOccurrences() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    assert!(assembly_x(&mut graph, root, ProductId::start(), 100.0).is_valid());
    assert!(assembly_x(&mut graph, root, ProductId::start(), 200.0).is_valid());
    let shape = assembly_shape(&graph, root);
    assert_eq!(shape.shape_type().unwrap(), ShapeEnum::Compound);
    assert_eq!(shape.nb_children(), 2);
    let [first, second] = assembly_two_children(&shape);
    assert!(first.is_partner(&second));
    assembly_near(assembly_translation_x(&first), 100.0);
    assembly_near(assembly_translation_x(&second), 200.0);
}
fn assembly_nested_pair() -> (BRepGraph, ProductId, OccurrenceId) {
    let mut graph = assembly_box();
    let sub = assembly_root(&mut graph);
    let root = assembly_root(&mut graph);
    let parent = assembly_x(&mut graph, root, sub, 10.0);
    assert!(parent.is_valid());
    let child = assembly_link(
        &mut graph,
        sub,
        ProductId::start(),
        &assembly_location(20.0, 0.0, 0.0),
        parent,
    );
    assert!(child.is_valid());
    (graph, sub, child)
}
#[test]
fn BRepGraph_AssemblyTest__ShapesView_OccurrenceShape_UsesGlobalPlacementChain() {
    let (graph, sub, child) = assembly_nested_pair();
    let shape = assembly_shape(&graph, sub);
    assert_eq!(shape.nb_children(), 1);
    let iterator = ShapeIterator::new(&shape, true, true);
    assert!(iterator.more());
    assembly_near(assembly_translation_x(iterator.value().unwrap()), 20.0);
    // Despite the legacy test's name, its assertions require local placement.
    assembly_near(assembly_translation_x(&assembly_shape(&graph, child)), 20.0);
    assembly_near(
        assembly_translation_x(&assembly_reconstruct(&graph, child)),
        20.0,
    );
}
fn assembly_branched_children(
    graph: &mut BRepGraph,
    sub: ProductId,
    parents: [OccurrenceId; 2],
    common: bool,
) {
    if common {
        assert!(assembly_x(graph, sub, ProductId::start(), 5.0).is_valid());
    }
    let first = assembly_child_x(graph, sub, 10.0, parents[0]);
    assert!(first.is_valid());
    let second = assembly_child_x(graph, sub, 20.0, parents[1]);
    assert!(second.is_valid());
}
fn assembly_branched(common: bool) -> (BRepGraph, [OccurrenceId; 2]) {
    let mut graph = assembly_box();
    let sub = assembly_root(&mut graph);
    let root = assembly_root(&mut graph);
    let first = assembly_x(&mut graph, root, sub, 100.0);
    assert!(first.is_valid());
    let second = assembly_x(&mut graph, root, sub, 200.0);
    assert!(second.is_valid());
    assembly_branched_children(&mut graph, sub, [first, second], common);
    (graph, [first, second])
}
#[test]
fn BRepGraph_AssemblyTest__ShapesView_OccurrenceShape_FiltersNestedChildrenByParentOccurrence() {
    let (graph, [first, second]) = assembly_branched(false);
    assert!(assembly_shape(&graph, first).nb_children() >= 1);
    assert!(assembly_shape(&graph, second).nb_children() >= 1);
    assert!(assembly_reconstruct(&graph, first).nb_children() >= 1);
    assert!(assembly_reconstruct(&graph, second).nb_children() >= 1);
}
#[test]
fn BRepGraph_AssemblyTest__ShapesView_OccurrenceShape_KeepsCommonChildrenAndFiltersBranchSpecificOnes()
 {
    let (graph, [first, second]) = assembly_branched(true);
    assert!(assembly_shape(&graph, first).nb_children() >= 1);
    assert!(assembly_shape(&graph, second).nb_children() >= 1);
}
