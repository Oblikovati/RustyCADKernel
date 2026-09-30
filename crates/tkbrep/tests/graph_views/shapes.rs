#[test]
fn BRepGraph_ViewsTest__ShapesView_Shape_NonNull() {
    let graph = views_box();
    assert!(
        !graph
            .shapes()
            .shape(FaceId::new(0).into())
            .unwrap()
            .is_null()
    );
}

#[test]
fn BRepGraph_ViewsTest__ShapesView_HasOriginal_True() {
    let graph = views_box();
    assert!(graph.shapes().has_original(FaceId::new(0).into()));
}

#[test]
fn BRepGraph_ViewsTest__ShapesView_Original_ValidNode_ReturnsShape() {
    let graph = views_box();
    assert!(!graph.shapes().original(FaceId::new(0).into()).is_null());
}

#[test]
fn BRepGraph_ViewsTest__ShapesView_Original_InvalidNode_ReturnsNull() {
    let graph = views_box();
    assert!(graph.shapes().original(NodeId::default()).is_null());
}

#[test]
fn BRepGraph_ViewsTest__ShapesView_Reconstruct_InvalidNode_ReturnsNull() {
    let graph = views_box();
    assert!(
        graph
            .shapes()
            .reconstruct(NodeId::default())
            .unwrap()
            .is_null()
    );
}
#[test]
fn BRepGraph_ViewsTest__ShapesView_RemovedNode_OriginalQueries_AreUnavailable() {
    let mut graph = views_box();
    let id = FaceId::new(0).into();
    assert!(graph.shapes().has_original(id));
    views_remove(&mut graph, id);
    assert!(!graph.shapes().has_original(id));
    assert!(graph.shapes().original(id).is_null());
    assert!(graph.shapes().shape(id).unwrap().is_null());
}
#[test]
fn BRepGraph_ViewsTest__ShapesView_Reconstruct_RemovedNode_ReturnsNull() {
    let mut graph = views_box();
    let id = FaceId::new(0).into();
    views_remove(&mut graph, id);
    assert!(graph.shapes().reconstruct(id).unwrap().is_null());
}
#[test]
fn BRepGraph_ViewsTest__ShapesView_FindNodeAndHasNode_RemovedNode_AreUnavailable() {
    let mut graph = views_box();
    let id = FaceId::new(0).into();
    let shape = graph.shapes().original(id);
    assert!(!shape.is_null());
    assert!(graph.shapes().has_node(&shape));
    assert_eq!(graph.shapes().find_node(&shape), id);
    views_remove(&mut graph, id);
    assert!(!graph.shapes().has_node(&shape));
    assert!(!graph.shapes().find_node(&shape).is_valid());
}
fn views_square_wire(builder: &BRepBuilder) -> Wire {
    let points = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
    let vertices: Vec<Vertex> = points
        .into_iter()
        .map(|(x, y)| {
            let mut vertex = Vertex::default();
            builder
                .make_vertex_at(&mut vertex, Point3::new(x, y, 0.0), 1.0e-7)
                .unwrap();
            vertex
        })
        .collect();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for index in 0..4 {
        let edge = views_topological_edge(builder, &vertices[index], &vertices[(index + 1) % 4]);
        builder.add(&mut wire, &edge).unwrap();
    }
    wire.set_closed(true).unwrap();
    wire
}
fn views_topological_edge(builder: &BRepBuilder, first: &Vertex, last: &Vertex) -> Edge {
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    builder
        .add(&mut edge, &first.oriented(Orientation::Forward))
        .unwrap();
    builder
        .add(&mut edge, &last.oriented(Orientation::Reversed))
        .unwrap();
    edge
}
fn views_topological_face(builder: &BRepBuilder, wire: &Wire, orientation: Orientation) -> Face {
    let mut face = Face::default();
    builder.make_face(&mut face);
    builder.add(&mut face, wire).unwrap();
    face.set_orientation(orientation);
    face
}
#[test]
fn BRepGraph_ViewsTest__ShapesView_ShellAddFace_Internal_RoutedToSupplement() {
    let _fixture = views_box();
    let mut graph = BRepGraph::new();
    let builder = BRepBuilder::default();
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let wire = views_square_wire(&builder);
    let face = views_topological_face(&builder, &wire, Orientation::Forward);
    builder.add(&mut shell, &face).unwrap();
    let root = views_add_topology(&mut graph, &shell);
    let internal = views_topological_face(&builder, &wire, Orientation::Internal);
    let added = graph.shapes_mut().add_under(&internal, root).unwrap();
    assert!(added.is_ok());
    let layer = graph.layer_registry().find::<LayerTopoSupplement>();
    assert!(!layer.is_null());
    assert!(!layer.get().unwrap().attached_to(root).is_empty());
}
fn views_add_mixed_faces(graph: &mut BRepGraph, root: NodeId, builder: &BRepBuilder, wire: &Wire) {
    let forward = views_topological_face(builder, wire, Orientation::Forward);
    let forward_result = graph.shapes_mut().add_under(&forward, root).unwrap();
    assert!(forward_result.is_ok());
    assert!(forward_result.inserted_ref.is_valid());
    let internal = views_topological_face(builder, wire, Orientation::Internal);
    let internal_result = graph.shapes_mut().add_under(&internal, root).unwrap();
    assert!(internal_result.is_ok());
    assert!(!internal_result.inserted_ref.is_valid());
}
#[test]
fn BRepGraph_ViewsTest__ShapesView_CompoundAddChild_MixedOrientations_CoreRefAndSupplement() {
    let _fixture = views_box();
    let mut graph = BRepGraph::new();
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let root = views_add_topology(&mut graph, &compound);
    let wire = views_square_wire(&builder);
    views_add_mixed_faces(&mut graph, root, &builder, &wire);
    let layer = graph.layer_registry().find::<LayerTopoSupplement>();
    assert!(!layer.is_null());
    assert_eq!(layer.get().unwrap().attached_to(root).len(), 1);
}

fn views_add_topology(graph: &mut BRepGraph, shape: &Shape) -> NodeId {
    let options = shapes_view::Options {
        create_auto_product: false,
        ..Default::default()
    };
    let result = graph
        .shapes_mut()
        .add_with_options(shape, &options)
        .unwrap();
    assert!(result.is_ok());
    result.topology_root
}
