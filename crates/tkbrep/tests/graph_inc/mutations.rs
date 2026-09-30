fn inc_free_edge(graph: &mut BRepGraph, first: VertexId, last: VertexId) -> EdgeId {
    let id = graph
        .editor()
        .edges()
        .add(first, last, &Handle::default(), 0.0, 1.0, 1e-7);
    assert!(id.is_valid());
    id
}
fn inc_runtime_vertices(graph: &mut BRepGraph) -> [VertexId; 3] {
    [0.0, 1.0, 2.0].map(|x| {
        graph
            .editor()
            .vertices()
            .add(Point3::new(x, 0.0, 0.0), 1e-7)
    })
}
#[test]
fn Relations_EdgeOpsAdd_BindsStartEndVertices() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let a = graph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 1e-7);
    let b = graph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 0.0, 0.0), 1e-7);
    assert!(a.is_valid());
    assert!(b.is_valid());
    let id = inc_free_edge(&mut graph, a, b);
    assert!(graph.topo().vertices().edges(a).contains(&id));
    assert!(graph.topo().vertices().edges(b).contains(&id));
    assert!(graph.validate_relations());
}
#[test]
fn Relations_RemoveEdge_UnbindsStartEndVertices() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let a = graph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 1e-7);
    let b = graph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 0.0, 0.0), 1e-7);
    let id = inc_free_edge(&mut graph, a, b);
    graph.editor().generic().remove_node(id.into()).unwrap();
    assert!(!graph.topo().vertices().edges(a).contains(&id));
    assert!(!graph.topo().vertices().edges(b).contains(&id));
    assert!(graph.validate_relations());
}
#[test]
fn Relations_SetRefChildVertexId_RebindsVertexToEdges() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let [a, b, c] = inc_runtime_vertices(&mut graph);
    let id = inc_free_edge(&mut graph, a, b);
    let reference = graph.topo().edges().definition(id).start_vertex_ref_id;
    graph
        .editor()
        .vertices()
        .set_ref_child_vertex_id(reference, c)
        .unwrap();
    assert!(!graph.topo().vertices().edges(a).contains(&id));
    assert!(graph.topo().vertices().edges(c).contains(&id));
    assert!(graph.validate_relations());
}
#[test]
fn Relations_SetEndVertexRefId_RebindsVertexToEdges() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let [a, b, c] = inc_runtime_vertices(&mut graph);
    let id = inc_free_edge(&mut graph, a, b);
    let extra = inc_free_edge(&mut graph, a, c);
    let reference = graph.topo().edges().definition(extra).end_vertex_ref_id;
    let old = graph.topo().edges().definition(id).end_vertex_ref_id;
    // References are single-use slots; attempting to steal one must be a no-op.
    graph
        .editor()
        .edges()
        .set_end_vertex_ref_id(id, reference)
        .unwrap();
    assert!(graph.topo().vertices().edges(b).contains(&id));
    assert!(!graph.topo().vertices().edges(c).contains(&id));
    assert_eq!(graph.topo().edges().definition(id).end_vertex_ref_id, old);
    assert!(graph.validate_relations());
}
fn inc_remove_first_wire(graph: &mut BRepGraph) {
    for i in 0..graph.topo().faces().nb() {
        let id = FaceId::new(i);
        let refs = &graph.topo().faces().relations(id).wire_ref_ids;
        if refs.is_empty() {
            continue;
        }
        let reference = refs[0];
        assert!(graph.editor().faces().remove_wire(id, reference).unwrap());
        break;
    }
    assert!(graph.validate_relations());
}
#[test]
fn Relations_AfterEditorMutations_StaysConsistent() {
    let mut graph = inc_add(&inc_compound(&[inc_box(), inc_cylinder(12.0)]));
    assert!(graph.validate_relations());
    inc_remove_first_wire(&mut graph);
    inc_remove_first_face(&mut graph);
    inc_remove_first_shell(&mut graph);
    assert!(graph.editor().validate_mutation_boundary(None));
}
#[test]
fn Relations_BoxThroughCompact_StaysConsistent() {
    let mut graph = inc_add(&inc_box());
    let reference = graph.topo().faces().relations(FaceId::start()).wire_ref_ids[0];
    assert!(
        graph
            .editor()
            .faces()
            .remove_wire(FaceId::start(), reference)
            .unwrap()
    );
    assert!(graph.validate_relations());
    compact::perform(&mut graph).unwrap();
    assert!(graph.validate_relations());
}
#[test]
fn Relations_SetRefFaceId_RebindsFaceToShells() {
    let mut graph = inc_add(&inc_box());
    let reference = graph
        .topo()
        .shells()
        .relations(ShellId::start())
        .face_ref_ids[0];
    let old = graph.refs().faces().entry(reference).child_face_id;
    assert_ne!(old, FaceId::new(1));
    graph
        .editor()
        .faces()
        .set_ref_face_id(reference, FaceId::new(1))
        .unwrap();
    assert!(graph.validate_relations());
}
#[test]
fn Relations_SetRefChildShellId_RebindsShellToSolids() {
    let mut graph = inc_add(&inc_box());
    assert!(graph.topo().shells().nb() >= 1);
    let reference = graph
        .topo()
        .solids()
        .relations(SolidId::start())
        .shell_ref_ids[0];
    let shell = graph.refs().shells().entry(reference).child_shell_id;
    graph
        .editor()
        .shells()
        .set_ref_child_shell_id(reference, shell)
        .unwrap();
    assert!(graph.validate_relations());
}
#[test]
fn Relations_SetChildEdgeIdOnCoEdge_RebindsEdgeToCoEdges() {
    let mut graph = inc_add(&inc_box());
    assert!(graph.topo().co_edges().nb() >= 2);
    let id = CoEdgeId::start();
    let old = graph.topo().co_edges().definition(id).child_edge_id;
    let new = (0..graph.topo().edges().nb())
        .map(EdgeId::new)
        .find(|edge| *edge != old)
        .unwrap();
    graph
        .editor()
        .co_edges()
        .set_child_edge_id(id, new)
        .unwrap();
    assert!(!graph.topo().edges().co_edges(old).contains(&id));
    assert!(graph.topo().edges().co_edges(new).contains(&id));
    assert!(graph.validate_relations());
}
#[test]
fn Relations_SetFaceIdOnCoEdge_OnlyBondUnbinds() {
    let mut graph = inc_add(&inc_box());
    let id = CoEdgeId::start();
    let old = graph.topo().co_edges().definition(id).clone();
    graph
        .editor()
        .co_edges()
        .set_face_id(id, FaceId::default())
        .unwrap();
    assert!(
        !graph
            .topo()
            .edges()
            .faces_of(old.child_edge_id)
            .any(|f| f == old.face_id)
    );
    assert!(graph.validate_relations());
}
#[test]
fn Relations_SetterIdempotency_NoOp() {
    let mut graph = inc_add(&inc_box());
    let id = CoEdgeId::start();
    let def = graph.topo().co_edges().definition(id).clone();
    graph
        .editor()
        .co_edges()
        .set_child_edge_id(id, def.child_edge_id)
        .unwrap();
    graph
        .editor()
        .co_edges()
        .set_face_id(id, def.face_id)
        .unwrap();
    assert!(graph.validate_relations());
    inc_identity_vertex_ref(&mut graph);
}
#[test]
fn Relations_RemoveRef_UnbindsByKind() {
    let mut graph = inc_add(&inc_box());
    let shell = ShellId::start();
    let refs = &graph.topo().shells().relations(shell).face_ref_ids;
    assert!(!refs.is_empty());
    let reference = refs[0];
    let face = graph.refs().faces().entry(reference).child_face_id;
    assert!(
        graph
            .editor()
            .generic()
            .remove_ref(reference.into())
            .unwrap()
    );
    for reference in &graph.topo().faces().relations(face).parent_face_ref_ids {
        let parent = graph.refs().faces().entry(*reference).parent_shell_id;
        assert_ne!(parent, shell);
    }
    assert!(graph.validate_relations());
}

