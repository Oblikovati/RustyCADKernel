fn views_box() -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph
        .shapes_mut()
        .add(&support::make_box(10.0, 20.0, 30.0))
        .unwrap();
    graph
}
fn views_remove(graph: &mut BRepGraph, node: impl Into<NodeId>) {
    graph.editor().generic().remove_node(node.into()).unwrap();
}
fn views_face_ref(graph: &BRepGraph) -> FaceRefId {
    let ids = graph.refs().faces().ids_of(ShellId::start());
    assert!(!ids.is_empty());
    ids[0]
}
fn views_adjacent_faces(graph: &BRepGraph, id: FaceId) -> usize {
    let mut iterator = RelatedIterator::new(graph, id.into()).unwrap();
    let mut count = 0;
    while iterator.more() {
        count += usize::from(iterator.current_relation() == RelationKind::AdjacentFace);
        iterator.next().unwrap();
    }
    count
}
fn views_adjacent_edges(graph: &BRepGraph, id: EdgeId) -> usize {
    if !id.is_valid_for(graph.topo().edges().nb()) || id.is_removed(graph) {
        return 0;
    }
    let mut adjacent = Vec::new();
    let mut vertices = DefsVertexOfEdge::new(graph, id);
    while vertices.more() {
        let vertex = vertices.current_id();
        vertices.next();
        for other in graph.topo().vertices().edges(vertex) {
            if *other != id && !other.is_removed(graph) && !adjacent.contains(other) {
                adjacent.push(*other);
            }
        }
    }
    adjacent.len()
}
fn views_active_refs<T: RefTag>(graph: &BRepGraph, ids: &[TypedId<T>]) -> usize {
    ids.iter()
        .filter(|id| !graph.refs().generic().is_removed((**id).into()))
        .count()
}

fn views_active_count(graph: &BRepGraph, kind: NodeKind, expected: u32) {
    let generic = graph.topo().generic();
    let active = (0..generic.nb(kind))
        .filter(|i| !generic.is_removed(NodeId::new(kind, *i)))
        .count();
    assert_eq!(active, expected as usize);
}
