#[test]
fn BRepGraph_ReconstructTest__Shape_UnmodifiedGraph_SameAsOriginal() {
    let graph = reconstruct_graph(&reconstruct_box());
    let id = SolidId::new(0).into();
    assert!(graph.shapes().has_original(id));
    let shape = graph.shapes().shape(id).unwrap();
    let original = graph.shapes().original(id);
    assert!(!original.is_null());
    assert!(shape.is_same(&original));
}
#[test]
fn BRepGraph_ReconstructTest__HasOriginal_BuildFace_ReturnsTrue() {
    let graph = reconstruct_graph(&reconstruct_box());
    assert!(graph.shapes().has_original(FaceId::start().into()));
}
#[test]
fn BRepGraph_ReconstructTest__Original_Face_IsSameAsBuildInputFace() {
    let shape = reconstruct_box();
    let explorer = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    assert!(explorer.more());
    let first_face = explorer.current().clone();
    let graph = reconstruct_graph(&shape);
    let original = graph.shapes().original(FaceId::start().into());
    assert!(!original.is_null());
    assert!(original.is_same(&first_face));
}
#[test]
fn BRepGraph_ReconstructTest__HasOriginal_ManualVertex_ReturnsFalse() {
    let mut graph = reconstruct_graph(&reconstruct_box());
    let id = graph
        .editor()
        .vertices()
        .add(Point3::new(42.0, 0.0, 0.0), 0.001);
    assert!(id.is_valid());
    assert!(!graph.shapes().has_original(id.into()));
}
#[test]
fn BRepGraph_ReconstructTest__FindNode_OriginalFace_RoundTrip() {
    let graph = reconstruct_graph(&reconstruct_box());
    let id = FaceId::start().into();
    let original = graph.shapes().original(id);
    assert!(!original.is_null());
    assert_eq!(graph.shapes().find_node(&original), id);
}
#[test]
fn BRepGraph_ReconstructTest__Reconstruct_Face_ValidShape() {
    let graph = reconstruct_graph(&reconstruct_box());
    assert!(graph.topo().faces().nb() > 0);
    let reconstructed = graph.shapes().reconstruct(FaceId::start().into()).unwrap();
    assert!(!reconstructed.is_null());
    assert_eq!(reconstructed.shape_type().unwrap(), ShapeEnum::Face);
}
#[test]
fn BRepGraph_ReconstructTest__Reconstruct_Edge_ValidShape() {
    let graph = reconstruct_graph(&reconstruct_box());
    assert!(graph.topo().edges().nb() > 0);
    for id in reconstruction_ids(&graph) {
        if edge::degenerated(&graph, id) {
            continue;
        }
        let reconstructed = graph.shapes().reconstruct(id.into()).unwrap();
        assert!(!reconstructed.is_null());
        assert_eq!(reconstructed.shape_type().unwrap(), ShapeEnum::Edge);
        return;
    }
    panic!("no non-degenerate edge found; expected at least one box edge");
}
#[test]
fn BRepGraph_ReconstructTest__Reconstruct_Vertex_CorrectPoint() {
    let graph = reconstruct_graph(&reconstruct_box());
    assert!(graph.topo().vertices().nb() > 0);
    let expected = vertex::pnt(&graph, VertexId::start());
    let reconstructed = graph
        .shapes()
        .reconstruct(NodeId::new(NodeKind::VERTEX, 0))
        .unwrap();
    assert!(!reconstructed.is_null());
    assert_eq!(reconstructed.shape_type().unwrap(), ShapeEnum::Vertex);
    let actual = BRepTool::pnt(&topods::vertex(&reconstructed).unwrap()).unwrap();
    assert!(actual.distance(&expected) <= precision::confusion());
}
