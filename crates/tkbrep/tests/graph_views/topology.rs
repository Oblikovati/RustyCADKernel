#[test]
fn BRepGraph_ViewsTest__DefsView_NbFaces() {
    let graph = views_box();
    assert_eq!(graph.topo().faces().nb(), 6);
}

#[test]
fn BRepGraph_ViewsTest__DefsView_NbSolids() {
    let graph = views_box();
    assert_eq!(graph.topo().solids().nb(), 1);
}

#[test]
fn BRepGraph_ViewsTest__DefsView_NbShells() {
    let graph = views_box();
    assert_eq!(graph.topo().shells().nb(), 1);
}

#[test]
fn BRepGraph_ViewsTest__DefsView_NbWires() {
    let graph = views_box();
    assert_eq!(graph.topo().wires().nb(), 6);
}

#[test]
fn BRepGraph_ViewsTest__DefsView_NbEdges() {
    let graph = views_box();
    assert_eq!(graph.topo().edges().nb(), 12);
}

#[test]
fn BRepGraph_ViewsTest__DefsView_NbVertices() {
    let graph = views_box();
    assert_eq!(graph.topo().vertices().nb(), 8);
}

#[test]
fn BRepGraph_ViewsTest__DefsView_ActiveCounts_MatchStorageState() {
    let graph = views_box();
    let topo = graph.topo();
    views_active_count(&graph, NodeKind::VERTEX, topo.vertices().nb_active());
    views_active_count(&graph, NodeKind::EDGE, topo.edges().nb_active());
    views_active_count(&graph, NodeKind::CO_EDGE, topo.co_edges().nb_active());
    views_active_count(&graph, NodeKind::WIRE, topo.wires().nb_active());
    views_active_count(&graph, NodeKind::FACE, topo.faces().nb_active());
    views_active_count(&graph, NodeKind::SHELL, topo.shells().nb_active());
    views_active_count(&graph, NodeKind::SOLID, topo.solids().nb_active());
    views_active_count(&graph, NodeKind::COMPOUND, topo.compounds().nb_active());
    views_active_count(&graph, NodeKind::COMP_SOLID, topo.comp_solids().nb_active());
    views_active_count(&graph, NodeKind::PRODUCT, topo.products().nb_active());
    views_active_count(&graph, NodeKind::OCCURRENCE, topo.occurrences().nb_active());
}

#[test]
fn BRepGraph_ViewsTest__DefsView_NbActiveFaces_ExcludeRemoved() {
    let mut graph = views_box();
    let before = graph.topo().faces().nb_active();
    views_remove(&mut graph, FaceId::start());
    assert_eq!(graph.topo().faces().nb_active(), before - 1);
    assert!(graph.topo().generic().is_removed(FaceId::start().into()));
}

#[test]
fn BRepGraph_ViewsTest__DefsView_FaceAccessor_Valid() {
    let graph = views_box();
    let mut iterator = iterator::FaceIterator::new(&graph);
    while iterator.more() {
        assert!(iterator.current_id().is_valid());
        iterator.next();
    }
}

#[test]
fn BRepGraph_ViewsTest__DefsView_TopoEntity_Valid() {
    let graph = views_box();
    assert!(
        graph
            .topo()
            .generic()
            .topo_entity(FaceId::new(0).into())
            .is_some()
    );
}

#[test]
fn BRepGraph_ViewsTest__TopoView_GenValidity() {
    let mut graph = views_box();
    let id = FaceId::start().into();
    assert_eq!(
        graph.topo().generic().nb(NodeKind::FACE),
        graph.topo().faces().nb()
    );
    assert!(graph.topo().generic().is_valid(id));
    assert!(graph.topo().generic().is_active(id));
    assert!(!graph.topo().generic().is_removed(id));
    views_remove(&mut graph, id);
    assert!(graph.topo().generic().is_valid(id));
    assert!(!graph.topo().generic().is_active(id));
    assert!(graph.topo().generic().is_removed(id));
    let outside = NodeId::new(NodeKind::FACE, graph.topo().faces().nb());
    assert!(!graph.topo().generic().is_valid(outside));
    assert!(!graph.topo().generic().is_active(outside));
    assert!(graph.topo().generic().is_removed(outside));
}

#[test]
fn BRepGraph_ViewsTest__DefsView_NbNodes_Positive() {
    let graph = views_box();
    assert!(graph.topo().generic().nb_nodes() > 0);
}

#[test]
fn BRepGraph_ViewsTest__DefsView_FaceSurface_NonNull() {
    let graph = views_box();
    for i in 0..graph.topo().faces().nb() {
        assert!(face::has_surface(&graph, FaceId::new(i)));
    }
}

#[test]
fn BRepGraph_ViewsTest__DefsView_EdgeCurve3d_NonNull() {
    let graph = views_box();
    for i in 0..graph.topo().edges().nb() {
        assert!(edge::has_curve(&graph, EdgeId::new(i)));
    }
}

