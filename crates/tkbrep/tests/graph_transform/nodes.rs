#[test]
fn BRepGraph_TransformTest__TransformSingleFace() {
    let graph = transform_box(true);
    assert!(graph.topo().faces().nb() > 0);
    let transform = graph_translation(10.0, 20.0, 30.0);
    let result = transformed_node(
        &graph,
        NodeId::new(NodeKind::FACE, 0),
        &transform,
        MeshPolicy::Drop,
    );
    assert_eq!(result.topo().faces().nb(), 1);
}
#[test]
fn BRepGraph_TransformTest__TransformNode_FaceKind_VertexPointsShifted() {
    let graph = transform_box(false);
    let node = NodeId::new(NodeKind::FACE, 0);
    let source = copy_transform_subgraph(&graph, node);
    let transform = graph_translation(5.0, 10.0, 15.0);
    let result = transformed_node(&graph, node, &transform, MeshPolicy::Drop);
    assert_eq!(result.topo().faces().nb(), 1);
    assert_vertices_transformed(&source, &result, &transform);
}
#[test]
fn BRepGraph_TransformTest__TransformNode_ShellKind() {
    let graph = transform_box(false);
    assert!(graph.topo().shells().nb() >= 1);
    let node = NodeId::new(NodeKind::SHELL, 0);
    let source = copy_transform_subgraph(&graph, node);
    let transform = graph_translation(100.0, 0.0, 0.0);
    let result = transformed_node(&graph, node, &transform, MeshPolicy::Drop);
    assert_eq!(result.topo().shells().nb(), 1);
    assert_eq!(result.topo().faces().nb(), graph.topo().faces().nb());
    assert_vertices_transformed(&source, &result, &transform);
}
#[test]
fn BRepGraph_TransformTest__TransformNode_SolidKind() {
    let graph = transform_box(false);
    assert!(graph.topo().solids().nb() >= 1);
    let node = NodeId::new(NodeKind::SOLID, 0);
    let source = copy_transform_subgraph(&graph, node);
    let transform = graph_translation(0.0, 50.0, 0.0);
    let result = transformed_node(&graph, node, &transform, MeshPolicy::Drop);
    assert_eq!(result.topo().solids().nb(), 1);
    assert_eq!(result.topo().shells().nb(), graph.topo().shells().nb());
    assert_eq!(result.topo().faces().nb(), graph.topo().faces().nb());
    assert_vertices_transformed(&source, &result, &transform);
}
#[test]
fn BRepGraph_TransformTest__TransformNode_VertexKind() {
    let graph = transform_box(false);
    assert!(graph.topo().vertices().nb() >= 1);
    let original = vertex::pnt(&graph, VertexId::start());
    let transform = graph_translation(1.0, 2.0, 3.0);
    let result = transformed_node(
        &graph,
        NodeId::new(NodeKind::VERTEX, 0),
        &transform,
        MeshPolicy::Drop,
    );
    assert_eq!(result.topo().vertices().nb(), 1);
    transform_point_near(
        vertex::pnt(&result, VertexId::start()),
        Point3::new(original.x() + 1.0, original.y() + 2.0, original.z() + 3.0),
    );
}
fn transform_compound_graph() -> BRepGraph {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    builder
        .add(&mut compound, &support::make_box(5.0, 5.0, 5.0))
        .unwrap();
    builder
        .add(&mut compound, &support::make_box(3.0, 3.0, 3.0))
        .unwrap();
    transform_source(&compound, false)
}
fn transform_compsolid_graph() -> BRepGraph {
    let builder = BRepBuilder::default();
    let mut compound = CompSolid::default();
    builder.make_comp_solid(&mut compound);
    builder
        .add(&mut compound, &support::make_box(4.0, 4.0, 4.0))
        .unwrap();
    transform_source(&compound, false)
}
#[test]
fn BRepGraph_TransformTest__TransformNode_CompoundKind() {
    let graph = transform_compound_graph();
    assert!(graph.topo().compounds().nb() >= 1);
    let node = NodeId::new(NodeKind::COMPOUND, 0);
    let source = copy_transform_subgraph(&graph, node);
    let transform = graph_translation(20.0, 0.0, 0.0);
    let result = transformed_node(&graph, node, &transform, MeshPolicy::Drop);
    assert_eq!(result.topo().compounds().nb(), 1);
    assert_eq!(result.topo().solids().nb(), graph.topo().solids().nb());
    assert_eq!(result.topo().faces().nb(), graph.topo().faces().nb());
    assert_vertices_transformed(&source, &result, &transform);
}
#[test]
fn BRepGraph_TransformTest__TransformNode_CompSolidKind() {
    let graph = transform_compsolid_graph();
    assert!(graph.topo().comp_solids().nb() >= 1);
    let node = NodeId::new(NodeKind::COMP_SOLID, 0);
    let source = copy_transform_subgraph(&graph, node);
    let transform = graph_translation(0.0, 0.0, 7.0);
    let result = transformed_node(&graph, node, &transform, MeshPolicy::Drop);
    assert_eq!(result.topo().comp_solids().nb(), 1);
    assert_eq!(result.topo().solids().nb(), graph.topo().solids().nb());
    assert_vertices_transformed(&source, &result, &transform);
}
#[test]
fn BRepGraph_TransformTest__TransformNode_NegativeScale_VertexPointsMirrored() {
    let graph = transform_source(&support::make_box(10.0, 10.0, 10.0), false);
    let node = SolidId::start().into();
    let source = copy_transform_subgraph(&graph, node);
    let mut transform = Transform3::default();
    transform.set_mirror_point(Point3::new(0.0, 0.0, 0.0));
    let result = transformed_node(&graph, node, &transform, MeshPolicy::Drop);
    assert_eq!(result.topo().faces().nb(), graph.topo().faces().nb());
    assert_eq!(result.topo().vertices().nb(), graph.topo().vertices().nb());
    assert_vertices_transformed(&source, &result, &transform);
}
