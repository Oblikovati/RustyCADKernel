fn synthetic_cylinder_wires(graph: &mut BRepGraph) -> [WireId; 3] {
    let vertices = synthetic_vertices(
        graph,
        [Point3::new(5.0, 0.0, 0.0), Point3::new(5.0, 0.0, 15.0)],
    );
    let z = synthetic_direction(0.0, 0.0, 1.0);
    let bottom = synthetic_circle(graph, vertices[0], Point3::default(), z, 5.0);
    let top = synthetic_circle(graph, vertices[1], Point3::new(0.0, 0.0, 15.0), z, 5.0);
    let seam = synthetic_line(graph, vertices[0], vertices[1], 15.0);
    let forward = ParityOrientation::FORWARD;
    let reversed = ParityOrientation::REVERSED;
    [
        synthetic_wire(graph, &[(bottom, forward), (seam, reversed)]),
        synthetic_wire(graph, &[(top, forward), (seam, forward)]),
        synthetic_wire(graph, &[(seam, forward), (seam, reversed)]),
    ]
}
#[test]
fn BRepGraph_ValidateTest__Synthetic_Cylinder_AuditClean() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let wires = synthetic_cylinder_wires(&mut graph);
    let bottom = synthetic_plane(Point3::default(), synthetic_direction(0.0, 0.0, -1.0));
    let top = synthetic_plane(
        Point3::new(0.0, 0.0, 15.0),
        synthetic_direction(0.0, 0.0, 1.0),
    );
    let frame = synthetic_frame(Point3::default(), synthetic_direction(0.0, 0.0, 1.0));
    let lateral = Handle::new(GeomCylindricalSurface::new(frame, 5.0).unwrap()).into_cast();
    let faces = [
        synthetic_face(&mut graph, &bottom, wires[0]),
        synthetic_face(&mut graph, &top, wires[1]),
        synthetic_face(&mut graph, &lateral, wires[2]),
    ];
    synthetic_solid(&mut graph, &faces);
    assert!(audit_graph(&graph).is_valid());
}
#[test]
fn BRepGraph_ValidateTest__Synthetic_Sphere_AuditClean() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let wire = synthetic_sphere_wire(&mut graph);
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 8.0).unwrap()).into_cast();
    let face = synthetic_face(&mut graph, &sphere, wire);
    synthetic_solid(&mut graph, &[face]);
    assert!(audit_graph(&graph).is_valid());
}

fn synthetic_sphere_wire(graph: &mut BRepGraph) -> WireId {
    let vertex = graph
        .editor()
        .vertices()
        .add(Point3::new(8.0, 0.0, 0.0), 1e-7);
    let normal = synthetic_direction(0.0, 1.0, 0.0);
    let seam = synthetic_circle(graph, vertex, Point3::default(), normal, 8.0);
    synthetic_wire(
        graph,
        &[
            (seam, ParityOrientation::FORWARD),
            (seam, ParityOrientation::REVERSED),
        ],
    )
}
