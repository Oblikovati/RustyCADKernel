fn scenario_paired_coedges(graph: &BRepGraph, check_edge: bool) -> usize {
    let mut count = 0;
    for id in (0..graph.topo().co_edges().nb()).map(CoEdgeId::new) {
        let pair = co_edge::seam_pair(graph, id);
        if !pair.is_valid() {
            continue;
        }
        count += 1;
        assert_eq!(co_edge::seam_pair(graph, pair), id);
        if check_edge {
            let defs = graph.topo().co_edges();
            assert_eq!(
                defs.definition(pair).child_edge_id,
                defs.definition(id).child_edge_id
            );
        }
    }
    count
}
#[test]
fn BRepGraph_ScenarioMatrix__Sphere_SeamCoEdgePair_Bidirectional() {
    let original = scenario_sphere();
    let populated = scenario_populated(&original);
    let count = scenario_paired_coedges(&populated, true);
    assert!(count > 0);
    let graph = scenario_graph(&original);
    scenario_valid(&graph);
    let reconstructed = scenario_reconstruct(&graph, SolidId::start());
    let round_trip = scenario_populated(&reconstructed);
    assert_eq!(scenario_paired_coedges(&round_trip, false), count);
}
fn scenario_split_seam(graph: &mut BRepGraph, check_inputs: bool) -> [EdgeId; 2] {
    let id = scenario_first_seam(graph);
    let range = edge::range(graph, id);
    if check_inputs {
        assert!(!edge::degenerated(graph, id));
        assert!(range.0 < range.1);
    }
    let point = Point3::new(5.0, 0.0, 7.5);
    let vertex = graph.editor().vertices().add(point, 1e-7);
    if check_inputs {
        assert!(vertex.is_valid());
    }
    let (first, second) = graph
        .editor()
        .edges()
        .split(id, vertex, 0.5 * (range.0 + range.1))
        .unwrap();
    assert!(first.is_valid());
    assert!(second.is_valid());
    [first, second]
}
fn scenario_edge_pair_symmetry(graph: &BRepGraph, id: EdgeId) {
    for coedge in graph.topo().edges().co_edges(id) {
        let pair = co_edge::seam_pair(graph, *coedge);
        if pair.is_valid() {
            assert_eq!(co_edge::seam_pair(graph, pair), *coedge);
        }
    }
}
#[test]
fn BRepGraph_ScenarioMatrix__Cylinder_SeamEdgeSplit_AuditStable() {
    let mut graph = scenario_graph(&scenario_cylinder(5.0, 15.0));
    let [first, second] = scenario_split_seam(&mut graph, true);
    scenario_valid(&graph);
    scenario_edge_pair_symmetry(&graph, first);
    scenario_edge_pair_symmetry(&graph, second);
}
fn scenario_coedge_incidence(graph: &BRepGraph, id: EdgeId) {
    let coedges = graph.topo().edges().co_edges(id);
    assert!(!coedges.is_empty());
    for coedge in coedges {
        let def = graph.topo().co_edges().definition(*coedge);
        assert!(def.curve2d_rep_id.is_valid());
        assert!(def.face_id.is_valid());
        assert_eq!(def.child_edge_id, id);
        let range = co_edge::range(graph, *coedge);
        assert!(range.0 < range.1);
    }
}
#[test]
fn BRepGraph_ScenarioMatrix__Cylinder_SeamEdgeSplit_CoEdgeFaceIncidence() {
    let mut graph = scenario_graph(&scenario_cylinder(5.0, 15.0));
    let [first, second] = scenario_split_seam(&mut graph, false);
    scenario_coedge_incidence(&graph, first);
    scenario_coedge_incidence(&graph, second);
    scenario_valid(&graph);
}
