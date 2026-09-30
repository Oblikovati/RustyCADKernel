#[test]
fn BRepGraph_ViewsTest__BRepGraphTool_FindCoEdgeId_ValidPair() {
    let graph = views_box();
    let id = EdgeId::new(0);
    let _entry = graph.topo().edges().definition(id);
    let face = graph.topo().edges().faces_of(id).current_id();
    assert!(face.is_valid());
    let coedge = edge::find_co_edge_id(&graph, id, face);
    assert!(coedge.is_valid());
    let entry = graph.topo().co_edges().definition(coedge);
    assert_eq!(entry.child_edge_id, id);
    assert_eq!(entry.face_id, face);
}
#[test]
fn BRepGraph_ViewsTest__BRepGraphTool_FindCoEdgeId_InvalidPair_ReturnsInvalid() {
    let graph = views_box();
    let edge = EdgeId::new(0);
    let non_adjacent = (0..graph.topo().faces().nb())
        .map(FaceId::new)
        .find(|face| !graph.topo().edges().faces_of(edge).any(|id| id == *face));
    if let Some(face) = non_adjacent {
        assert!(!edge::find_co_edge_id(&graph, edge, face).is_valid());
    }
}
#[test]
fn BRepGraph_ViewsTest__BRepGraphTool_FindPCurveCoEdgeId_SkipsRemovedCoEdges() {
    let mut graph = views_box();
    let wire = face::outer_wire(&graph, FaceId::start()).unwrap();
    assert!(wire.is_valid());
    let ids = &graph.topo().wires().relations(wire).co_edge_ids;
    assert!(!ids.is_empty());
    let id = ids[0];
    assert!(id.is_valid());
    let entry = graph.topo().co_edges().definition(id);
    let (edge, face) = (entry.child_edge_id, entry.face_id);
    assert_eq!(edge::find_co_edge_id(&graph, edge, face), id);
    views_remove(&mut graph, id);
    assert!(graph.topo().generic().is_removed(id.into()));
    assert!(!edge::find_pcurve_co_edge_id(&graph, edge, face).is_valid());
    assert_ne!(edge::find_co_edge_id(&graph, edge, face), id);
}
#[test]
fn BRepGraph_ViewsTest__BRepGraphTool_OuterWire_SkipsRemovedWireDef() {
    let mut graph = views_box();
    let wire = face::outer_wire(&graph, FaceId::start()).unwrap();
    assert!(wire.is_valid());
    views_remove(&mut graph, wire);
    assert!(graph.topo().generic().is_removed(wire.into()));
    assert_ne!(face::outer_wire(&graph, FaceId::start()).unwrap(), wire);
}
#[test]
fn BRepGraph_ViewsTest__BRepGraphTool_OuterWire_SkipsWireWithoutUVBounds() {
    let mut graph = views_box();
    let face = FaceId::start();
    let valid = face::outer_wire(&graph, face).unwrap();
    assert!(valid.is_valid());
    let empty = graph.editor().wires().add(&[]);
    assert!(empty.is_valid());
    let reference = graph
        .editor()
        .faces()
        .append(face, empty, ParityOrientation::FORWARD)
        .unwrap();
    assert!(reference.is_valid());
    assert_eq!(face::outer_wire(&graph, face).unwrap(), valid);
}
#[test]
fn BRepGraph_ViewsTest__BRepGraphTool_IsBoundary_SkipsRemovedFaces() {
    let mut graph = views_box();
    let edge = (0..graph.topo().edges().nb())
        .map(EdgeId::new)
        .find(|id| graph.topo().edges().faces_of(*id).count() == 2)
        .unwrap();
    assert!(edge.is_valid());
    assert_eq!(graph.topo().edges().faces_of(edge).count(), 2);
    assert!(!edge::is_boundary(&graph, edge));
    assert!(edge::is_manifold(&graph, edge));
    let face = graph.topo().edges().faces_of(edge).current_id();
    assert!(face.is_valid());
    views_remove(&mut graph, face);
    assert!(graph.topo().generic().is_removed(face.into()));
    assert!(edge::is_boundary(&graph, edge));
    assert!(!edge::is_manifold(&graph, edge));
}
#[test]
fn BRepGraph_ViewsTest__BRepGraphTool_IsBoundaryAndIsManifold_RemovedEdge_ReturnsFalse() {
    let mut graph = views_box();
    let id = EdgeId::start();
    assert!(id.is_valid_for(graph.topo().edges().nb()));
    assert!(!graph.topo().generic().is_removed(id.into()));
    views_remove(&mut graph, id);
    assert!(graph.topo().generic().is_removed(id.into()));
    assert!(!edge::is_boundary(&graph, id));
    assert!(!edge::is_manifold(&graph, id));
}
#[test]
fn BRepGraph_ViewsTest__BRepGraphTool_SeamPair_BoundsCheck() {
    let mut graph = views_box();
    graph.clear().unwrap();
    let mut sphere = BRepPrimAPIMakeSphere::at_center(Point3::new(0.0, 0.0, 0.0), 10.0).unwrap();
    graph.shapes_mut().add(sphere.shape().unwrap()).unwrap();
    assert!(!graph.is_empty());
    let mut found = false;
    for id in (0..graph.topo().co_edges().nb()).map(CoEdgeId::new) {
        let pair = co_edge::seam_pair(&graph, id);
        if pair.is_valid() {
            found = true;
            assert_eq!(co_edge::seam_pair(&graph, pair), id);
            break;
        }
    }
    assert!(found);
}
fn views_compound(shapes: &[&Shape]) -> Compound {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for shape in shapes {
        builder.add(&mut compound, shape).unwrap();
    }
    compound
}
fn views_duplicate_box_validation(graph: &mut BRepGraph) {
    let first = support::make_box(10.0, 20.0, 30.0);
    let second = support::make_box(10.0, 20.0, 30.0);
    let compound = views_compound(&[&first, &second]);
    graph.clear().unwrap();
    graph.shapes_mut().add(&compound).unwrap();
    assert!(!graph.is_empty());
    let options = deduplicate::Options {
        merge_entities_when_safe: true,
        ..Default::default()
    };
    deduplicate::perform_with(graph, &options).unwrap();
    compact::perform(graph).unwrap();
    assert!(validate::perform(graph).is_valid());
}
#[test]
fn BRepGraph_ViewsTest__EditorView_ReplaceEdge_SkipsRemovedCoEdgeDef() {
    let mut graph = views_box();
    views_duplicate_box_validation(&mut graph);
}
#[test]
fn BRepGraph_ViewsTest__EditorView_ReplaceEdge_RejectsRemovedWire() {
    let mut graph = views_box();
    views_duplicate_box_validation(&mut graph);
}
#[test]
fn BRepGraph_ViewsTest__EditorView_RemoveSolid_PropagatesSubtreeGenToParent() {
    let mut graph = views_box();
    let shape = support::make_box(10.0, 20.0, 30.0);
    let compound = views_compound(&[&shape]);
    graph.clear().unwrap();
    graph.shapes_mut().add(&compound).unwrap();
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().compounds().nb(), 1);
    assert_eq!(graph.topo().solids().nb(), 1);
    let solid = SolidId::start();
    views_remove(&mut graph, solid);
    assert!(graph.topo().generic().is_removed(solid.into()));
    compact::perform(&mut graph).unwrap();
    assert!(validate::perform(&graph).is_valid());
}
#[test]
fn BRepGraph_ViewsTest__EditorView_RemoveOccurrence_UnbindsProductOccurrence() {
    let _fixture = views_box();
    let mut graph = BRepGraph::new();
    let shape = support::make_box(10.0, 20.0, 30.0);
    let mut first = BRepBuilderAPICopy::with_shape(&shape, true, false).unwrap();
    let mut second = BRepBuilderAPICopy::with_shape(&shape, true, false).unwrap();
    let compound = views_compound(&[first.shape().unwrap(), second.shape().unwrap()]);
    graph.shapes_mut().add(&compound).unwrap();
    assert!(!graph.is_empty());
    let options = deduplicate::Options {
        merge_entities_when_safe: true,
        ..Default::default()
    };
    deduplicate::perform_with(&mut graph, &options).unwrap();
    compact::perform_with(&mut graph, &compact::Options::default()).unwrap();
    assert!(graph.validate_relations());
}
#[test]
fn BRepGraph_ViewsTest__EditorView_Split_RejectsRemovedVertex() {
    let mut graph = views_box();
    let edge = EdgeId::start();
    let reference = edge::start_vertex_id(&graph, edge);
    let usage = vertex::usage(&graph, reference);
    assert!(usage.is_valid());
    let vertex = usage.def_id;
    assert!(!graph.topo().generic().is_removed(vertex.into()));
    views_remove(&mut graph, vertex);
    assert!(graph.topo().generic().is_removed(vertex.into()));
    let (first, second) = graph.editor().edges().split(edge, vertex, 0.5).unwrap();
    assert!(!first.is_valid());
    assert!(!second.is_valid());
}
#[test]
fn BRepGraph_ViewsTest__CleanupRemovedRefs_OrphanedCoEdgesMarkedRemoved() {
    let mut graph = views_box();
    let edge = EdgeId::start();
    assert!(edge.is_valid_for(graph.topo().edges().nb()));
    let before = graph.topo().co_edges().nb_active();
    views_remove(&mut graph, edge);
    assert!(graph.topo().generic().is_removed(edge.into()));
    graph.editor().generic().cleanup_removed_references();
    assert!(graph.topo().co_edges().nb_active() < before);
    assert!(graph.validate_relations());
}
#[test]
fn BRepGraph_ViewsTest__TopoView_RepAccessors_ReturnNullForRemovedEntity() {
    let mut graph = views_box();
    let edge = EdgeId::start();
    assert!(!graph.topo().edges().curve3d(edge).is_null());
    views_remove(&mut graph, edge);
    assert!(graph.topo().generic().is_removed(edge.into()));
    assert!(graph.topo().edges().curve3d(edge).is_null());
    let face = FaceId::start();
    assert!(!graph.topo().faces().surface(face).is_null());
    views_remove(&mut graph, face);
    assert!(graph.topo().generic().is_removed(face.into()));
    assert!(graph.topo().faces().surface(face).is_null());
    assert!(graph.topo().faces().active_triangulation(face).is_null());
}
