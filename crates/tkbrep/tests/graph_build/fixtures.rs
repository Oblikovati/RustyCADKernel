fn build_graph(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph.shapes_mut().add(shape).unwrap();
    graph
}
fn build_shape(maker: &mut impl BRepBuilderAPIMakeShape) -> Shape {
    maker.shape().unwrap().clone()
}
fn build_box(dx: f64, dy: f64, dz: f64) -> Shape {
    build_shape(&mut BRepPrimAPIMakeBox::from_sizes(dx, dy, dz).unwrap())
}
fn build_sphere(radius: f64) -> Shape {
    build_shape(&mut BRepPrimAPIMakeSphere::new(radius).unwrap())
}
fn build_compound(shapes: &[Shape]) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for shape in shapes {
        builder.add(&mut compound, shape).unwrap();
    }
    compound.into_shape()
}
fn build_map(shape: &Shape, kind: ShapeEnum) -> IndexedMapOfShape {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut map).unwrap();
    map
}
fn assert_build_counts(graph: &BRepGraph, shape: &Shape, kinds: &[ShapeEnum]) {
    for kind in kinds {
        let count = match kind {
            ShapeEnum::Solid => graph.topo().solids().nb(),
            ShapeEnum::Shell => graph.topo().shells().nb(),
            ShapeEnum::Face => graph.topo().faces().nb(),
            ShapeEnum::Wire => graph.topo().wires().nb(),
            ShapeEnum::Edge => graph.topo().edges().nb(),
            ShapeEnum::Vertex => graph.topo().vertices().nb(),
            _ => panic!(
                "unexpected kind {kind:?}; expected solid, shell, face, wire, edge or vertex"
            ),
        };
        assert_eq!(count as usize, build_map(shape, *kind).extent());
    }
}
fn has_surface_type(graph: &BRepGraph, name: &str) -> bool {
    let mut faces = FaceIterator::new(graph);
    let mut found = false;
    while faces.more() {
        let surface = face::surface(graph, faces.current_id());
        if let Some(surface) = surface.get() {
            found |= surface.dynamic_type().name() == name;
        }
        faces.next();
    }
    found
}
fn assert_planar_surfaces(graph: &BRepGraph) {
    let mut faces = FaceIterator::new(graph);
    while faces.more() {
        let surface = face::surface(graph, faces.current_id());
        assert!(!surface.is_null());
        assert_eq!(surface.get().unwrap().dynamic_type().name(), "Geom_Plane");
        faces.next();
    }
}
fn build_degenerate_count(graph: &BRepGraph) -> usize {
    let mut edges = EdgeIterator::new(graph);
    let mut count = 0;
    while edges.more() {
        count += usize::from(edge::degenerated(graph, edges.current_id()));
        edges.next();
    }
    count
}
fn flat_add(graph: &mut BRepGraph, shape: &Shape) {
    let options = Options {
        create_auto_product: false,
        flatten: true,
        parallel: false,
        ..Options::default()
    };
    graph
        .shapes_mut()
        .add_with_options(shape, &options)
        .unwrap();
}
fn first_build_face(shape: &Shape) -> Shape {
    let explorer = topexp::Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    assert!(explorer.more());
    topods::face(explorer.current()).unwrap().into_shape()
}
fn build_standalone_edge(first: f64, last: f64) -> Shape {
    build_shape(
        &mut BRepBuilderAPIMakeEdge::from_points(
            Point3::new(first, 0.0, 0.0),
            Point3::new(last, 0.0, 0.0),
        )
        .unwrap(),
    )
}
fn build_standalone_vertex(x: f64) -> Shape {
    build_shape(&mut BRepBuilderAPIMakeVertex::new(Point3::new(x, 0.0, 0.0)).unwrap())
}
fn build_standalone_wire() -> Shape {
    let edge = topods::edge(&build_standalone_edge(0.0, 10.0)).unwrap();
    build_shape(&mut BRepBuilderAPIMakeWire::from_edge(&edge).unwrap())
}
