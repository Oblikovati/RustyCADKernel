#[test]
fn BRepGraph_BuildTest__Box_FaceDefCount_MatchesTopExp() {
    let shape = build_box(10.0, 20.0, 30.0);
    let graph = build_graph(&shape);
    assert_build_counts(&graph, &shape, &[ShapeEnum::Face]);
    assert_eq!(graph.topo().faces().nb(), 6);
}
#[test]
fn BRepGraph_BuildTest__Box_EdgeDefCount_MatchesTopExp() {
    let shape = build_box(10.0, 20.0, 30.0);
    let graph = build_graph(&shape);
    assert_build_counts(&graph, &shape, &[ShapeEnum::Edge]);
    assert_eq!(graph.topo().edges().nb(), 12);
}
#[test]
fn BRepGraph_BuildTest__Box_VertexDefCount_MatchesTopExp() {
    let shape = build_box(10.0, 20.0, 30.0);
    let graph = build_graph(&shape);
    assert_build_counts(&graph, &shape, &[ShapeEnum::Vertex]);
    assert_eq!(graph.topo().vertices().nb(), 8);
}
#[test]
fn BRepGraph_BuildTest__Box_VertexPoints_MatchBRepTool() {
    let shape = build_box(10.0, 20.0, 30.0);
    let graph = build_graph(&shape);
    let map = build_map(&shape, ShapeEnum::Vertex);
    assert_eq!(graph.topo().vertices().nb() as usize, map.extent());
    let mut vertices = VertexIterator::new(&graph);
    while vertices.more() {
        let point = vertex::pnt(&graph, vertices.current_id());
        assert!((1..=map.extent()).any(|i| point.distance(
            &BRepTool::pnt(&topods::vertex(map.find_key(i).unwrap()).unwrap()).unwrap()
        ) < precision::confusion()));
        vertices.next();
    }
}
#[test]
fn BRepGraph_BuildTest__Box_FaceTolerances_MatchBRepTool() {
    let shape = build_box(10.0, 20.0, 30.0);
    let graph = build_graph(&shape);
    let map = build_map(&shape, ShapeEnum::Face);
    let mut definitions = FaceIterator::new(&graph);
    while definitions.more() {
        let tolerance = face::tolerance(&graph, definitions.current_id());
        assert!((1..=map.extent()).any(|i| {
            (tolerance
                - BRepTool::face_tolerance(&topods::face(map.find_key(i).unwrap()).unwrap())
                    .unwrap())
            .abs()
                < precision::confusion()
        }));
        definitions.next();
    }
}
#[test]
fn BRepGraph_BuildTest__Box_EdgeTolerances_MatchBRepTool() {
    let shape = build_box(10.0, 20.0, 30.0);
    let graph = build_graph(&shape);
    let map = build_map(&shape, ShapeEnum::Edge);
    let mut definitions = EdgeIterator::new(&graph);
    while definitions.more() {
        let tolerance = edge::tolerance(&graph, definitions.current_id());
        assert!((1..=map.extent()).any(|i| {
            (tolerance
                - BRepTool::edge_tolerance(&topods::edge(map.find_key(i).unwrap()).unwrap())
                    .unwrap())
            .abs()
                < precision::confusion()
        }));
        definitions.next();
    }
}
#[test]
fn BRepGraph_BuildTest__Box_AllSurfacesArePlanes() {
    let shape = build_box(10.0, 20.0, 30.0);
    let graph = build_graph(&shape);
    assert_eq!(graph.topo().faces().nb(), 6);
    assert_planar_surfaces(&graph);
}
#[test]
fn BRepGraph_BuildTest__Box_NoDegenerateEdges() {
    let shape = build_box(10.0, 20.0, 30.0);
    let graph = build_graph(&shape);
    assert_eq!(build_degenerate_count(&graph), 0);
}
#[test]
fn BRepGraph_BuildTest__Box_EdgeVertexDefsAreValid() {
    let shape = build_box(10.0, 20.0, 30.0);
    let graph = build_graph(&shape);
    let mut edges = EdgeIterator::new(&graph);
    while edges.more() {
        let first = edge::start_vertex_id(&graph, edges.current_id());
        let last = edge::end_vertex_id(&graph, edges.current_id());
        assert_vertex_reference(&graph, first);
        assert_vertex_reference(&graph, last);
        edges.next();
    }
}
#[test]
fn BRepGraph_BuildTest__Box_FaceSurfacesAreValid() {
    let shape = build_box(10.0, 20.0, 30.0);
    let graph = build_graph(&shape);
    let mut faces = FaceIterator::new(&graph);
    while faces.more() {
        assert!(face::has_surface(&graph, faces.current_id()));
        faces.next();
    }
}
#[test]
fn BRepGraph_BuildTest__Box_EdgeParamRange_IsNonDegenerate() {
    let shape = build_box(10.0, 20.0, 30.0);
    let graph = build_graph(&shape);
    let mut edges = EdgeIterator::new(&graph);
    while edges.more() {
        let (first, last) = edge::range(&graph, edges.current_id());
        assert!(first < last);
        edges.next();
    }
}

fn assert_vertex_reference(graph: &BRepGraph, reference: rustycad_tkbrep::brep_graph::VertexRefId) {
    assert!(reference.is_valid());
    let vertex = graph.refs().vertices().entry(reference).child_vertex_id;
    assert_eq!(NodeId::from(vertex).kind, NodeKind::VERTEX);
}
