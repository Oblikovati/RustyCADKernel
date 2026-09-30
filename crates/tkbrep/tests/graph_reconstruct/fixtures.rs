fn reconstruct_graph(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph.shapes_mut().add(shape).unwrap();
    assert!(!graph.is_empty());
    graph
}
fn reconstruct_box() -> Shape {
    support::make_box(10.0, 20.0, 30.0).into_shape()
}
fn reconstruction_area(shape: &Shape) -> f64 {
    let mut props = GProps::new();
    BRepGProp::surface_properties(shape, &mut props, false, false).unwrap();
    props.mass()
}
fn reconstruction_volume(shape: &Shape) -> f64 {
    let mut props = GProps::new();
    BRepGProp::volume_properties(shape, &mut props, false, false, false).unwrap();
    props.mass()
}
fn reconstructed_measure(shape: &Shape, measure: fn(&Shape) -> f64) {
    let original = measure(shape);
    let graph = reconstruct_graph(shape);
    let reconstructed = graph
        .shapes()
        .reconstruct(NodeId::new(NodeKind::SOLID, 0))
        .unwrap();
    let result = measure(&reconstructed);
    assert!(
        (result - original).abs() <= original * 0.01,
        "actual {result}, expected {original} within 1%"
    );
}
fn reconstruction_subshapes(shape: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut result = Vec::new();
    while explorer.more() {
        result.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    result
}
fn reconstruction_ids<T: NodeTraits>(graph: &BRepGraph) -> Vec<TypedId<T>> {
    let mut iterator = DefIterator::<T>::new(graph);
    let mut result = Vec::new();
    while iterator.more() {
        result.push(iterator.current_id());
        iterator.next();
    }
    result
}
fn reconstructed_curve(graph: &BRepGraph, id: EdgeId) -> EdgeCurve3D {
    let shape = graph.shapes().reconstruct(id.into()).unwrap();
    BRepTool::curve_with_location(&topods::edge(&shape).unwrap()).unwrap()
}
fn reconstructed_point(graph: &BRepGraph, id: VertexId) -> Point3 {
    let shape = graph.shapes().reconstruct(id.into()).unwrap();
    BRepTool::pnt(&topods::vertex(&shape).unwrap()).unwrap()
}
fn assert_reconstruction_coordinates(actual: Point3, expected: Point3) {
    assert!((actual.x() - expected.x()).abs() <= precision::confusion());
    assert!((actual.y() - expected.y()).abs() <= precision::confusion());
    assert!((actual.z() - expected.z()).abs() <= precision::confusion());
}
fn assert_reconstructed_pcurves(shape: &Shape) {
    let face = topods::face(shape).unwrap();
    for shape in reconstruction_subshapes(shape, ShapeEnum::Edge) {
        let edge = topods::edge(&shape).unwrap();
        if BRepTool::degenerated(&edge).unwrap() {
            continue;
        }
        assert!(
            !BRepTool::curve_on_face(&edge, &face)
                .unwrap()
                .pcurve
                .is_null()
        );
    }
}
