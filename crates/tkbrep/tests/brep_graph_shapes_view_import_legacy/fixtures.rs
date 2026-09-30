fn import_box_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph
        .shapes_mut()
        .add(&support::make_box(10.0, 20.0, 30.0))
        .unwrap();
    graph
}

fn import_line(origin: Point3, direction: Direction3) -> Handle<dyn GeomCurve> {
    let axis = Axis1::new(origin, direction);
    let line = GeomLine::new(axis);
    Handle::new(line).into_cast()
}

fn import_plane() -> Handle<dyn GeomSurface> {
    let plane = Plane3::default();
    let surface = GeomPlane::from_pln(plane);
    Handle::new(surface).into_cast()
}

fn import_rectangle_edges(graph: &mut BRepGraph) -> ([VertexId; 4], [EdgeId; 4]) {
    let points = import_rectangle_points();
    let directions = import_rectangle_directions();
    let vertices = points.map(|point| graph.editor().vertices().add(point, 0.001));
    let edges = std::array::from_fn(|index| {
        let line = import_line(points[index], directions[index]);
        graph.editor().edges().add(
            vertices[index],
            vertices[(index + 1) % 4],
            &line,
            0.,
            10.,
            0.001,
        )
    });
    (vertices, edges)
}

fn import_rectangle_wire(graph: &mut BRepGraph, edges: &[EdgeId; 4]) -> WireId {
    let coedges = edges.map(|edge| graph.editor().co_edges().add(edge, FORWARD));
    graph.editor().wires().add(&coedges)
}

fn import_rectangle_face(graph: &mut BRepGraph) -> (FaceId, WireId, [EdgeId; 4], [VertexId; 4]) {
    let (vertices, edges) = import_rectangle_edges(graph);
    let wire = import_rectangle_wire(graph, &edges);
    let face = graph
        .editor()
        .faces()
        .add(&import_plane(), wire, &[], 0.001);
    (face, wire, edges, vertices)
}

fn import_program_error<T>(result: Result<T, Failure>) {
    let failure = result
        .err()
        .expect("expected Standard_ProgramError, got success");
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
}

fn import_shared_edges(graph: &BRepGraph, first: FaceId, second: FaceId) -> u32 {
    let mut related = RelatedIterator::new(graph, first.into()).unwrap();
    let mut count = 0;
    while related.more() {
        let edge = EdgeId::from_node_id(related.current());
        if related.current_relation() == RelationKind::BoundaryEdge
            && graph
                .topo()
                .edges()
                .faces_of(edge)
                .any(|face| face == second)
        {
            count += 1;
        }
        related.next().unwrap();
    }
    count
}

fn import_adjacent_faces(graph: &BRepGraph, face: FaceId) -> u32 {
    let mut related = RelatedIterator::new(graph, face.into()).unwrap();
    let mut count = 0;
    while related.more() {
        count += u32::from(related.current_relation() == RelationKind::AdjacentFace);
        related.next().unwrap();
    }
    count
}

fn import_adjacent_edges(graph: &BRepGraph, edge: EdgeId) -> usize {
    if !edge.is_valid_for(graph.topo().edges().nb()) || edge.is_removed(graph) {
        return 0;
    }
    let mut adjacent = Vec::<EdgeId>::new();
    let mut vertices = DefsVertexOfEdge::new(graph, edge);
    while vertices.more() {
        import_collect_adjacent(graph, edge, vertices.current_id(), &mut adjacent);
        vertices.next();
    }
    adjacent.len()
}

fn import_collect_adjacent(
    graph: &BRepGraph,
    edge: EdgeId,
    vertex: VertexId,
    adjacent: &mut Vec<EdgeId>,
) {
    for &candidate in graph.topo().vertices().edges(vertex) {
        if candidate != edge && !candidate.is_removed(graph) && !adjacent.contains(&candidate) {
            adjacent.push(candidate);
        }
    }
}

fn import_rectangle_directions() -> [Direction3; 4] {
    [
        Direction3::X,
        Direction3::Y,
        Direction3::new(-1., 0., 0.).unwrap(),
        Direction3::new(0., -1., 0.).unwrap(),
    ]
}

fn import_rectangle_points() -> [Point3; 4] {
    [
        Point3::new(0., 0., 0.),
        Point3::new(10., 0., 0.),
        Point3::new(10., 10., 0.),
        Point3::new(0., 10., 0.),
    ]
}
