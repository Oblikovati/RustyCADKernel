fn geometry_source(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph.shapes_mut().add(shape).unwrap();
    assert!(!graph.is_empty());
    graph
}
fn geometry_box(cube: bool) -> BRepGraph {
    let sizes = if cube {
        (10.0, 10.0, 10.0)
    } else {
        (10.0, 20.0, 30.0)
    };
    geometry_source(&support::make_box(sizes.0, sizes.1, sizes.2))
}
fn geometry_sphere() -> BRepGraph {
    let mut maker = BRepPrimAPIMakeSphere::new(15.0).unwrap();
    geometry_source(maker.shape().unwrap())
}
fn geometry_cylinder(height: f64) -> BRepGraph {
    let mut maker = BRepPrimAPIMakeCylinder::new(5.0, height);
    geometry_source(maker.shape().unwrap())
}
fn geometry_compound(first: &Shape, second: &Shape) -> Shape {
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    BRepBuilder::default().add(&mut compound, first).unwrap();
    BRepBuilder::default().add(&mut compound, second).unwrap();
    compound.into_shape()
}
fn geometry_two_components() -> BRepGraph {
    let first = support::make_box(10.0, 10.0, 10.0);
    let mut second =
        BRepPrimAPIMakeBox::at_point(Point3::new(50.0, 50.0, 50.0), 60.0, 60.0, 60.0).unwrap();
    geometry_source(&geometry_compound(&first, second.shape().unwrap()))
}
fn geometry_same_domain(graph: &BRepGraph, id: FaceId) -> Vec<FaceId> {
    let surface = face::surface(graph, id);
    if surface.is_null() {
        return Vec::new();
    }
    geometry_ids::<FaceTag>(graph)
        .into_iter()
        .filter(|other| *other != id && face::surface(graph, *other) == surface)
        .collect()
}
fn geometry_component_root(graph: &BRepGraph, id: FaceId) -> NodeId {
    for kind in [NodeKind::SOLID, NodeKind::SHELL] {
        let explorer = ParentExplorer::with_target(graph, id.into(), kind).unwrap();
        if explorer.more() {
            return explorer.current().def_id;
        }
    }
    id.into()
}
fn geometry_component_counts(graph: &BRepGraph) -> HashMap<NodeId, u32> {
    let mut counts = HashMap::new();
    for id in geometry_ids::<FaceTag>(graph) {
        *counts
            .entry(geometry_component_root(graph, id))
            .or_insert(0) += 1;
    }
    counts
}
fn geometry_first_coedge(graph: &BRepGraph) -> (EdgeId, CoEdgeId) {
    for id in geometry_ids::<EdgeTag>(graph) {
        if let Some(coedge) = graph.topo().edges().co_edges(id).first() {
            return (id, *coedge);
        }
    }
    (EdgeId::default(), CoEdgeId::default())
}
fn geometry_same_range(graph: &BRepGraph, id: EdgeId) -> bool {
    graph
        .topo()
        .edges()
        .relations(id)
        .co_edge_ids
        .iter()
        .all(|id| co_edge::same_range(graph, *id))
}
fn geometry_same_parameter(graph: &BRepGraph, id: EdgeId) -> bool {
    graph
        .topo()
        .edges()
        .relations(id)
        .co_edge_ids
        .iter()
        .all(|id| co_edge::same_parameter(graph, *id).unwrap())
}
fn geometry_param_near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= precision::p_confusion(),
        "actual {actual}, expected {expected} within PConfusion"
    );
}

fn geometry_ids<T: rustycad_tkbrep::brep_graph_inc::NodeTag>(graph: &BRepGraph) -> Vec<TypedId<T>> {
    let mut iterator = DefIterator::<T>::new(graph);
    let mut ids = Vec::new();
    while iterator.more() {
        ids.push(iterator.current_id());
        iterator.next();
    }
    ids
}
