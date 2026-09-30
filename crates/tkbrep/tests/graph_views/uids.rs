#[test]
fn BRepGraph_ViewsTest__UIDsView_Of_Valid() {
    let graph = views_box();
    assert!(graph.uids().of_node(FaceId::new(0).into()).is_valid());
}

#[test]
fn BRepGraph_ViewsTest__UIDsView_Generation_Positive() {
    let graph = views_box();
    assert!(graph.uids().generation() > 0);
}

#[test]
fn BRepGraph_ViewsTest__UIDsView_NodeIdFrom_RoundTrip() {
    let graph = views_box();
    let id = FaceId::new(0).into();
    let uid = graph.uids().of_node(id);
    assert!(uid.is_valid());
    assert_eq!(graph.uids().node_id_from(&uid), id);
}

#[test]
fn BRepGraph_ViewsTest__UIDsView_NodeIdFrom_MultipleRoundTrip() {
    let graph = views_box();
    let face = graph.uids().of_node(FaceId::start().into());
    let edge = graph.uids().of_node(EdgeId::start().into());
    assert!(face.is_valid());
    assert!(edge.is_valid());
    let uids = [face, edge];
    assert_eq!(uids.len(), 2);
    assert_eq!(
        graph.uids().node_id_from(&uids[0]),
        NodeId::from(FaceId::start())
    );
    assert_eq!(
        graph.uids().node_id_from(&uids[1]),
        NodeId::from(EdgeId::start())
    );
}

#[test]
fn BRepGraph_ViewsTest__UIDsView_NodeIdFrom_InvalidAndUnknownUID() {
    let graph = views_box();
    let uids = [Uid::default(), Uid::new(NodeKind::FACE, 9999)];
    assert_eq!(uids.len(), 2);
    assert!(!graph.uids().node_id_from(&uids[0]).is_valid());
    assert!(!graph.uids().node_id_from(&uids[1]).is_valid());
}

#[test]
fn BRepGraph_ViewsTest__UIDsView_NodeLookup_RemovedNode_ReturnsInvalidAndHasFalse() {
    let mut graph = views_box();
    let id: NodeId = FaceId::new(0).into();
    let uid = graph.uids().of_node(id);
    assert!(uid.is_valid());
    assert!(graph.uids().has_node(&uid));
    assert_eq!(graph.uids().node_id_from(&uid), id);
    views_remove(&mut graph, id);
    assert!(!graph.uids().has_node(&uid));
    assert!(!graph.uids().node_id_from(&uid).is_valid());
}

#[test]
fn BRepGraph_ViewsTest__UIDsView_Of_RemovedNode_ReturnsInvalid() {
    let mut graph = views_box();
    let id: NodeId = FaceId::new(0).into();
    let uid = graph.uids().of_node(id);
    assert!(uid.is_valid());
    views_remove(&mut graph, id);
    assert!(!graph.uids().of_node(id).is_valid());
}

#[test]
fn BRepGraph_ViewsTest__UIDsView_RefLookup_RemovedRef_ReturnsInvalidAndHasFalse() {
    let mut graph = views_box();
    let id: RefId = views_face_ref(&graph).into();
    let uid = graph.uids().of_ref(id);
    assert!(uid.is_valid());
    assert!(graph.uids().has_ref(&uid));
    assert_eq!(graph.uids().ref_id_from(&uid), id);
    assert!(graph.editor().generic().remove_ref(id).unwrap());
    assert!(!graph.uids().has_ref(&uid));
    assert!(!graph.uids().ref_id_from(&uid).is_valid());
}

#[test]
fn BRepGraph_ViewsTest__UIDsView_Of_RemovedRef_ReturnsInvalid() {
    let mut graph = views_box();
    let id: RefId = views_face_ref(&graph).into();
    let uid = graph.uids().of_ref(id);
    assert!(uid.is_valid());
    assert!(graph.editor().generic().remove_ref(id).unwrap());
    assert!(!graph.uids().of_ref(id).is_valid());
}

#[test]
fn BRepGraph_ViewsTest__UIDsView_Of_OutOfRangeRef_ReturnsInvalid() {
    let graph = views_box();
    assert!(
        !graph
            .uids()
            .of_ref(FaceRefId::new(graph.refs().faces().nb()).into())
            .is_valid()
    );
}