fn inc_remove_first_face(graph: &mut BRepGraph) {
    let shell = ShellId::start();
    let refs = &graph.topo().shells().relations(shell).face_ref_ids;
    assert!(!refs.is_empty());
    let reference = refs[0];
    assert!(
        graph
            .editor()
            .shells()
            .remove_face(shell, reference)
            .unwrap()
    );
    assert!(graph.validate_relations());
}
fn inc_remove_first_shell(graph: &mut BRepGraph) {
    let solid = SolidId::start();
    let refs = &graph.topo().solids().relations(solid).shell_ref_ids;
    assert!(!refs.is_empty());
    let reference = refs[0];
    assert!(
        graph
            .editor()
            .solids()
            .remove_shell(solid, reference)
            .unwrap()
    );
    assert!(graph.validate_relations());
}
fn inc_identity_vertex_ref(graph: &mut BRepGraph) {
    let reference = graph
        .topo()
        .edges()
        .definition(EdgeId::start())
        .start_vertex_ref_id;
    if reference.is_valid() {
        let vertex = graph.refs().vertices().entry(reference).child_vertex_id;
        graph
            .editor()
            .vertices()
            .set_ref_child_vertex_id(reference, vertex)
            .unwrap();
        assert!(graph.validate_relations());
    }
}
