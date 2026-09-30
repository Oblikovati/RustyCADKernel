#[test]
fn BRepGraph_GeometryTest__Box_FindPCurveCoEdgeId_AllEdgeFacePairs_Valid() {
    let graph = geometry_box(false);
    let mut count = 0;
    for edge_id in geometry_ids::<EdgeTag>(&graph) {
        for id in graph.topo().edges().co_edges(edge_id) {
            let def = graph.topo().co_edges().definition(*id);
            assert!(edge::find_pcurve_co_edge_id(&graph, edge_id, def.face_id).is_valid());
            count += 1;
        }
    }
    assert!(count > 0);
}
#[test]
fn BRepGraph_GeometryTest__CoEdge_FaceIdValid() {
    let graph = geometry_box(false);
    for edge_id in geometry_ids::<EdgeTag>(&graph) {
        for id in graph.topo().edges().co_edges(edge_id) {
            let face_id = graph.topo().co_edges().definition(*id).face_id;
            assert!(face_id.is_valid());
            assert_eq!(NodeId::from(face_id).kind, NodeKind::FACE);
        }
    }
}
#[test]
fn BRepGraph_GeometryTest__CoEdge_ParamRange_NonZero() {
    let graph = geometry_box(false);
    let mut count = 0;
    for edge_id in geometry_ids::<EdgeTag>(&graph) {
        for id in graph.topo().edges().co_edges(edge_id) {
            let (first, last) = co_edge::range(&graph, *id);
            assert!((last - first).abs() > precision::p_confusion());
            count += 1;
        }
    }
    assert!(count > 0);
}
#[test]
fn BRepGraph_GeometryTest__SetPCurveTwoArgPreservesExistingRange() {
    let mut graph = geometry_box(false);
    let (_, id) = geometry_first_coedge(&graph);
    assert!(id.is_valid());
    let before = co_edge::range(&graph, id);
    let replacement: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(100.0, 200.0),
        Direction2::X,
    ))
    .into_cast();
    graph
        .editor()
        .co_edges()
        .set_pcurve_keep_range(id, &replacement)
        .unwrap();
    let after = co_edge::range(&graph, id);
    geometry_param_near(after.0, before.0);
    geometry_param_near(after.1, before.1);
    assert!(co_edge::pcurve(&graph, id) == replacement);
}
#[test]
fn BRepGraph_GeometryTest__AllCoEdgesHaveCurve2d() {
    let graph = geometry_box(true);
    let mut count = 0;
    for edge_id in geometry_ids::<EdgeTag>(&graph) {
        for id in graph.topo().edges().co_edges(edge_id) {
            assert!(co_edge::has_pcurve(&graph, *id));
            count += 1;
        }
    }
    assert!(count > 0);
}
#[test]
fn BRepGraph_GeometryTest__CoEdgePCurveAdaptor_FallsBackOnPlaneWhenStoredPCurveRemoved() {
    let mut graph = geometry_box(true);
    let (edge_id, id) = geometry_first_coedge(&graph);
    assert!(id.is_valid());
    assert!(!co_edge::pcurve(&graph, id).is_null());
    graph
        .editor()
        .co_edges()
        .set_pcurve_keep_range(id, &Handle::default())
        .unwrap();
    assert!(co_edge::pcurve(&graph, id).is_null());
    let adaptor = co_edge::pcurve_adaptor(&graph, id).unwrap();
    assert!(adaptor.is_initialized());
    assert!(!co_edge::has_pcurve(&graph, id));
    let range = edge::range(&graph, edge_id);
    geometry_param_near(adaptor.first_parameter().unwrap(), range.0);
    geometry_param_near(adaptor.last_parameter().unwrap(), range.1);
}
#[test]
fn BRepGraph_GeometryTest__Box_EdgeDef_SameParameter_IsSet() {
    let graph = geometry_box(false);
    assert!(graph.topo().edges().nb() > 0);
    for id in geometry_ids::<EdgeTag>(&graph) {
        assert!(geometry_same_parameter(&graph, id));
    }
}
#[test]
fn BRepGraph_GeometryTest__Box_EdgeDef_SameRange_IsSet() {
    let graph = geometry_box(false);
    assert!(graph.topo().edges().nb() > 0);
    for id in geometry_ids::<EdgeTag>(&graph) {
        assert!(geometry_same_range(&graph, id));
    }
}
fn geometry_first_curved_pair(graph: &BRepGraph) -> (EdgeId, CoEdgeId) {
    for id in geometry_ids::<CoEdgeTag>(graph) {
        let edge_id = graph.topo().co_edges().definition(id).child_edge_id;
        if edge_id.is_valid_for(graph.topo().edges().nb())
            && !edge_id.is_removed(graph)
            && !edge::curve(graph, edge_id).is_null()
            && !co_edge::pcurve(graph, id).is_null()
        {
            return (edge_id, id);
        }
    }
    (EdgeId::default(), CoEdgeId::default())
}
#[test]
fn BRepGraph_GeometryTest__DerivedStateCache_LazyAndFreshAfterPCurveRangeMutation() {
    let mut graph = geometry_box(false);
    assert!(graph.cache_registry().find::<CacheDerivedState>().is_null());
    let (edge_id, id) = geometry_first_curved_pair(&graph);
    assert!(edge_id.is_valid_for(graph.topo().edges().nb()));
    assert!(id.is_valid_for(graph.topo().co_edges().nb()));
    assert!(geometry_same_range(&graph, edge_id));
    assert!(!graph.cache_registry().find::<CacheDerivedState>().is_null());
    let range = edge::range(&graph, edge_id);
    graph
        .editor()
        .co_edges()
        .set_param_range(id, range.0 + 1.0, range.1)
        .unwrap();
    assert!(!geometry_same_range(&graph, edge_id));
    assert!(!geometry_same_parameter(&graph, edge_id));
}
fn geometry_seam_coedge(graph: &BRepGraph) -> Option<(EdgeId, CoEdgeId)> {
    for edge_id in geometry_ids::<EdgeTag>(graph) {
        for id in graph.topo().edges().co_edges(edge_id) {
            if co_edge::is_seam(graph, *id) {
                return Some((edge_id, *id));
            }
        }
    }
    None
}
#[test]
fn BRepGraph_GeometryTest__Cylinder_SeamEdge_HasTwoCoEdges() {
    let graph = geometry_cylinder(20.0);
    let (_, id) = geometry_seam_coedge(&graph).expect("cylinder: expected seam edge");
    let pair = co_edge::seam_pair(&graph, id);
    let first = graph.topo().co_edges().definition(id);
    let second = graph.topo().co_edges().definition(pair);
    assert_ne!(first.orientation, second.orientation);
    assert_eq!(first.face_id, second.face_id);
}
fn geometry_oriented_seam(graph: &BRepGraph) {
    // The original skips if no seam is found; this fixture has a seam in the oracle.
    let Some((edge_id, id)) = geometry_seam_coedge(graph) else {
        return;
    };
    let face_id = graph.topo().co_edges().definition(id).face_id;
    let forward =
        edge::find_pcurve_co_edge_id_oriented(graph, edge_id, face_id, Orientation::Forward);
    let reversed =
        edge::find_pcurve_co_edge_id_oriented(graph, edge_id, face_id, Orientation::Reversed);
    assert!(forward.is_valid());
    assert!(reversed.is_valid());
    assert_ne!(forward, reversed);
}
#[test]
fn BRepGraph_GeometryTest__Cylinder_SeamEdge_FindPCurveCoEdgeId_WithOrientation() {
    let graph = geometry_cylinder(20.0);
    geometry_oriented_seam(&graph);
}
#[test]
fn BRepGraph_GeometryTest__Cylinder_SeamEdge_FindPCurveCoEdgeId_DistinguishesOrientation() {
    let graph = geometry_cylinder(20.0);
    geometry_oriented_seam(&graph);
}
#[test]
fn BRepGraph_GeometryTest__Box_FindPCurveCoEdgeId_MatchesCoEdge() {
    let graph = geometry_box(false);
    for edge_id in geometry_ids::<EdgeTag>(&graph) {
        for id in graph.topo().edges().co_edges(edge_id) {
            let def = graph.topo().co_edges().definition(*id);
            assert_eq!(
                edge::find_pcurve_co_edge_id_oriented(
                    &graph,
                    edge_id,
                    def.face_id,
                    def.orientation.into()
                ),
                *id
            );
        }
    }
}
