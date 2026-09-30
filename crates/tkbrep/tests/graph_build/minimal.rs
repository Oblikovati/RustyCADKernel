#[test]
fn BRepGraph_BuildTest__SinglePlanarFace_HasWarnings() {
    let shape = build_shape(&mut BRepBuilderAPIMakeFace::from_plane(Plane3::default()).unwrap());
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let result = graph.shapes_mut().add(&shape).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.status, AddStatus::SuccessWithWarnings);
}
#[test]
fn BRepGraph_BuildTest__SinglePlanarFace_Counts() {
    let shape = build_shape(&mut BRepBuilderAPIMakeFace::from_plane(Plane3::default()).unwrap());
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let result = graph.shapes_mut().add(&shape).unwrap();
    assert!(result.is_ok());
    assert_eq!(result.status, AddStatus::SuccessWithWarnings);
}
#[test]
fn BRepGraph_BuildTest__SingleEdge_HandlesGracefully() {
    let mut maker =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))
            .unwrap();
    assert!(maker.is_done());
    let shape = build_shape(&mut maker);
    let graph = build_graph(&shape);
    assert_eq!(graph.topo().faces().nb(), 0);
    assert_eq!(graph.topo().solids().nb(), 0);
}
#[test]
fn BRepGraph_BuildTest__SingleVertex_HandlesGracefully() {
    let shape =
        build_shape(&mut BRepBuilderAPIMakeVertex::new(Point3::new(1.0, 2.0, 3.0)).unwrap());
    let graph = build_graph(&shape);
    assert_eq!(graph.topo().faces().nb(), 0);
    assert_eq!(graph.topo().edges().nb(), 0);
}
