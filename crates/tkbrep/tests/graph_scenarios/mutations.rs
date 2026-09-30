fn scenario_move_first_vertex(graph: &mut BRepGraph) -> Point3 {
    assert!(graph.topo().vertices().nb() > 0);
    let old = vertex::pnt(graph, VertexId::start());
    let changed = Point3::new(old.x() + 50.0, old.y(), old.z());
    let mut guard = graph.editor().vertices().mutate(VertexId::start()).unwrap();
    VertexOps::set_point_guarded(&mut guard, changed);
    changed
}
fn scenario_find_changed_vertex(shape: &Shape, changed: Point3) -> bool {
    Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Shape)
        .unwrap()
        .any(|shape| {
            let point = BRepTool::pnt(&topods::vertex(&shape).unwrap()).unwrap();
            point.square_distance(&changed) < precision::square_confusion()
        })
}
fn scenario_vertex_round_trip(graph: &BRepGraph, original: &Shape, changed: Point3, area: f64) {
    let reconstructed = scenario_reconstruct(graph, SolidId::start());
    assert_eq!(reconstructed.shape_type().unwrap(), ShapeEnum::Solid);
    // Native areas are surface-driven, so moving a vertex does not change them.
    scenario_near(scenario_area(&reconstructed), area, area * 0.01);
    scenario_near(
        vertex::pnt(graph, VertexId::start()).x(),
        changed.x(),
        precision::confusion(),
    );
    assert!(scenario_find_changed_vertex(&reconstructed, changed));
    let round_trip = scenario_populated(&reconstructed);
    scenario_valid(&round_trip);
    scenario_counts_from_shape(&round_trip, original);
}
#[test]
fn BRepGraph_ScenarioMatrix__Box_MutateVertex_ValidateReconstructPopulateRoundTrip() {
    let original = support::make_box(10.0, 20.0, 30.0);
    let area = scenario_area(&original);
    let mut graph = scenario_graph(&original);
    scenario_valid(&graph);
    let changed = scenario_move_first_vertex(&mut graph);
    scenario_valid(&graph);
    scenario_vertex_round_trip(&graph, &original, changed, area);
}
#[test]
fn BRepGraph_ScenarioMatrix__Cylinder_SeamEdge_MutationAndBothSubsystemsConsistent() {
    let original = scenario_cylinder(5.0, 15.0);
    let mut graph = scenario_graph(&original);
    let baseline = scenario_populated(&original);
    let seam = scenario_first_seam(&baseline);
    assert!(seam.index < graph.topo().edges().nb());
    scenario_edge_tolerance(&mut graph, seam, 0.5);
    scenario_valid(&graph);
    let reconstructed = scenario_reconstruct(&graph, SolidId::start());
    let round_trip = scenario_populated(&reconstructed);
    assert_eq!(
        round_trip.topo().vertices().nb(),
        baseline.topo().vertices().nb()
    );
    scenario_equal_faces_edges(&round_trip, &baseline);
    assert!(!scenario_seams(&round_trip).is_empty());
    scenario_valid(&round_trip);
}
fn scenario_box_cylinder() -> (Shape, f64) {
    let cuboid = support::make_box(6.0, 6.0, 6.0).into_shape();
    let cylinder = scenario_cylinder(3.0, 10.0);
    let area = scenario_area(&cuboid) + scenario_area(&cylinder);
    (scenario_compound(&[cuboid, cylinder]), area)
}
fn scenario_compound_round_trip(
    graph: &BRepGraph,
    baseline: &BRepGraph,
    original: &Shape,
    area: f64,
) {
    let reconstructed = scenario_reconstruct(graph, CompoundId::start());
    assert_eq!(scenario_count(&reconstructed, ShapeEnum::Solid), 2);
    assert_eq!(
        scenario_count(&reconstructed, ShapeEnum::Face),
        scenario_count(original, ShapeEnum::Face)
    );
    scenario_near(scenario_area(&reconstructed), area, area * 0.01);
    scenario_compound_repopulated(&reconstructed, baseline);
}
#[test]
fn BRepGraph_ScenarioMatrix__Compound_BoxAndCylinder_MutationReconstructAreaRegression() {
    let (original, area) = scenario_box_cylinder();
    let baseline = scenario_populated(&original);
    assert_eq!(baseline.topo().solids().nb(), 2);
    assert_eq!(baseline.topo().compounds().nb(), 1);
    assert!(!scenario_seams(&baseline).is_empty());
    let mut graph = scenario_graph(&original);
    assert_eq!(graph.topo().solids().nb(), 2);
    scenario_valid(&graph);
    assert!(graph.topo().faces().nb() > 0);
    scenario_face_tolerance(&mut graph, 0.5);
    scenario_valid(&graph);
    scenario_compound_round_trip(&graph, &baseline, &original, area);
}

fn scenario_compound_repopulated(reconstructed: &Shape, baseline: &BRepGraph) {
    let round_trip = scenario_populated(reconstructed);
    assert_eq!(
        round_trip.topo().solids().nb(),
        baseline.topo().solids().nb()
    );
    scenario_equal_faces_edges(&round_trip, baseline);
    assert_eq!(
        scenario_seams(&round_trip).len(),
        scenario_seams(baseline).len()
    );
    scenario_valid(&round_trip);
}
