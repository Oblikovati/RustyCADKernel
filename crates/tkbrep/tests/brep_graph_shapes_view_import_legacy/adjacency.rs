#[test]
fn BRepGraph_ShapesViewImportTest__FacesOfEdge_BoxSharedEdge() {
    let graph = import_box_graph();
    // Every edge in a box is shared by exactly 2 faces.
    for index in 0..graph.topo().edges().nb() {
        let count = graph.topo().edges().faces_of(EdgeId::new(index)).count();
        assert_eq!(count, 2, "Edge {index} has {count} faces");
    }
}

#[test]
fn BRepGraph_ShapesViewImportTest__SharedEdges_AdjacentBoxFaces() {
    let graph = import_box_graph();
    assert_eq!(graph.topo().faces().nb(), 6);
    let pairs = (0..graph.topo().faces().nb())
        .flat_map(|a| ((a + 1)..graph.topo().faces().nb()).map(move |b| (a, b)));
    let sharing = pairs
        .filter(|&(a, b)| import_shared_edges(&graph, FaceId::new(a), FaceId::new(b)) > 0)
        .count();
    // A box has 12 edges, each shared by 2 faces, so 12 sharing pairs.
    assert_eq!(sharing, 12);
}

#[test]
fn BRepGraph_ShapesViewImportTest__AdjacentFaces_BoxFace() {
    let graph = import_box_graph();
    assert_eq!(graph.topo().faces().nb(), 6);
    for index in 0..graph.topo().faces().nb() {
        let count = import_adjacent_faces(&graph, FaceId::new(index));
        assert_eq!(count, 4, "Face {index} has {count} adjacent faces");
    }
}

#[test]
fn BRepGraph_ShapesViewImportTest__FacesOfEdge_NoFaces_Programmatic() {
    let mut graph = BRepGraph::new();
    let (_, _, edge) = import_single_edge(&mut graph);
    assert_eq!(graph.topo().edges().faces_of(edge).count(), 0);
}

#[test]
fn BRepGraph_ShapesViewImportTest__EdgesOfFace_Box_HasEdges() {
    let graph = import_box_graph();
    let explorer = ChildExplorer::with_target(&graph, FaceId::start(), NodeKind::EDGE).unwrap();
    let count = explorer.count();
    assert_eq!(count, 4);
}

#[test]
fn BRepGraph_ShapesViewImportTest__VerticesOfEdge_Box_HasTwoVertices() {
    let graph = import_box_graph();
    let explorer = ChildExplorer::with_target(&graph, EdgeId::start(), NodeKind::VERTEX).unwrap();
    let count = explorer.count();
    assert_eq!(count, 2);
}

#[test]
fn BRepGraph_ShapesViewImportTest__EdgesOfVertex_Box_ThreeEdges() {
    let graph = import_box_graph();
    let edges = graph.topo().vertices().edges(VertexId::start());
    assert_eq!(edges.len(), 3);
}

#[test]
fn BRepGraph_ShapesViewImportTest__AdjacentEdges_Box_SharedVertex() {
    let graph = import_box_graph();
    let count = import_adjacent_edges(&graph, EdgeId::start());
    assert!(count >= 4);
}

#[test]
fn BRepGraph_ShapesViewImportTest__NbFacesOfEdge_Box_TwoFaces() {
    let graph = import_box_graph();
    let count = graph.topo().edges().nb_faces(EdgeId::start());
    assert_eq!(count, 2);
}

#[test]
fn BRepGraph_ShapesViewImportTest__IsManifoldEdge_Box_True() {
    let graph = import_box_graph();
    assert!(tool::edge::is_manifold(&graph, EdgeId::start()));
    assert!(!tool::edge::is_boundary(&graph, EdgeId::start()));
}

#[test]
fn BRepGraph_ShapesViewImportTest__InvalidInput_ReturnsEmpty() {
    let graph = import_box_graph();
    assert_eq!(graph.topo().vertices().edges(VertexId::new(999)).len(), 0);
    assert_eq!(import_adjacent_edges(&graph, EdgeId::new(999)), 0);
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddWithHistory_PreservesRequestedAddedNodes() {
    let mut graph = BRepGraph::new();
    let layer = graph.layer_registry().ensure::<LayerHistory>().unwrap();
    layer.get().unwrap().set_enabled(false);
    let result = import_disabled_history_box(&mut graph);
    assert!(result.is_ok());
    assert!(result.added_nodes.extent() > 0);
    let mut from_roots = ShapeNodeMap::default();
    graph
        .shapes()
        .collect_history_inputs(&[result.topology_root], &mut from_roots)
        .unwrap();
    assert!(from_roots.extent() > 0);
    let layer = graph.layer_registry().ensure::<LayerHistory>().unwrap();
    assert_eq!(layer.get().unwrap().nb_records(), 0);
}

fn import_disabled_history_box(
    graph: &mut BRepGraph,
) -> rustycad_tkbrep::brep_graph::shapes_view::AddResult {
    let options = Options {
        track_added_nodes: true,
        ..Options::default()
    };
    let inputs = ShapeNodeMap::default();
    let box_shape = support::make_box(10., 20., 30.);
    graph
        .shapes_mut()
        .add_with_history_options(
            &box_shape,
            &inputs,
            &Handle::default(),
            &AsciiString::from("Test:DisabledHistory"),
            &options,
        )
        .unwrap()
}
