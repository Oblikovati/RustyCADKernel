fn seam_graph(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    // Upstream registerLayers() is intentionally empty at the pinned revision.
    graph.clear().unwrap();
    assert!(graph.shapes_mut().add(shape).unwrap().is_ok());
    graph
}
fn seam_cylinder() -> Shape {
    BRepPrimAPIMakeCylinder::new(5.0, 10.0)
        .shape()
        .unwrap()
        .clone()
}
fn seam_box() -> Shape {
    BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}
fn first_seam_coedge(graph: &BRepGraph) -> CoEdgeId {
    let mut edges = EdgeIterator::new(graph);
    while edges.more() {
        for id in graph.topo().edges().co_edges(edges.current_id()) {
            if co_edge::is_seam(graph, *id) {
                return *id;
            }
        }
        edges.next();
    }
    CoEdgeId::default()
}
fn seam_wire_ids(graph: &BRepGraph) -> Vec<WireId> {
    let mut wires = WireIterator::new(graph);
    let mut ids = Vec::new();
    while wires.more() {
        ids.push(wires.current_id());
        wires.next();
    }
    ids
}
fn seam_explore(shape: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut shapes = Vec::new();
    while explorer.more() {
        shapes.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    shapes
}
fn seam_children(shape: &Shape) -> Vec<Shape> {
    let mut iterator = ShapeIterator::new(shape, false, false);
    let mut children = Vec::new();
    while iterator.more() {
        children.push(iterator.value().unwrap().clone());
        iterator.next();
    }
    children
}
fn wire_child_counts(face: &Shape) -> Vec<usize> {
    seam_children(face)
        .iter()
        .filter(|child| child.shape_type().unwrap() == ShapeEnum::Wire)
        .map(|wire| seam_children(wire).len())
        .collect()
}
fn shape_wire_counts(shape: &Shape) -> Vec<usize> {
    seam_explore(shape, ShapeEnum::Face)
        .iter()
        .flat_map(wire_child_counts)
        .collect()
}
fn seam_free_wire() -> Shape {
    let builder = BRepBuilder::default();
    let mut first = Vertex::default();
    let mut last = Vertex::default();
    builder
        .make_vertex_at(&mut first, Point3::new(0.0, 0.0, 0.0), 1e-7)
        .unwrap();
    builder
        .make_vertex_at(&mut last, Point3::new(1.0, 0.0, 0.0), 1e-7)
        .unwrap();
    let maker = BRepBuilderAPIMakeEdge::from_vertices(&first, &last).unwrap();
    assert!(maker.is_done());
    let wire = BRepBuilderAPIMakeWire::from_edge(&maker.edge().unwrap()).unwrap();
    assert!(wire.is_done());
    wire.wire().unwrap().into_shape()
}
fn assert_no_seam_coedges(graph: &BRepGraph) {
    for i in 0..graph.topo().co_edges().nb() {
        assert!(!co_edge::is_seam(graph, CoEdgeId::new(i)));
    }
}
