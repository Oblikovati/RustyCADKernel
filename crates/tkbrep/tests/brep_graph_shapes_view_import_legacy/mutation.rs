#[test]
fn BRepGraph_ShapesViewImportTest__RemoveVertex_IsRemoved() {
    let mut graph = BRepGraph::new();
    let vertex = graph
        .editor()
        .vertices()
        .add(Point3::new(1., 2., 3.), 0.001);
    assert!(!graph.topo().generic().is_removed(vertex.into()));
    graph.editor().generic().remove_node(vertex.into()).unwrap();
    assert!(graph.topo().generic().is_removed(vertex.into()));
}

#[test]
fn BRepGraph_ShapesViewImportTest__RemoveFaceFromBox() {
    let mut graph = import_box_graph();
    assert_eq!(graph.topo().faces().nb(), 6);
    let face = FaceId::new(0);
    assert!(!graph.topo().generic().is_removed(face.into()));
    graph.editor().generic().remove_node(face.into()).unwrap();
    assert!(graph.topo().generic().is_removed(face.into()));
    for index in 1..graph.topo().faces().nb() {
        assert!(!graph.topo().generic().is_removed(FaceId::new(index).into()));
    }
}

#[test]
fn BRepGraph_ShapesViewImportTest__RemoveInvalidNode_NoError() {
    let mut graph = BRepGraph::new();
    let invalid = NodeId::default();
    assert!(!graph.topo().generic().is_valid(invalid));
    assert!(!graph.topo().generic().is_active(invalid));
    assert!(graph.topo().generic().is_removed(invalid));
    graph.editor().generic().remove_node(invalid).unwrap();
}

#[test]
fn BRepGraph_ShapesViewImportTest__RemoveAlreadyRemovedNode_NoError() {
    let mut graph = BRepGraph::new();
    let vertex = graph
        .editor()
        .vertices()
        .add(Point3::new(1., 2., 3.), 0.001);
    assert!(vertex.is_valid());
    graph.editor().generic().remove_node(vertex.into()).unwrap();
    assert!(graph.topo().generic().is_removed(vertex.into()));
    graph.editor().generic().remove_node(vertex.into()).unwrap();
    assert!(graph.topo().generic().is_removed(vertex.into()));
}

#[test]
fn BRepGraph_ShapesViewImportTest__MutableFaceDefinition_ChangesTolerance() {
    let mut graph = import_box_graph();
    assert!(graph.topo().faces().nb() > 0);
    let _original_tolerance = tool::face::tolerance(&graph, FaceId::start());
    let mut guard = graph.editor().faces().mutate(FaceId::start()).unwrap();
    editor::FaceOps::set_tolerance_guarded(&mut guard, 0.5);
    drop(guard);
    assert!((tool::face::tolerance(&graph, FaceId::start()) - 0.5).abs() <= 1e-10);
    assert!(
        graph
            .topo()
            .faces()
            .definition(FaceId::start())
            .base
            .own_gen
            > 0
    );
}

#[test]
fn BRepGraph_ShapesViewImportTest__MutableShellDefinition() {
    let mut graph = import_box_graph();
    assert!(graph.topo().shells().nb() > 0);
    let mut guard = graph.editor().shells().mutate(ShellId::start()).unwrap();
    guard.mark_dirty();
    drop(guard);
    assert!(
        graph
            .topo()
            .shells()
            .definition(ShellId::start())
            .base
            .own_gen
            > 0
    );
}

#[test]
fn BRepGraph_ShapesViewImportTest__MutableSolidDefinition() {
    let mut graph = import_box_graph();
    assert!(graph.topo().solids().nb() > 0);
    let mut guard = graph.editor().solids().mutate(SolidId::start()).unwrap();
    guard.mark_dirty();
    drop(guard);
    assert!(
        graph
            .topo()
            .solids()
            .definition(SolidId::start())
            .base
            .own_gen
            > 0
    );
}

#[test]
fn BRepGraph_ShapesViewImportTest__MutableCompoundDefinition() {
    let mut graph = BRepGraph::new();
    graph.editor().compounds().add(&[]);
    assert_eq!(graph.topo().compounds().nb(), 1);
    let mut guard = graph
        .editor()
        .compounds()
        .mutate(CompoundId::start())
        .unwrap();
    guard.mark_dirty();
    drop(guard);
    let generation = graph
        .topo()
        .compounds()
        .definition(CompoundId::start())
        .base
        .own_gen;
    assert!(generation > 0);
}

#[test]
fn BRepGraph_ShapesViewImportTest__MutableCompSolidDefinition() {
    let mut graph = BRepGraph::new();
    graph.editor().comp_solids().add(&[]);
    assert_eq!(graph.topo().comp_solids().nb(), 1);
    let mut guard = graph
        .editor()
        .comp_solids()
        .mutate(CompSolidId::start())
        .unwrap();
    guard.mark_dirty();
    drop(guard);
    let generation = graph
        .topo()
        .comp_solids()
        .definition(CompSolidId::start())
        .base
        .own_gen;
    assert!(generation > 0);
}

