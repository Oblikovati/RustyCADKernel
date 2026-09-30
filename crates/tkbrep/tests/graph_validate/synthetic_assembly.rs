fn synthetic_triangle_solid(
    graph: &mut BRepGraph,
    x: f64,
    plane: &Handle<dyn GeomSurface>,
) -> SolidId {
    let vertices = synthetic_vertices(
        graph,
        [
            Point3::new(x, 0.0, 0.0),
            Point3::new(x + 10.0, 0.0, 0.0),
            Point3::new(x + 5.0, 10.0, 0.0),
        ],
    );
    let edges = [
        (0, 1, 10.0),
        (1, 2, 125.0_f64.sqrt()),
        (2, 0, 125.0_f64.sqrt()),
    ]
    .map(|(first, last, length)| synthetic_line(graph, vertices[first], vertices[last], length));
    let wire = synthetic_wire(graph, &edges.map(|edge| (edge, ParityOrientation::FORWARD)));
    let face = synthetic_face(graph, plane, wire);
    synthetic_solid(graph, &[face])
}
#[test]
fn BRepGraph_ValidateTest__Synthetic_Compound_AuditClean() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let plane = synthetic_plane(Point3::default(), synthetic_direction(0.0, 0.0, 1.0));
    let first = synthetic_triangle_solid(&mut graph, 0.0, &plane);
    let second = synthetic_triangle_solid(&mut graph, 20.0, &plane);
    graph
        .editor()
        .compounds()
        .add(&[first.into(), second.into()]);
    assert!(audit_graph(&graph).is_valid());
}
fn synthetic_square_solid(graph: &mut BRepGraph) {
    let vertices = synthetic_vertices(
        graph,
        [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
            Point3::new(10.0, 10.0, 0.0),
            Point3::new(0.0, 10.0, 0.0),
        ],
    );
    let edges = [(0, 1), (1, 2), (2, 3), (3, 0)]
        .map(|(first, last)| synthetic_line(graph, vertices[first], vertices[last], 10.0));
    let wire = synthetic_wire(graph, &edges.map(|edge| (edge, ParityOrientation::FORWARD)));
    let plane = Handle::new(GeomPlane::new(Axis3::default())).into_cast();
    let face = synthetic_face(graph, &plane, wire);
    synthetic_solid(graph, &[face]);
}
#[test]
fn BRepGraph_ValidateTest__Synthetic_Assembly_AuditClean() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    synthetic_square_solid(&mut graph);
    // The original test assumes Product[0] without constructing a separate part product.
    let part = ProductId::start();
    let root = graph.editor().products().add_empty();
    graph
        .editor()
        .products()
        .append_document_root(root)
        .unwrap();
    validation_occurrence(&mut graph, root, part, &validation_translation(0.0));
    validation_occurrence(&mut graph, root, part, &validation_translation(20.0));
    graph.editor().commit_mutation();
    assert!(audit_graph(&graph).is_valid());
}
