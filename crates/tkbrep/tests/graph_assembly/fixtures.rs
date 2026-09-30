fn assembly_source(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph.shapes_mut().add(shape).unwrap();
    graph
}
fn assembly_box() -> BRepGraph {
    let shape = support::make_box(10.0, 10.0, 10.0);
    assembly_source(&shape)
}
fn assembly_root(graph: &mut BRepGraph) -> ProductId {
    let id = graph.editor().products().add_empty();
    graph.editor().products().append_document_root(id).unwrap();
    id
}
fn assembly_location(x: f64, y: f64, z: f64) -> Location {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(x, y, z));
    Location::new(transform)
}
fn assembly_link(
    graph: &mut BRepGraph,
    parent: ProductId,
    part: ProductId,
    location: &Location,
    context: OccurrenceId,
) -> OccurrenceId {
    graph
        .editor()
        .products()
        .append(parent, part, location, context)
        .unwrap()
        .0
}
fn assembly_origin(graph: &mut BRepGraph, parent: ProductId, part: ProductId) -> OccurrenceId {
    assembly_link(
        graph,
        parent,
        part,
        &Location::default(),
        OccurrenceId::default(),
    )
}
fn assembly_x(graph: &mut BRepGraph, parent: ProductId, part: ProductId, x: f64) -> OccurrenceId {
    assembly_link(
        graph,
        parent,
        part,
        &assembly_location(x, 0.0, 0.0),
        OccurrenceId::default(),
    )
}
fn assembly_roots(graph: &BRepGraph) -> Vec<ProductId> {
    let mut iterator = RootProductIterator::new(graph);
    let mut roots = Vec::new();
    while iterator.more() {
        roots.push(iterator.current());
        iterator.next();
    }
    roots
}
fn assembly_remove(graph: &mut BRepGraph, node: impl Into<NodeId>) {
    graph
        .editor()
        .generic()
        .remove_subgraph(node.into())
        .unwrap();
}
fn assembly_remove_node(graph: &mut BRepGraph, node: impl Into<NodeId>) {
    graph.editor().generic().remove_node(node.into()).unwrap();
}
fn assembly_near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= precision::confusion(),
        "actual {actual}, expected {expected} within Confusion"
    );
}
fn assembly_local_translation(graph: &BRepGraph, id: OccurrenceId) -> Vector3 {
    graph
        .topo()
        .occurrences()
        .occurrence_location(id)
        .transformation()
        .translation_part()
        .into()
}
fn assembly_compound(shapes: &[Shape]) -> Shape {
    let mut compound = Compound::default();
    let builder = BRepBuilder::default();
    builder.make_compound(&mut compound);
    for shape in shapes {
        builder.add(&mut compound, shape).unwrap();
    }
    compound.into_shape()
}
fn assembly_sphere() -> Shape {
    let mut maker = BRepPrimAPIMakeSphere::new(5.0).unwrap();
    maker.shape().unwrap().clone()
}
fn assembly_shape(graph: &BRepGraph, node: impl Into<NodeId>) -> Shape {
    let shape = graph.shapes().shape(node.into()).unwrap();
    assert!(!shape.is_null());
    shape
}
fn assembly_reconstruct(graph: &BRepGraph, node: impl Into<NodeId>) -> Shape {
    let shape = graph.shapes().reconstruct(node.into()).unwrap();
    assert!(!shape.is_null());
    shape
}

fn assembly_chain(
    graph: &mut BRepGraph,
    links: &[(ProductId, ProductId, [f64; 3])],
) -> Vec<OccurrenceId> {
    let mut context = OccurrenceId::default();
    let mut occurrences = Vec::new();
    for (parent, part, [x, y, z]) in links {
        let location = assembly_location(*x, *y, *z);
        context = assembly_link(graph, *parent, *part, &location, context);
        occurrences.push(context);
    }
    occurrences
}
fn assembly_child_x(
    graph: &mut BRepGraph,
    sub: ProductId,
    x: f64,
    context: OccurrenceId,
) -> OccurrenceId {
    let location = assembly_location(x, 0.0, 0.0);
    assembly_link(graph, sub, ProductId::start(), &location, context)
}