#[test]
fn BRepGraph_ShapesViewImportTest__SkipsRemovedFaces() {
    let mut graph = import_box_graph();
    assert_eq!(graph.topo().faces().nb(), 6);
    let mut removal = graph.editor().generic();
    removal.remove_node(NodeId::new(NodeKind::FACE, 0)).unwrap();
    removal.remove_node(NodeId::new(NodeKind::FACE, 3)).unwrap();
    let mut iterator = FaceIterator::new(&graph);
    let mut count = 0;
    while iterator.more() {
        assert!(!iterator.current_id().is_removed(&graph));
        count += 1;
        iterator.next();
    }
    assert_eq!(count, 4);
}

#[test]
fn BRepGraph_ShapesViewImportTest__SkipsRemovedEdges() {
    let mut graph = import_box_graph();
    let edges = graph.topo().edges().nb();
    assert!(edges > 0);
    graph
        .editor()
        .generic()
        .remove_node(NodeId::new(NodeKind::EDGE, 0))
        .unwrap();
    let mut iterator = EdgeIterator::new(&graph);
    let mut count = 0;
    while iterator.more() {
        assert!(!iterator.current_id().is_removed(&graph));
        count += 1;
        iterator.next();
    }
    assert_eq!(count, edges - 1);
}

#[test]
fn BRepGraph_ShapesViewImportTest__SkipsFirstNode() {
    let mut graph = BRepGraph::new();
    for x in [0., 1., 2.] {
        graph.editor().vertices().add(Point3::new(x, 0., 0.), 0.001);
    }
    graph
        .editor()
        .generic()
        .remove_node(NodeId::new(NodeKind::VERTEX, 0))
        .unwrap();
    let mut iterator = VertexIterator::new(&graph);
    let mut count = 0;
    while iterator.more() {
        assert!(!iterator.current_id().is_removed(&graph));
        count += 1;
        iterator.next();
    }
    assert_eq!(count, 2);
}

#[test]
fn BRepGraph_ShapesViewImportTest__RemoveFace_RemovesWiresAndEdges() {
    let mut graph = BRepGraph::new();
    let (face, wire, edges, vertices) = import_rectangle_face(&mut graph);
    graph
        .editor()
        .generic()
        .remove_subgraph(face.into())
        .unwrap();
    assert!(graph.topo().generic().is_removed(face.into()));
    assert!(graph.topo().generic().is_removed(wire.into()));
    for edge in edges {
        assert!(graph.topo().generic().is_removed(edge.into()));
    }
    for vertex in vertices {
        assert!(graph.topo().generic().is_removed(vertex.into()));
    }
}

#[test]
fn BRepGraph_ShapesViewImportTest__RemoveSolid_CascadesToFaces() {
    let mut graph = import_box_graph();
    let solid = SolidId::new(0);
    graph
        .editor()
        .generic()
        .remove_subgraph(solid.into())
        .unwrap();
    assert!(graph.topo().generic().is_removed(solid.into()));
    for index in 0..graph.topo().shells().nb() {
        let shell = ShellId::new(index);
        assert!(graph.topo().generic().is_removed(shell.into()));
    }
    for index in 0..graph.topo().faces().nb() {
        assert!(graph.topo().generic().is_removed(FaceId::new(index).into()));
    }
}

#[test]
fn BRepGraph_ShapesViewImportTest__RemoveSubgraph_SharedFace_PreservesSharedEdgesAndVertices() {
    let mut graph = import_box_graph();
    let (faces, edges, vertices) = (
        graph.topo().faces().nb(),
        graph.topo().edges().nb(),
        graph.topo().vertices().nb(),
    );
    assert_eq!(faces, 6);
    assert_eq!(edges, 12);
    assert_eq!(vertices, 8);
    let removed = FaceId::new(0);
    graph
        .editor()
        .generic()
        .remove_subgraph(removed.into())
        .unwrap();
    assert!(graph.topo().generic().is_removed(removed.into()));
    import_assert_shared_survivors(&graph, faces, edges, vertices);
}

fn import_assert_shared_survivors(graph: &BRepGraph, faces: u32, edges: u32, vertices: u32) {
    // Shared edges retain another parent, and all vertices retain their edges.
    for index in 1..faces {
        assert!(!graph.topo().generic().is_removed(FaceId::new(index).into()));
    }
    for index in 0..edges {
        assert!(!graph.topo().generic().is_removed(EdgeId::new(index).into()));
    }
    for index in 0..vertices {
        assert!(
            !graph
                .topo()
                .generic()
                .is_removed(VertexId::new(index).into())
        );
    }
}
