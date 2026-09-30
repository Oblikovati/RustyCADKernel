fn mutation_face_vertex(graph: &BRepGraph) -> VertexId {
    let outer = ref_test_tools::outer_wire_of_face(graph, FaceId::start()).unwrap();
    assert!(outer.is_valid());
    let coedges = ref_test_tools::co_edges_of_wire(graph, outer);
    assert!(!coedges.is_empty());
    let coedge = graph.topo().co_edges().definition(coedges[0]);
    let reference = edge::start_vertex_id(graph, coedge.child_edge_id);
    assert!(reference.is_valid());
    graph.refs().vertices().entry(reference).child_vertex_id
}
fn shift_guarded_vertex(graph: &mut BRepGraph, id: VertexId, old: Point3) {
    let mut guard = graph.editor().vertices().mutate(id).unwrap();
    VertexOps::set_point_guarded(&mut guard, Point3::new(old.x(), old.y(), old.z() + 5.0));
}
#[test]
fn BRepGraph_ReconstructTest__AfterVertexMutation_ModifiedFlagAndPointChanged() {
    let mut graph = reconstruct_graph(&reconstruct_box());
    let id = mutation_face_vertex(&graph);
    let old = vertex::pnt(&graph, id);
    shift_guarded_vertex(&mut graph, id, old);
    assert!(graph.topo().vertices().definition(id).base.own_gen > 0);
    assert!(old.distance(&vertex::pnt(&graph, id)) > precision::confusion());
}
#[test]
fn BRepGraph_ReconstructTest__AfterToleranceMutation_NewTShape() {
    let mut graph = reconstruct_graph(&reconstruct_box());
    let id = EdgeId::new(0);
    let before = graph.shapes().shape(id.into()).unwrap();
    {
        let mut guard = graph.editor().edges().mutate(EdgeId::start()).unwrap();
        let tolerance = guard.tolerance;
        EdgeOps::set_tolerance_guarded(&mut guard, tolerance + 1.0);
    }
    let after = graph.shapes().shape(id.into()).unwrap();
    assert!(!after.is_null());
    assert!(!after.is_same(&before));
}
fn volume_pair_matches(actual: [f64; 2], expected: [f64; 2]) -> bool {
    (actual[0] - expected[0]).abs() < expected[0] * 0.01
        && (actual[1] - expected[1]).abs() < expected[1] * 0.01
}
#[test]
fn BRepGraph_ReconstructTest__CompoundRoot_TwoSolids_Preserved() {
    let first = reconstruct_box();
    let second = support::make_box(5.0, 10.0, 15.0).into_shape();
    let mut compound = Compound::default();
    let builder = BRepBuilder::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &first).unwrap();
    builder.add(&mut compound, &second).unwrap();
    let graph = reconstruct_graph(&compound);
    assert_eq!(graph.topo().solids().nb(), 2);
    assert_compound_volumes(&graph, &first, &second);
}
fn assert_compound_volumes(graph: &BRepGraph, first: &Shape, second: &Shape) {
    let expected = [reconstruction_volume(first), reconstruction_volume(second)];
    let actual = [0, 1].map(|i| {
        let shape = graph
            .shapes()
            .reconstruct(NodeId::new(NodeKind::SOLID, i))
            .unwrap();
        reconstruction_volume(&shape)
    });
    assert!(
        volume_pair_matches(actual, expected)
            || volume_pair_matches(actual, [expected[1], expected[0]])
    );
}
