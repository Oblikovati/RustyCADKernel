#[test]
fn BRepGraph_ViewsTest__MutView_EdgeDef_IncrementsOwnGen() {
    let mut graph = views_box();
    let mut guard = graph.editor().edges().mutate(EdgeId::start()).unwrap();
    guard.mark_dirty();
    drop(guard);
    assert!(
        graph
            .topo()
            .edges()
            .definition(EdgeId::start())
            .base
            .own_gen
            > 0
    );
}

#[test]
fn BRepGraph_ViewsTest__MutView_InvalidNode_ThrowsProgramError() {
    let mut graph = views_box();
    let error = graph
        .editor()
        .faces()
        .mutate(FaceId::new(777777))
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
}

#[test]
fn BRepGraph_ViewsTest__MutView_RemovedNode_ThrowsProgramError() {
    let mut graph = views_box();
    let id = FaceId::new(0);
    views_remove(&mut graph, id);
    let error = graph.editor().faces().mutate(id).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
}

#[test]
fn BRepGraph_ViewsTest__MutView_RemovedRef_ThrowsProgramError() {
    let mut graph = views_box();
    let id = views_face_ref(&graph);
    assert!(graph.editor().generic().remove_ref(id.into()).unwrap());
    let error = graph.editor().faces().mutate_ref(id).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
}

#[test]
fn BRepGraph_ViewsTest__EditorView_AddVertex_Works() {
    let mut graph = views_box();
    let before = graph.topo().vertices().nb();
    let vertex = graph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 2.0, 3.0), 0.001);
    assert!(vertex.is_valid());
    assert_eq!(graph.topo().vertices().nb(), before + 1);
}

#[test]
fn BRepGraph_ViewsTest__EditorView_IsRemoved_False() {
    let graph = views_box();
    assert!(!graph.topo().generic().is_removed(FaceId::new(0).into()));
}

#[test]
fn BRepGraph_ViewsTest__History_ConstLookup() {
    let graph = views_box();
    let history = graph.layer_registry().find::<LayerHistory>();
    assert!(!history.is_null());
    assert!(history.get().unwrap().is_enabled());
}

#[test]
fn BRepGraph_ViewsTest__History_MutableLookup() {
    let graph = views_box();
    views_history(&graph).get().unwrap().set_enabled(false);
    assert!(!views_history(&graph).get().unwrap().is_enabled());
    views_history(&graph).get().unwrap().set_enabled(true);
    assert!(views_history(&graph).get().unwrap().is_enabled());
}
fn views_history(graph: &BRepGraph) -> rustycad_tkernel::standard::Handle<LayerHistory> {
    graph.layer_registry().ensure::<LayerHistory>().unwrap()
}
