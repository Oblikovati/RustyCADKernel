fn scenario_face_bound_fixture(graph: &mut BRepGraph) -> (EdgeId, FaceId) {
    let id = scenario_unbound_edge(graph);
    let coedge = graph
        .editor()
        .co_edges()
        .add(id, ParityOrientation::FORWARD);
    assert!(coedge.is_valid());
    let wire = graph.editor().wires().add(&[coedge]);
    assert!(wire.is_valid());
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_point(Point3::new(0.0, 0.0, 0.0), Direction3::Z).unwrap())
            .into_cast();
    let face = graph.editor().faces().add(&surface, wire, &[], 1e-7);
    assert!(face.is_valid());
    (id, face)
}
fn scenario_face_bound_pcurve(graph: &mut BRepGraph, edge_id: EdgeId, face_id: FaceId) -> CoEdgeId {
    let curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), Direction2::X)).into_cast();
    let id = graph.editor().co_edges().add_with_pcurve(
        edge_id,
        face_id,
        &curve,
        0.0,
        1.0,
        ParityOrientation::REVERSED,
    );
    assert!(id.is_valid());
    id
}
fn scenario_face_bound_queries(graph: &BRepGraph, edge_id: EdgeId, face_id: FaceId, id: CoEdgeId) {
    let def = graph.topo().co_edges().definition(id);
    assert_eq!(def.child_edge_id, edge_id);
    assert_eq!(def.face_id, face_id);
    assert_eq!(def.orientation.orientation(), Orientation::Reversed);
    assert!(def.curve2d_rep_id.is_valid());
    assert_eq!(edge::find_pcurve_co_edge_id(graph, edge_id, face_id), id);
    assert_eq!(
        edge::find_pcurve_co_edge_id_oriented(graph, edge_id, face_id, Orientation::Reversed),
        id
    );
    assert!(graph.topo().edges().co_edges(edge_id).contains(&id));
    assert!(
        graph
            .topo()
            .edges()
            .faces_of(edge_id)
            .any(|id| id == face_id)
    );
}
#[test]
fn BRepGraph_ScenarioMatrix__EditorFaceBoundPCurveCoEdge_RelationsAndLookup() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let (edge_id, face_id) = scenario_face_bound_fixture(&mut graph);
    let id = scenario_face_bound_pcurve(&mut graph, edge_id, face_id);
    scenario_face_bound_queries(&graph, edge_id, face_id, id);
    assert!(graph.validate_relations());
    scenario_valid(&graph);
}
fn scenario_splittable(graph: &BRepGraph, with_refs: bool) -> EdgeId {
    let id = (0..graph.topo().edges().nb())
        .map(EdgeId::new)
        .find(|id| {
            let def = graph.topo().edges().definition(*id);
            !edge::degenerated(graph, *id)
                && def.curve3d_rep_id.is_valid()
                && (!with_refs
                    || (def.start_vertex_ref_id.is_valid() && def.end_vertex_ref_id.is_valid()))
        })
        .unwrap_or_default();
    assert!(id.is_valid());
    id
}
fn scenario_split_box(graph: &mut BRepGraph, id: EdgeId) -> [EdgeId; 2] {
    let range = edge::range(graph, id);
    let vertex = graph
        .editor()
        .vertices()
        .add(Point3::new(5.0, 10.0, 15.0), 1e-7);
    let (first, second) = graph
        .editor()
        .edges()
        .split(id, vertex, 0.5 * (range.0 + range.1))
        .unwrap();
    [first, second]
}
fn scenario_boundary_refs(graph: &BRepGraph, id: EdgeId) -> [VertexRefId; 2] {
    let def = graph.topo().edges().definition(id);
    let refs = [def.start_vertex_ref_id, def.end_vertex_ref_id];
    for id in refs {
        assert!(id.is_valid());
        assert!(!id.is_removed(graph));
    }
    refs
}
#[test]
fn BRepGraph_ScenarioMatrix__BoxEdgeSplit_BoundaryVertexRetirement() {
    let mut graph = scenario_graph(&support::make_box(10.0, 20.0, 30.0));
    let edge = scenario_splittable(&graph, true);
    let refs = scenario_boundary_refs(&graph, edge);
    let [first, second] = scenario_split_box(&mut graph, edge);
    assert!(first.is_valid());
    assert!(second.is_valid());
    assert!(refs[0].is_removed(&graph));
    assert!(refs[1].is_removed(&graph));
    scenario_valid(&graph);
}
#[test]
fn BRepGraph_ScenarioMatrix__BoxEdgeSplit_SubEdgesHaveNoOriginal() {
    let mut graph = scenario_graph(&support::make_box(10.0, 20.0, 30.0));
    let edge = scenario_splittable(&graph, false);
    assert!(graph.shapes().has_original(edge.into()));
    let [first, second] = scenario_split_box(&mut graph, edge);
    assert!(first.is_valid());
    assert!(second.is_valid());
    assert!(!graph.shapes().has_original(first.into()));
    assert!(!graph.shapes().has_original(second.into()));
    assert!(graph.shapes().original(first.into()).is_null());
    assert!(graph.shapes().original(second.into()).is_null());
}
#[test]
fn BRepGraph_ScenarioMatrix__BoxEdgeSplit_ShapeReconstructsSubEdge() {
    let mut graph = scenario_graph(&support::make_box(10.0, 20.0, 30.0));
    let edge = scenario_splittable(&graph, false);
    let [first, second] = scenario_split_box(&mut graph, edge);
    let first = graph.shapes().shape(first.into()).unwrap();
    let second = graph.shapes().shape(second.into()).unwrap();
    assert!(!first.is_null());
    assert!(!second.is_null());
    assert_eq!(first.shape_type().unwrap(), ShapeEnum::Edge);
    assert_eq!(second.shape_type().unwrap(), ShapeEnum::Edge);
}
#[test]
fn BRepGraph_ScenarioMatrix__EditorAddedVertex_HasNoOriginal() {
    let mut graph = scenario_graph(&support::make_box(10.0, 20.0, 30.0));
    let vertex = graph
        .editor()
        .vertices()
        .add(Point3::new(42.0, 42.0, 42.0), 1e-7);
    assert!(vertex.is_valid());
    assert!(!graph.shapes().has_original(vertex.into()));
    assert!(graph.shapes().original(vertex.into()).is_null());
}

fn scenario_unbound_edge(graph: &mut BRepGraph) -> EdgeId {
    let start = Point3::new(0.0, 0.0, 0.0);
    let end = Point3::new(1.0, 0.0, 0.0);
    let first = graph.editor().vertices().add(start, 1e-7);
    let last = graph.editor().vertices().add(end, 1e-7);
    assert!(first.is_valid());
    assert!(last.is_valid());
    let id = graph
        .editor()
        .edges()
        .add(first, last, &Handle::default(), 0.0, 1.0, 1e-7);
    assert!(id.is_valid());
    id
}