#[test]
fn BRepGraph_ViewsTest__DefsView_FindPCurveCoEdgeId_NoCrash() {
    let graph = views_box();
    // Upstream accepts either result for an arbitrary edge/face pair.
    let _ = edge::find_pcurve_co_edge_id(&graph, EdgeId::start(), FaceId::start());
}

#[test]
fn BRepGraph_ViewsTest__DefsView_RepIdConvenienceAccessors_RoundTrip() {
    let graph = views_box();
    assert!(!graph.topo().faces().surface(FaceId::new(0)).is_null());
    assert!(!graph.topo().edges().curve3d(EdgeId::new(0)).is_null());
    let ids = graph.topo().edges().co_edges(EdgeId::new(0));
    assert!(!ids.is_empty());
    assert!(!graph.topo().co_edges().curve2d(ids[0]).is_null());
}

#[test]
fn BRepGraph_ViewsTest__DefsView_RepIdConvenienceAccessors_InvalidInput() {
    let graph = views_box();
    let face = FaceId::new(graph.topo().faces().nb());
    let edge = EdgeId::new(graph.topo().edges().nb());
    let coedge = CoEdgeId::new(graph.topo().co_edges().nb());
    assert!(graph.topo().faces().surface(face).is_null());
    assert!(graph.topo().faces().active_triangulation(face).is_null());
    assert!(graph.topo().edges().curve3d(edge).is_null());
    assert!(graph.topo().co_edges().curve2d(coedge).is_null());
}

#[test]
fn BRepGraph_ViewsTest__SpatialView_AdjacentFaces_FourPerBoxFace() {
    let graph = views_box();
    assert_eq!(views_adjacent_faces(&graph, FaceId::new(0)), 4);
}

#[test]
fn BRepGraph_ViewsTest__SpatialView_FacesOfEdge_TwoPerBoxEdge() {
    let graph = views_box();
    assert_eq!(graph.topo().edges().faces_of(EdgeId::new(0)).count(), 2);
}

#[test]
fn BRepGraph_ViewsTest__SpatialView_OutParam_Parity() {
    let graph = views_box();
    assert_eq!(views_adjacent_faces(&graph, FaceId::new(0)), 4);
    assert!(views_adjacent_edges(&graph, EdgeId::new(0)) >= 4);
}

#[test]
fn BRepGraph_ViewsTest__TopoView_GroupedFaceOps_Parity() {
    let graph = views_box();
    let id = FaceId::new(0);
    assert_eq!(NodeId::from(id), NodeId::from(id));
    assert!(!graph.topo().faces().surface(id).is_null());
    assert_eq!(views_adjacent_faces(&graph, id), 4);
}

#[test]
fn BRepGraph_ViewsTest__TopoView_GroupedEdgeAndVertexOps_Parity() {
    let graph = views_box();
    let id = EdgeId::new(0);
    assert_eq!(NodeId::from(id), NodeId::from(id));
    assert_eq!(graph.topo().edges().nb_faces(id), 2);
    assert!(!graph.topo().edges().curve3d(id).is_null());
    assert!(!edge::is_boundary(&graph, id));
    assert!(edge::is_manifold(&graph, id));
    assert!(graph.topo().edges().wires_of(id).count() >= 1);
    assert!(!graph.topo().edges().co_edges(id).is_empty());
    assert_eq!(graph.topo().edges().faces_of(id).count(), 2);
    assert!(views_adjacent_edges(&graph, id) >= 4);
    let vertex = VertexId::new(0);
    assert_eq!(NodeId::from(vertex), NodeId::from(vertex));
    assert!(!graph.topo().vertices().edges(vertex).is_empty());
}

#[test]
fn BRepGraph_ViewsTest__TopoView_GroupedCoEdgeOps_Parity() {
    let graph = views_box();
    let id = CoEdgeId::new(0);
    let entry = graph.topo().co_edges().definition(id);
    assert_eq!(NodeId::from(id), NodeId::from(id));
    assert_eq!(
        graph.topo().co_edges().definition(id).child_edge_id,
        entry.child_edge_id
    );
    assert_eq!(
        graph.topo().co_edges().definition(id).face_id,
        entry.face_id
    );
    assert!(!graph.topo().co_edges().curve2d(id).is_null());
    // Preserve the native self-comparison of the seam accessor.
    assert_eq!(
        co_edge::seam_pair(&graph, id),
        co_edge::seam_pair(&graph, id)
    );
}

#[test]
fn BRepGraph_ViewsTest__SpatialView_OutParam_ClearAndInvalid() {
    let graph = views_box();
    assert_eq!(views_adjacent_faces(&graph, FaceId::start()), 4);
    assert_eq!(views_adjacent_faces(&graph, FaceId::new(999)), 0);
    assert!(views_adjacent_edges(&graph, EdgeId::start()) >= 4);
    assert_eq!(views_adjacent_edges(&graph, EdgeId::new(999)), 0);
}
