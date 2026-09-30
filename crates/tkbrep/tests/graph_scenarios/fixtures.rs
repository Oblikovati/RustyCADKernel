fn scenario_graph(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph.shapes_mut().add(shape).unwrap();
    assert!(!graph.is_empty());
    graph
}
fn scenario_populated(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    populate::perform(&mut graph, shape, false, &populate::Options).unwrap();
    assert!(!graph.is_empty());
    graph
}
fn scenario_valid(graph: &BRepGraph) {
    // The original explicitly chooses Lightweight despite its audit comments.
    let result = validate::perform_with_options(graph, &Options::lightweight());
    assert!(
        result.is_valid(),
        "expected structurally valid graph, issues {:?}",
        result.issues
    );
}
fn scenario_count(shape: &Shape, kind: ShapeEnum) -> u32 {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut map).unwrap();
    map.extent() as u32
}
fn scenario_area(shape: &Shape) -> f64 {
    let mut props = GProps::new();
    BRepGProp::surface_properties(shape, &mut props, false, false).unwrap();
    props.mass()
}
fn scenario_near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual {actual}, expected {expected} within {tolerance}"
    );
}
fn scenario_reconstruct(graph: &BRepGraph, id: impl Into<NodeId>) -> Shape {
    let shape = graph.shapes().reconstruct(id.into()).unwrap();
    assert!(!shape.is_null());
    shape
}
fn scenario_cylinder(radius: f64, height: f64) -> Shape {
    let mut maker = BRepPrimAPIMakeCylinder::new(radius, height);
    maker.shape().unwrap().clone()
}
fn scenario_sphere() -> Shape {
    let mut maker = BRepPrimAPIMakeSphere::new(10.0).unwrap();
    maker.shape().unwrap().clone()
}
fn scenario_compound(shapes: &[Shape]) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for shape in shapes {
        builder.add(&mut compound, shape).unwrap();
    }
    compound.into_shape()
}
fn scenario_comp_solid(shapes: &[Shape]) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = CompSolid::default();
    builder.make_comp_solid(&mut compound);
    for shape in shapes {
        builder.add(&mut compound, shape).unwrap();
    }
    compound.into_shape()
}
fn scenario_seams(graph: &BRepGraph) -> Vec<EdgeId> {
    (0..graph.topo().edges().nb())
        .map(EdgeId::new)
        .filter(|edge| {
            graph
                .topo()
                .edges()
                .relations(*edge)
                .co_edge_ids
                .iter()
                .any(|id| co_edge::seam_pair(graph, *id).is_valid())
        })
        .collect()
}
fn scenario_edge_tolerance(graph: &mut BRepGraph, id: EdgeId, increment: f64) {
    let mut guard = graph.editor().edges().mutate(id).unwrap();
    let tolerance = guard.tolerance + increment;
    EdgeOps::set_tolerance_guarded(&mut guard, tolerance);
}
fn scenario_face_tolerance(graph: &mut BRepGraph, increment: f64) {
    let mut guard = graph.editor().faces().mutate(FaceId::start()).unwrap();
    let tolerance = guard.tolerance + increment;
    FaceOps::set_tolerance_guarded(&mut guard, tolerance);
}
fn scenario_equal_faces_edges(actual: &BRepGraph, expected: &BRepGraph) {
    assert_eq!(actual.topo().faces().nb(), expected.topo().faces().nb());
    assert_eq!(actual.topo().edges().nb(), expected.topo().edges().nb());
}

fn scenario_first_seam(graph: &BRepGraph) -> EdgeId {
    let seams = scenario_seams(graph);
    let id = seams.first().copied().unwrap_or_default();
    assert!(id.is_valid());
    id
}
fn scenario_counts_from_shape(graph: &BRepGraph, original: &Shape) {
    let faces = scenario_count(original, ShapeEnum::Face);
    assert_eq!(graph.topo().faces().nb(), faces);
    let edges = scenario_count(original, ShapeEnum::Edge);
    assert_eq!(graph.topo().edges().nb(), edges);
}
