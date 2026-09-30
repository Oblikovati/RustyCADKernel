use rustycad_tkbrep::brep_graph::tool::edge as validation_edge;
#[test]
fn BRepGraph_ValidateTest__WireConnectivity_DisconnectedEdges() {
    let mut graph = validation_box();
    assert!(graph.topo().wires().nb() > 0);
    let edge = first_connected_wire_edge(&graph);
    assert!(NodeId::from(edge).is_valid());
    let mut guard = graph.editor().edges().mutate(edge).unwrap();
    let reference = guard.end_vertex_ref_id;
    let end = validation_reference_vertex(guard.graph(), reference);
    let start = validation_reference_vertex(guard.graph(), guard.start_vertex_ref_id);
    corrupt_wire_end(guard.graph_mut(), reference, start, end);
    assert_ne!(validation_reference_vertex(guard.graph(), reference), end);
    assert_connectivity_warning(guard.graph());
}
fn corrupt_wire_end(graph: &mut BRepGraph, reference: VertexRefId, start: VertexId, end: VertexId) {
    for i in 0..graph.topo().vertices().nb() {
        let vertex = VertexId::new(i);
        if vertex == start || vertex == end {
            continue;
        }
        graph
            .editor()
            .vertices()
            .mutate_ref(reference)
            .unwrap()
            .internal()
            .child_vertex_id = vertex;
        break;
    }
}
fn edge_for_validation_split(graph: &BRepGraph) -> EdgeId {
    for i in 0..graph.topo().edges().nb() {
        let id = EdgeId::new(i);
        let definition = graph.topo().edges().definition(id);
        if !validation_edge::degenerated(graph, id)
            && definition.curve3d_rep_id.is_valid()
            && validation_edge::start_vertex_id(graph, id).is_valid()
            && validation_edge::end_vertex_id(graph, id).is_valid()
        {
            return id;
        }
    }
    EdgeId::default()
}
#[test]
fn BRepGraph_ValidateTest__AfterSplitEdge_ProducesSubEdges() {
    let mut graph = validation_box();
    let before = graph.topo().edges().nb();
    let edge = edge_for_validation_split(&graph);
    assert!(edge.is_valid());
    let (vertex, parameter) = validation_split_vertex(&mut graph, edge);
    let (sub_a, sub_b) = graph
        .editor()
        .edges()
        .split(edge, vertex, parameter)
        .unwrap();
    assert_eq!(graph.topo().edges().nb(), before + 2);
    assert!(sub_a.is_valid());
    assert!(sub_b.is_valid());
    assert!(edge.is_removed(&graph));
    assert!(audit_graph(&graph).is_valid());
}

fn first_connected_wire_edge(graph: &BRepGraph) -> EdgeId {
    let wire = (0..graph.topo().wires().nb())
        .map(WireId::new)
        .find(|id| graph.topo().wires().relations(*id).co_edge_ids.len() >= 2)
        .unwrap();
    assert!(wire.is_valid());
    let coedges = &graph.topo().wires().relations(wire).co_edge_ids;
    assert!(!coedges.is_empty());
    graph.topo().co_edges().definition(coedges[0]).child_edge_id
}
fn assert_connectivity_warning(graph: &BRepGraph) {
    let result = audit_graph(graph);
    let text = "Wire edges not connected";
    assert!(!contains_validation_issue(&result, Severity::Error, text));
    assert!(contains_validation_issue(&result, Severity::Warning, text));
}
fn validation_split_vertex(graph: &mut BRepGraph, edge: EdgeId) -> (VertexId, f64) {
    let (first, last) = validation_edge::range(graph, edge);
    let parameter = 0.5 * (first + last);
    let point = validation_edge::curve(graph, edge)
        .get()
        .unwrap()
        .d0(parameter)
        .unwrap();
    let tolerance = validation_edge::tolerance(graph, edge);
    (graph.editor().vertices().add(point, tolerance), parameter)
}
