fn synthetic_box_edges(graph: &mut BRepGraph) -> [EdgeId; 12] {
    let points = synthetic_box_points();
    let vertices = synthetic_vertices(graph, points);
    let edges = [
        (0, 1, 10.0),
        (1, 2, 20.0),
        (2, 3, 10.0),
        (3, 0, 20.0),
        (4, 5, 10.0),
        (5, 6, 20.0),
        (6, 7, 10.0),
        (7, 4, 20.0),
        (0, 4, 30.0),
        (1, 5, 30.0),
        (2, 6, 30.0),
        (3, 7, 30.0),
    ];
    edges
        .map(|(first, last, length)| synthetic_line(graph, vertices[first], vertices[last], length))
}
fn synthetic_box_wires(graph: &mut BRepGraph, edges: [EdgeId; 12]) -> [WireId; 6] {
    let sequences = [
        [0, 1, 2, 3],
        [4, 5, 6, 7],
        [0, 9, 4, 8],
        [1, 10, 5, 9],
        [2, 11, 6, 10],
        [3, 8, 7, 11],
    ];
    std::array::from_fn(|i| {
        let uses: [(EdgeId, ParityOrientation); 4] = std::array::from_fn(|j| {
            let orientation = if i < 2 || j < 2 {
                ParityOrientation::FORWARD
            } else {
                ParityOrientation::REVERSED
            };
            (edges[sequences[i][j]], orientation)
        });
        synthetic_wire(graph, &uses)
    })
}
fn synthetic_box_faces(graph: &mut BRepGraph, wires: [WireId; 6]) -> [FaceId; 6] {
    let planes = [
        ((0.0, 0.0, 0.0), (0.0, 0.0, -1.0)),
        ((0.0, 0.0, 30.0), (0.0, 0.0, 1.0)),
        ((0.0, 0.0, 0.0), (0.0, -1.0, 0.0)),
        ((10.0, 0.0, 0.0), (1.0, 0.0, 0.0)),
        ((10.0, 20.0, 0.0), (0.0, 1.0, 0.0)),
        ((0.0, 20.0, 0.0), (-1.0, 0.0, 0.0)),
    ];
    std::array::from_fn(|i| {
        let (point, normal) = planes[i];
        let surface = synthetic_plane(
            Point3::new(point.0, point.1, point.2),
            synthetic_direction(normal.0, normal.1, normal.2),
        );
        synthetic_face(graph, &surface, wires[i])
    })
}
#[test]
fn BRepGraph_ValidateTest__Synthetic_Box_AuditClean() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let edges = synthetic_box_edges(&mut graph);
    let wires = synthetic_box_wires(&mut graph, edges);
    let faces = synthetic_box_faces(&mut graph, wires);
    synthetic_solid(&mut graph, &faces);
    assert_valid_no_errors(&audit_graph(&graph));
}

fn synthetic_box_points() -> [Point3; 8] {
    [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(10.0, 0.0, 0.0),
        Point3::new(10.0, 20.0, 0.0),
        Point3::new(0.0, 20.0, 0.0),
        Point3::new(0.0, 0.0, 30.0),
        Point3::new(10.0, 0.0, 30.0),
        Point3::new(10.0, 20.0, 30.0),
        Point3::new(0.0, 20.0, 30.0),
    ]
}
