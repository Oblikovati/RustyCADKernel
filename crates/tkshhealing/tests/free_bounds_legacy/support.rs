struct BoundaryFixture {
    edges: Vec<Shape>,
    loop_wire: Shape,
    internal: Shape,
    external: Shape,
    empty: Shape,
}
fn boundary_fixture() -> BoundaryFixture {
    let mut edges = boundary_fixture_edges();
    let internal = edges[3].oriented(Orientation::Internal);
    let external = edges[3].oriented(Orientation::External);
    edges.truncate(3);
    edges.push(internal.clone());
    edges.push(external.clone());
    BoundaryFixture {
        loop_wire: boundary_wire(&edges[..3]),
        internal: boundary_wire(&[internal]),
        external: boundary_wire(&[external]),
        empty: boundary_wire(&[]),
        edges,
    }
}
fn boundary_wire(edges: &[Shape]) -> Shape {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for edge in edges {
        builder.add(&mut wire, edge).unwrap();
    }
    wire.into_shape()
}
fn check_boundary_result(result: &[Shape], expected: (usize, usize, usize, usize, bool)) {
    assert_eq!(result.len(), expected.0);
    let mut totals = (0, 0, 0, false);
    for wire in result {
        count_boundary_wire(wire, &mut totals);
    }
    assert_eq!(totals.0, expected.1);
    assert_eq!(totals.1, expected.2);
    assert_eq!(totals.2, expected.3);
    assert_eq!(totals.3, expected.4);
}
fn count_boundary_wire(wire: &Shape, totals: &mut (usize, usize, usize, bool)) {
    let mut count = 0;
    for edge in Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        count += 1;
        totals.0 += 1;
        if edge.orientation() == Orientation::Internal {
            totals.1 += 1;
        } else if edge.orientation() == Orientation::External {
            totals.2 += 1;
        }
    }
    if count == 3 && wire.closed().unwrap() {
        totals.3 = true;
    }
}
fn check_boundary_wires(mut wires: Vec<Shape>, expected: (usize, usize, usize, usize, bool)) {
    let result =
        ShapeAnalysisFreeBounds::connect_wires_to_wires(&mut wires, precision::confusion(), true)
            .unwrap();
    check_boundary_result(&result, expected);
}
fn check_boundary_edges(mut edges: Vec<Shape>) {
    let result =
        ShapeAnalysisFreeBounds::connect_edges_to_wires(&mut edges, precision::confusion(), true)
            .unwrap();
    check_boundary_result(&result, (1, 3, 0, 0, true));
}
fn square_boundary_face(offset: f64) -> Face {
    let points = [
        Point3::new(offset, offset, 0.),
        Point3::new(offset + 1., offset, 0.),
        Point3::new(offset + 1., offset + 1., 0.),
        Point3::new(offset, offset + 1., 0.),
    ];
    let mut wire = BRepBuilderAPIMakeWire::new();
    for index in 0..4 {
        let edge = BRepBuilderAPIMakeEdge::from_points(points[index], points[(index + 1) % 4])
            .unwrap()
            .edge()
            .unwrap();
        wire.add_edge(&edge).unwrap();
    }
    BRepBuilderAPIMakeFace::from_wire(&wire.wire().unwrap(), false)
        .unwrap()
        .face()
        .unwrap()
}

fn boundary_fixture_edges() -> Vec<Shape> {
    let points = [
        Point3::new(0., 0., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(0., 1., 0.),
        Point3::new(2., -1., 0.),
    ];
    let vertices = points.map(boundary_vertex);
    let mut edges = Vec::new();
    for (first, last) in [(0, 1), (1, 2), (2, 0), (0, 3)] {
        let maker =
            BRepBuilderAPIMakeEdge::from_vertices(&vertices[first], &vertices[last]).unwrap();
        assert!(maker.is_done());
        edges.push(maker.edge().unwrap().into_shape());
    }
    edges
}

fn boundary_vertex(point: Point3) -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, point, precision::confusion())
        .unwrap();
    vertex
}
