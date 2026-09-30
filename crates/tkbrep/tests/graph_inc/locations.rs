fn inc_translation(x: f64, y: f64, z: f64) -> Transform3 {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(x, y, z));
    transform
}
#[test]
fn Compound_TranslatedChildren_VolumePreserved() {
    let box1 = support::make_box(10.0, 10.0, 10.0);
    let box2 = support::make_box(10.0, 10.0, 10.0);
    let translated =
        BRepBuilderAPITransform::with_shape(&box2, inc_translation(100.0, 0.0, 0.0), false, false)
            .unwrap()
            .shape()
            .unwrap()
            .clone();
    let compound = inc_compound(&[box1.into_shape(), translated]);
    let original = inc_volume(&compound);
    inc_near(original, 2000.0);
    let graph = inc_populate(&compound, false);
    let recon = inc_reconstruct(&graph, CompoundId::start());
    inc_near(inc_volume(&recon), original);
    assert_eq!(inc_subshapes(&recon, ShapeEnum::Solid).len(), 2);
    assert_eq!(inc_subshapes(&recon, ShapeEnum::Face).len(), 12);
}
#[test]
fn Populate_AppliesNormalChildLocationsToDefinitions() {
    let edge = support::make_edge(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let moved = edge
        .moved(&Location::new(inc_translation(10.0, 0.0, 0.0)), false)
        .unwrap();
    let wire = inc_wire(&[edge.into_shape(), moved]);
    let graph = inc_populate(&wire, false);
    assert_eq!(graph.topo().wires().nb(), 1);
    assert_eq!(graph.topo().edges().nb(), 2);
    assert_eq!(graph.topo().vertices().nb(), 4);
    inc_definition_x_bounds(&graph);
    let recon = inc_reconstruct(&graph, WireId::start());
    assert_eq!(inc_subshapes(&recon, ShapeEnum::Edge).len(), 2);
    assert_eq!(inc_subshapes(&recon, ShapeEnum::Vertex).len(), 4);
}
fn inc_stacked_solid() -> Shape {
    let face = BRepBuilderAPIMakeFace::from_wire(&inc_stacked_wire(), false)
        .unwrap()
        .face()
        .unwrap();
    let face = inc_shift(&face, 0.0, 0.0, 5.0);
    let shell = inc_shift(&inc_shell(&[face]), 7.0, 0.0, 0.0);
    inc_shift(&inc_solid(&[shell]), 0.0, 11.0, 0.0)
}
fn inc_identity_refs(graph: &BRepGraph) {
    let counts = [
        (RefKind::SHELL, graph.refs().shells().nb()),
        (RefKind::FACE, graph.refs().faces().nb()),
        (RefKind::WIRE, graph.refs().wires().nb()),
        (RefKind::VERTEX, graph.refs().vertices().nb()),
        (RefKind::SOLID, graph.refs().solids().nb()),
    ];
    for (kind, count) in counts {
        for i in 0..count {
            assert!(
                graph
                    .refs()
                    .generic()
                    .local_location(RefId::new(kind, i))
                    .is_equal(&Location::default())
            );
        }
    }
}
#[test]
fn Populate_AppliesStackedNormalLocationsToDefinitions() {
    let graph = inc_populate(&inc_stacked_solid(), false);
    let counts = inc_counts(&graph);
    assert_eq!(
        [counts[5], counts[4], counts[3], counts[2], counts[1]],
        [1, 1, 1, 1, 4]
    );
    inc_identity_refs(&graph);
    let recon = inc_reconstruct(&graph, SolidId::start());
    inc_reconstructed_bounds(&recon);
}

fn inc_definition_x_bounds(graph: &BRepGraph) {
    let points: Vec<f64> = (0..graph.topo().vertices().nb())
        .map(|i| {
            let def = graph.topo().vertices().definition(VertexId::new(i));
            def.point.x()
        })
        .collect();
    inc_near(points.iter().copied().fold(f64::MAX, f64::min), 0.0);
    inc_near(points.iter().copied().fold(-f64::MAX, f64::max), 11.0);
}
fn inc_shift(shape: &Shape, x: f64, y: f64, z: f64) -> Shape {
    shape
        .moved(&Location::new(inc_translation(x, y, z)), false)
        .unwrap()
}
fn inc_stacked_wire() -> Wire {
    let points =
        [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].map(|(x, y)| Point3::new(x, y, 0.0));
    let mut maker = BRepBuilderAPIMakeWire::new();
    for i in 0..4 {
        let edge = support::make_edge(points[i], points[(i + 1) % 4]);
        let moved = inc_shift(&edge, 2.0, 0.0, 0.0);
        maker.add_edge(&topods::edge(&moved).unwrap()).unwrap();
    }
    assert!(maker.is_done());
    topods::wire(&inc_shift(&maker.wire().unwrap(), 0.0, 3.0, 0.0)).unwrap()
}
fn inc_reconstructed_bounds(shape: &Shape) {
    let points: Vec<Point3> = inc_subshapes(shape, ShapeEnum::Vertex)
        .iter()
        .map(|s| BRepTool::pnt(&topods::vertex(s).unwrap()).unwrap())
        .collect();
    for (axis, min, max) in [(1, 9.0, 10.0), (2, 14.0, 15.0), (3, 5.0, 5.0)] {
        let coordinates: Vec<f64> = points.iter().map(|p| p.coord(axis).unwrap()).collect();
        inc_near(coordinates.iter().copied().fold(f64::MAX, f64::min), min);
        inc_near(coordinates.iter().copied().fold(-f64::MAX, f64::max), max);
    }
}
