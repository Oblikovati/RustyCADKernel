fn inc_populate(shape: &Shape, parallel: bool) -> BRepGraph {
    let mut graph = BRepGraph::new();
    let status = populate::perform(&mut graph, shape, parallel, &populate::Options).unwrap();
    assert_ne!(status, populate::BuildStatus::Failed);
    assert!(!graph.is_empty());
    graph
}
fn inc_add(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph.shapes_mut().add(shape).unwrap();
    assert!(!graph.is_empty());
    graph
}
fn inc_box() -> Shape {
    support::make_box(10.0, 20.0, 30.0).into_shape()
}
fn inc_cylinder(height: f64) -> Shape {
    BRepPrimAPIMakeCylinder::new(5.0, height)
        .shape()
        .unwrap()
        .clone()
}
fn inc_sphere() -> Shape {
    BRepPrimAPIMakeSphere::new(8.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}
fn inc_reconstruct(graph: &BRepGraph, node: impl Into<NodeId>) -> Shape {
    let shape = reconstruct::node(graph, node.into()).unwrap();
    assert!(!shape.is_null());
    shape
}
fn inc_area(shape: &Shape) -> f64 {
    let mut props = GProps::new();
    BRepGProp::surface_properties(shape, &mut props, false, false).unwrap();
    props.mass()
}
fn inc_volume(shape: &Shape) -> f64 {
    let mut props = GProps::new();
    BRepGProp::volume_properties(shape, &mut props, false, false, false).unwrap();
    props.mass()
}
fn inc_near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= precision::confusion(),
        "{actual} expected {expected} within Confusion"
    );
}
fn inc_subshapes(shape: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut result = Vec::new();
    while explorer.more() {
        result.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    result
}
fn inc_compound(children: &[Shape]) -> Shape {
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    for child in children {
        BRepBuilder::default().add(&mut compound, child).unwrap();
    }
    compound.into_shape()
}
fn inc_counts(graph: &BRepGraph) -> [u32; 6] {
    let topo = graph.topo();
    [
        topo.vertices().nb(),
        topo.edges().nb(),
        topo.wires().nb(),
        topo.faces().nb(),
        topo.shells().nb(),
        topo.solids().nb(),
    ]
}
fn inc_roundtrip_measure(shape: &Shape, measure: fn(&Shape) -> f64) {
    let original = measure(shape);
    let graph = inc_populate(shape, false);
    let recon = inc_reconstruct(&graph, SolidId::start());
    inc_near(measure(&recon), original);
}
fn inc_roundtrip_counts(shape: &Shape) {
    let graph = inc_populate(shape, false);
    let recon = inc_reconstruct(&graph, SolidId::start());
    for kind in [
        ShapeEnum::Face,
        ShapeEnum::Wire,
        ShapeEnum::Edge,
        ShapeEnum::Vertex,
    ] {
        assert_eq!(
            inc_subshapes(&recon, kind).len(),
            inc_subshapes(shape, kind).len()
        );
    }
}
fn inc_lightweight(graph: &BRepGraph) {
    assert!(validate::perform_with_options(graph, &validate::Options::lightweight()).is_valid());
}
