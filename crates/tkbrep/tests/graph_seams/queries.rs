#[test]
fn BRepGraph_SeamRedesignTest__CylinderLateralWire_BothSeamHalvesInCoEdgeIds() {
    let graph = seam_graph(&seam_cylinder());
    let seam = first_seam_coedge(&graph);
    assert!(seam.is_valid());
    let pair = co_edge::seam_pair(&graph, seam);
    assert!(pair.is_valid());
    let wire = graph.topo().co_edges().wire(seam);
    assert!(wire.is_valid());
    let ids: Vec<CoEdgeId> = CoEdgesOfWire::new(&graph, wire).collect();
    assert!(ids.contains(&seam));
    assert!(ids.contains(&pair));
}
#[test]
fn BRepGraph_SeamRedesignTest__CylinderLateralWire_NbCoEdges_MatchesTopoDSIterator() {
    let cylinder = seam_cylinder();
    let graph = seam_graph(&cylinder);
    let lateral = seam_explore(&cylinder, ShapeEnum::Face)
        .into_iter()
        .find(|face| seam_explore(face, ShapeEnum::Edge).len() == 4)
        .unwrap();
    assert!(!lateral.is_null());
    let classical: usize = wire_child_counts(&lateral).iter().sum();
    let count: u32 = seam_wire_ids(&graph)
        .into_iter()
        .filter(|id| wire_has_original_face(&graph, *id, &lateral))
        .map(|id| wire::nb_co_edges(&graph, id))
        .sum();
    assert_eq!(count as usize, classical);
}
fn wire_has_original_face(graph: &BRepGraph, wire: WireId, expected: &Shape) -> bool {
    let face = wire::face_of(graph, wire);
    if !face.is_valid() {
        return false;
    }
    let original = graph.shapes().original(face.into());
    !original.is_null() && original.is_same(expected)
}
#[test]
fn BRepGraph_SeamRedesignTest__SeamPair_DerivedQuery_IsSymmetric() {
    let sphere = BRepPrimAPIMakeSphere::new(10.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let graph = seam_graph(&sphere);
    let mut count = 0;
    for i in 0..graph.topo().co_edges().nb() {
        let id = CoEdgeId::new(i);
        let pair = co_edge::seam_pair(&graph, id);
        if !pair.is_valid() {
            continue;
        }
        count += 1;
        assert_eq!(co_edge::seam_pair(&graph, pair), id);
    }
    assert!(count > 0);
}
#[test]
fn BRepGraph_SeamRedesignTest__SeamPair_BoxHasNoSeams() {
    let graph = seam_graph(&seam_box());
    assert_no_seam_coedges(&graph);
}
#[test]
fn BRepGraph_SeamRedesignTest__SeamPair_FreeWireHasNoSeams() {
    let graph = seam_graph(&seam_free_wire());
    assert_no_seam_coedges(&graph);
}
#[test]
fn BRepGraph_SeamRedesignTest__EdgeOps_IsSeamOnFace_DerivedFromConnectivity() {
    let graph = seam_graph(&seam_cylinder());
    let seam = first_seam_coedge(&graph);
    assert!(seam.is_valid());
    let definition = graph.topo().co_edges().definition(seam);
    assert!(edge::is_seam_on_face(
        &graph,
        definition.child_edge_id,
        definition.face_id
    ));
    let box_graph = seam_graph(&seam_box());
    for i in 0..box_graph.topo().co_edges().nb() {
        let definition = box_graph.topo().co_edges().definition(CoEdgeId::new(i));
        if !definition.face_id.is_valid() {
            continue;
        }
        assert!(!edge::is_seam_on_face(
            &box_graph,
            definition.child_edge_id,
            definition.face_id
        ));
    }
}
#[test]
fn BRepGraph_SeamRedesignTest__NbDistinctEdges_AccountsForSeamHalves() {
    let graph = seam_graph(&seam_cylinder());
    let mut seam_wires = 0;
    for id in seam_wire_ids(&graph) {
        let raw = wire::nb_co_edges(&graph, id);
        let distinct = wire::nb_distinct_edges(&graph, id);
        assert!(distinct <= raw);
        if raw > distinct {
            seam_wires += 1;
            assert!(raw - distinct >= 1);
        }
    }
    assert!(seam_wires >= 1);
}
