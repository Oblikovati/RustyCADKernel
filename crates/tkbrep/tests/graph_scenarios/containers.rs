fn scenario_solid_parents(graph: &BRepGraph, exactly_one: bool) {
    for id in (0..graph.topo().solids().nb()).map(SolidId::new) {
        let parents = &graph.topo().solids().relations(id).parent_solid_ref_ids;
        if exactly_one {
            assert_eq!(parents.len(), 1);
        } else {
            assert!(!parents.is_empty());
        }
    }
}
fn scenario_comp_solid_round_trip(graph: &BRepGraph, counts: (u32, u32), baseline: &BRepGraph) {
    let reconstructed = scenario_reconstruct(graph, CompSolidId::start());
    assert_eq!(scenario_count(&reconstructed, ShapeEnum::Face), counts.0);
    assert_eq!(scenario_count(&reconstructed, ShapeEnum::Edge), counts.1);
    assert_eq!(scenario_count(&reconstructed, ShapeEnum::Solid), 2);
    let round_trip = scenario_populated(&reconstructed);
    assert_eq!(round_trip.topo().comp_solids().nb(), 1);
    assert_eq!(
        round_trip.topo().solids().nb(),
        baseline.topo().solids().nb()
    );
    scenario_equal_faces_edges(&round_trip, baseline);
    scenario_valid(&round_trip);
}
#[test]
fn BRepGraph_ScenarioMatrix__CompSolid_TwoBoxes_BothSubsystemsMutateReconstructRoundTrip() {
    let first = support::make_box(4.0, 4.0, 4.0).into_shape();
    let second = support::make_box(3.0, 3.0, 3.0).into_shape();
    let original = scenario_comp_solid(&[first, second]);
    let counts = (
        scenario_count(&original, ShapeEnum::Face),
        scenario_count(&original, ShapeEnum::Edge),
    );
    let baseline = scenario_populated(&original);
    assert_eq!(baseline.topo().comp_solids().nb(), 1);
    assert!(baseline.topo().solids().nb() >= 2);
    scenario_solid_parents(&baseline, false);
    scenario_valid(&baseline);
    let mut graph = scenario_graph(&original);
    scenario_valid(&graph);
    assert!(graph.topo().edges().nb() > 0);
    scenario_edge_tolerance(&mut graph, EdgeId::start(), 1.0);
    scenario_valid(&graph);
    scenario_comp_solid_round_trip(&graph, counts, &baseline);
}
fn scenario_free_edge(first_x: f64, last_x: f64) -> Shape {
    let first = Point3::new(first_x, 0.0, 0.0);
    let last = Point3::new(last_x, 0.0, 0.0);
    let maker = BRepBuilderAPIMakeEdge::from_points(first, last).unwrap();
    assert!(maker.is_done());
    maker.edge().unwrap().clone().into_shape()
}
fn scenario_free_vertex(point: Point3) -> Shape {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, point, precision::confusion())
        .unwrap();
    vertex.into_shape()
}
fn scenario_free_wire() -> Shape {
    let edge = scenario_free_edge(0.0, 10.0);
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge).unwrap();
    wire.into_shape()
}
fn scenario_atomic_counts(graph: &BRepGraph) {
    assert_eq!(graph.topo().compounds().nb(), 1);
    assert!(graph.topo().wires().nb() >= 1);
    assert!(graph.topo().edges().nb() >= 2);
}
#[test]
fn BRepGraph_ScenarioMatrix__Compound_FreeWireFreeEdgeFreeVertex_ValidateAndPopulate() {
    let original = scenario_compound(&[
        scenario_free_wire(),
        scenario_free_edge(20.0, 30.0),
        scenario_free_vertex(Point3::new(50.0, 50.0, 50.0)),
    ]);
    let graph = scenario_graph(&original);
    scenario_atomic_counts(&graph);
    assert!(graph.topo().vertices().nb() >= 1);
    scenario_valid(&graph);
    let populated = scenario_populated(&original);
    scenario_atomic_counts(&populated);
    scenario_valid(&populated);
}
fn scenario_first_subshape(shape: &Shape, kind: ShapeEnum) -> Shape {
    let explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    if explorer.more() {
        return explorer.current().clone();
    }
    Shape::default()
}
fn scenario_mixed_children() -> Shape {
    let cuboid = support::make_box(5.0, 5.0, 5.0).into_shape();
    // Upstream constructs an unused shell-container before selecting the shell.
    let mut unused = Compound::default();
    BRepBuilder::default().make_compound(&mut unused);
    let shell = scenario_first_subshape(&cuboid, ShapeEnum::Shell);
    let face = scenario_first_subshape(&cuboid, ShapeEnum::Face);
    let edge = scenario_free_edge(0.0, 10.0);
    let vertex = scenario_free_vertex(Point3::new(42.0, 13.0, 7.0));
    scenario_compound(&[cuboid, shell, face, edge, vertex])
}
fn scenario_mixed_child_kinds(graph: &BRepGraph) {
    let refs = &graph
        .topo()
        .compounds()
        .relations(CompoundId::start())
        .child_ref_ids;
    let kinds: Vec<NodeKind> = refs
        .iter()
        .map(|id| graph.refs().children().entry(*id).child_node_id.kind)
        .collect();
    assert!(kinds.contains(&NodeKind::SOLID));
    assert!(kinds.contains(&NodeKind::EDGE));
    assert!(kinds.contains(&NodeKind::VERTEX));
    // The original deliberately makes no assertion for shared shell/face child refs.
}
#[test]
fn BRepGraph_ScenarioMatrix__Compound_MixedAtomicChildren_RelationsCoverage() {
    let original = scenario_mixed_children();
    let graph = scenario_graph(&original);
    assert!(graph.topo().compounds().nb() > 0);
    scenario_valid(&graph);
    scenario_mixed_child_kinds(&graph);
    let populated = scenario_populated(&original);
    scenario_valid(&populated);
}
fn scenario_three_boxes() -> Shape {
    let first = support::make_box(5.0, 5.0, 5.0).into_shape();
    let mut second =
        BRepPrimAPIMakeBox::at_point(Point3::new(20.0, 0.0, 0.0), 5.0, 5.0, 5.0).unwrap();
    let mut third =
        BRepPrimAPIMakeBox::at_point(Point3::new(40.0, 0.0, 0.0), 5.0, 5.0, 5.0).unwrap();
    scenario_comp_solid(&[
        first,
        second.shape().unwrap().clone(),
        third.shape().unwrap().clone(),
    ])
}
#[test]
fn BRepGraph_ScenarioMatrix__CompSolid_ThreeBoxes_RelationsPerSolid() {
    let original = scenario_three_boxes();
    let graph = scenario_graph(&original);
    scenario_valid(&graph);
    let populated = scenario_populated(&original);
    assert_eq!(populated.topo().comp_solids().nb(), 1);
    assert!(populated.topo().solids().nb() >= 3);
    scenario_solid_parents(&populated, true);
    scenario_valid(&populated);
}
