fn inc_remove_cleanup(graph: &mut BRepGraph, id: impl Into<NodeId>) {
    graph.editor().generic().remove_node(id.into()).unwrap();
    graph.editor().generic().cleanup_removed_references();
}
#[test]
fn CleanupRemovedRefs_AfterRemoveFace_ValidatePasses() {
    let mut graph = inc_add(&inc_box());
    assert!(FaceId::start().is_valid_for(graph.topo().faces().nb()));
    inc_remove_cleanup(&mut graph, FaceId::start());
    assert!(graph.validate_relations());
}
#[test]
fn CleanupRemovedRefs_AfterRemoveFace_RemovesCoEdgePCurves() {
    let mut graph = inc_add(&inc_box());
    assert!(FaceId::start().is_valid_for(graph.topo().faces().nb()));
    let pairs = inc_face_pcurves(&graph);
    assert!(!pairs.is_empty());
    let old = graph.topo().geometry().nb_active_co_edge_curves2d();
    inc_remove_cleanup(&mut graph, FaceId::start());
    for (id, rep) in &pairs {
        let definition = graph.topo().co_edges().definition(*id);
        assert!(!definition.curve2d_rep_id.is_valid());
        assert!(RepId::from(*rep).is_removed(&graph));
    }
    let active = graph.topo().geometry().nb_active_co_edge_curves2d();
    assert_eq!(active, old - pairs.len() as u32);
    assert!(graph.validate_relations());
}
#[test]
fn CleanupRemovedRefs_AfterRemoveWire_ValidatePasses() {
    let mut graph = inc_add(&inc_box());
    let outer = face::outer_wire(&graph, FaceId::start()).unwrap();
    assert!(outer.is_valid());
    inc_remove_cleanup(&mut graph, outer);
    let result = validate::perform(&graph);
    assert!(graph.validate_relations());
    assert!(result.is_valid(), "{:?}", result.issues);
}
#[test]
fn CleanupRemovedRefs_AfterRemoveVertex_ValidatePasses() {
    let mut graph = inc_add(&inc_box());
    let start = edge::start_vertex_id(&graph, EdgeId::start());
    assert!(start.is_valid());
    let vertex = graph.refs().vertices().entry(start).child_vertex_id;
    inc_remove_cleanup(&mut graph, vertex);
    assert!(graph.validate_relations());
    assert!(
        !graph
            .topo()
            .edges()
            .definition(EdgeId::start())
            .start_vertex_ref_id
            .is_valid()
    );
}
#[test]
fn CleanupRemovedRefs_AfterRemoveMultiple_ValidatePasses() {
    let mut graph = inc_add(&inc_box());
    graph
        .editor()
        .generic()
        .remove_node(FaceId::start().into())
        .unwrap();
    graph
        .editor()
        .generic()
        .remove_node(EdgeId::start().into())
        .unwrap();
    graph.editor().generic().cleanup_removed_references();
    assert!(validate::perform(&graph).is_valid());
}
#[test]
fn Storage_MarkRemoved_UnbindsTShapeAndOriginal() {
    let mut graph = inc_add(&inc_box());
    let id = FaceId::start();
    assert!(id.is_valid_for(graph.topo().faces().nb()));
    assert!(graph.shapes().has_original(id.into()));
    let original = graph.shapes().original(id.into());
    assert!(!original.is_null());
    assert!(graph.shapes().has_node(&original));
    graph.editor().generic().remove_node(id.into()).unwrap();
    assert!(!graph.shapes().has_node(&original));
    assert!(!graph.shapes().has_original(id.into()));
}
#[test]
fn Storage_MarkRemoved_EraseLast_EmptyStore() {
    let mut graph = inc_add(&inc_box());
    let old = graph.topo().geometry().nb_face_surfaces();
    assert_ne!(old, 0);
    graph
        .editor()
        .generic()
        .remove_node(FaceId::start().into())
        .unwrap();
    compact::perform(&mut graph).unwrap();
    let new = graph.topo().geometry().nb_face_surfaces();
    assert!(new < old);
    assert!(graph.validate_relations());
    graph
        .editor()
        .faces()
        .set_surface(FaceId::start(), &inc_plane())
        .unwrap();
    assert_eq!(graph.topo().geometry().nb_face_surfaces(), new);
}
#[test]
fn Storage_MarkRemoved_UnbindsTShapeToNodeId() {
    let mut graph = inc_add(&inc_box());
    let solid = graph.shapes().shape(SolidId::start().into()).unwrap();
    assert!(!solid.is_null());
    let old = graph.shapes().find_node(&solid);
    assert!(old.is_valid());
    assert_eq!(old.kind, NodeKind::SOLID);
    graph.editor().generic().remove_node(old).unwrap();
    compact::perform(&mut graph).unwrap();
    assert!(!graph.shapes().find_node(&solid).is_valid());
    assert!(graph.validate_relations());
}
#[test]
fn Relations_BuildDelta_ExistingEdgeNewCoedge_GetsNewFace() {
    let mut graph = inc_add(&inc_compound(&[inc_box(), inc_box()]));
    assert_eq!(graph.topo().solids().nb(), 2);
    let options = deduplicate::Options {
        merge_entities_when_safe: true,
        ..Default::default()
    };
    deduplicate::perform_with(&mut graph, &options).unwrap();
    compact::perform(&mut graph).unwrap();
    for i in 0..graph.topo().edges().nb() {
        let id = EdgeId::new(i);
        if id.is_removed(&graph) {
            continue;
        }
        assert_ne!(graph.topo().edges().nb_faces(id), 0);
    }
    assert!(graph.validate_relations());
}

fn inc_face_pcurves(graph: &BRepGraph) -> Vec<(CoEdgeId, CoEdgeCurve2DRepId)> {
    (0..graph.topo().co_edges().nb())
        .map(CoEdgeId::new)
        .filter_map(|id| {
            let def = graph.topo().co_edges().definition(id);
            (def.face_id == FaceId::start() && def.curve2d_rep_id.is_valid())
                .then_some((id, def.curve2d_rep_id))
        })
        .collect()
}
