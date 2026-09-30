fn inc_wire_faces(graph: &BRepGraph, wire: WireId) -> Vec<FaceId> {
    graph
        .topo()
        .wires()
        .relations(wire)
        .parent_wire_ref_ids
        .iter()
        .map(|id| graph.refs().wires().entry(*id).parent_face_id)
        .collect()
}
fn inc_detached_wire_context(graph: &BRepGraph, wire: WireId, face: FaceId) {
    for id in &graph.topo().wires().relations(wire).co_edge_ids {
        let def = graph.topo().co_edges().definition(*id);
        assert_ne!(def.face_id, face);
        for id in graph.topo().edges().faces_of(def.child_edge_id) {
            assert_ne!(id, face);
        }
    }
}
#[test]
fn Relations_SetRefChildWireId_RebindsWireToFaces() {
    let mut graph = inc_add(&inc_box());
    let face0 = FaceId::start();
    let face1 = FaceId::new(1);
    assert!(face1.is_valid_for(graph.topo().faces().nb()));
    let reference = graph.topo().faces().relations(face0).wire_ref_ids[0];
    let other = graph.topo().faces().relations(face1).wire_ref_ids[0];
    let old = graph.refs().wires().entry(reference).child_wire_id;
    let new = graph.refs().wires().entry(other).child_wire_id;
    assert_ne!(old, new);
    graph
        .editor()
        .wires()
        .set_ref_child_wire_id(reference, new)
        .unwrap();
    assert!(!inc_wire_faces(&graph, old).contains(&face0));
    assert!(inc_wire_faces(&graph, new).contains(&face0));
    inc_detached_wire_context(&graph, old, face0);
    assert!(graph.validate_relations());
}
fn inc_find_seam(graph: &BRepGraph, by_wire: bool) -> (EdgeId, CoEdgeId) {
    for i in 0..graph.topo().edges().nb() {
        let edge = EdgeId::new(i);
        if let Some(coedge) = inc_repeated_owner(graph, edge, by_wire) {
            return (edge, coedge);
        }
    }
    panic!("cylinder has no seam edge; expected two coedges sharing a valid owner");
}
#[test]
fn Relations_SetFaceIdOnCoEdge_LastBondCheck() {
    let mut graph = inc_add(&inc_cylinder(10.0));
    let (edge, coedge) = inc_find_seam(&graph, false);
    assert!(edge.is_valid());
    let face = graph.topo().co_edges().definition(coedge).face_id;
    graph
        .editor()
        .co_edges()
        .set_face_id(coedge, FaceId::default())
        .unwrap();
    assert!(graph.topo().edges().faces_of(edge).any(|id| id == face));
    assert!(graph.validate_relations());
}
#[test]
fn Relations_SetChildEdgeIdOnCoEdge_LastBondInWireCheck() {
    let mut graph = inc_add(&inc_cylinder(10.0));
    let (edge, coedge) = inc_find_seam(&graph, true);
    assert!(edge.is_valid());
    let wire = graph.topo().co_edges().definition(coedge).parent_wire_id;
    let target = (0..graph.topo().edges().nb())
        .map(EdgeId::new)
        .find(|id| *id != edge)
        .unwrap();
    assert!(target.is_valid());
    graph
        .editor()
        .co_edges()
        .set_child_edge_id(coedge, target)
        .unwrap();
    assert!(graph.topo().edges().wires_of(edge).any(|id| id == wire));
    assert!(graph.validate_relations());
}
fn inc_compound_parents(graph: &BRepGraph, node: NodeId) -> Vec<CompoundId> {
    graph
        .topo()
        .generic()
        .compound_ref_ids(node)
        .iter()
        .map(|id| graph.refs().children().entry(*id).parent_compound_id)
        .collect()
}
#[test]
fn Relations_SetChildRefChildNodeId_CrossKindRebinds() {
    let cube = support::make_box(2.0, 2.0, 2.0).into_shape();
    let mut graph = inc_add(&inc_compound(&[cube]));
    assert!(graph.topo().compounds().nb() >= 1);
    assert!(graph.topo().solids().nb() >= 1);
    assert!(graph.topo().shells().nb() >= 1);
    let compound = CompoundId::start();
    let refs = &graph.topo().compounds().relations(compound).child_ref_ids;
    assert!(!refs.is_empty());
    let reference = refs[0];
    let old = graph.refs().children().entry(reference).child_node_id;
    assert_eq!(old.kind, NodeKind::SOLID);
    inc_rebind_compound_child(&mut graph, reference, old);
}
fn inc_runtime_wire(graph: &mut BRepGraph) -> (EdgeId, CoEdgeId, WireId) {
    let a = graph
        .editor()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 1e-7);
    let b = graph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 0.0, 0.0), 1e-7);
    let edge = inc_free_edge(graph, a, b);
    let coedge = graph
        .editor()
        .co_edges()
        .add(edge, ParityOrientation::FORWARD);
    assert!(coedge.is_valid());
    let wire = graph.editor().wires().add(&[coedge]);
    assert!(wire.is_valid());
    (edge, coedge, wire)
}
fn inc_runtime_face(graph: &mut BRepGraph, wire: WireId) -> FaceId {
    let face = graph.editor().faces().add(&inc_plane(), wire, &[], 1e-7);
    assert!(face.is_valid());
    face
}
fn inc_runtime_pcurve(graph: &mut BRepGraph, coedge: CoEdgeId, face: FaceId) -> CoEdgeCurve2DRepId {
    graph.editor().co_edges().set_face_id(coedge, face).unwrap();
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
    ))
    .into_cast();
    graph
        .editor()
        .co_edges()
        .set_pcurve(coedge, &line, 0.0, 1.0)
        .unwrap();
    let rep = graph.topo().co_edges().definition(coedge).curve2d_rep_id;
    assert!(rep.is_valid());
    assert!(graph.validate_relations());
    rep
}
#[test]
fn Relations_RemoveWireFromSharedWireClearsDetachedFaceContext() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let (edge, coedge, wire) = inc_runtime_wire(&mut graph);
    let face0 = inc_runtime_face(&mut graph, wire);
    let _face1 = inc_runtime_face(&mut graph, wire);
    inc_runtime_pcurve(&mut graph, coedge, face0);
    let old = graph.topo().geometry().nb_active_co_edge_curves2d();
    let reference = graph.topo().faces().relations(face0).wire_ref_ids[0];
    assert!(reference.is_valid());
    assert!(
        graph
            .editor()
            .faces()
            .remove_wire(face0, reference)
            .unwrap()
    );
    inc_assert_detached_pcurve(&graph, coedge, edge, face0, old);
}
#[test]
fn Relations_ValidateRejectsLivePCurveWithoutFaceContext() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let (_edge, coedge, wire) = inc_runtime_wire(&mut graph);
    let face = inc_runtime_face(&mut graph, wire);
    inc_runtime_pcurve(&mut graph, coedge, face);
    {
        let mut guard = graph.editor().co_edges().mutate(coedge).unwrap();
        guard.internal().face_id = FaceId::default();
    }
    assert!(!graph.validate_relations());
}

fn inc_repeated_owner(graph: &BRepGraph, edge: EdgeId, by_wire: bool) -> Option<CoEdgeId> {
    let mut seen = Vec::new();
    for id in graph.topo().edges().co_edges(edge) {
        let def = graph.topo().co_edges().definition(*id);
        let owner = if by_wire {
            def.parent_wire_id.into()
        } else {
            NodeId::from(def.face_id)
        };
        if !owner.is_valid() {
            continue;
        }
        if seen.contains(&owner) {
            return Some(*id);
        }
        seen.push(owner);
    }
    None
}
fn inc_rebind_compound_child(graph: &mut BRepGraph, reference: ChildRefId, old: NodeId) {
    let shell = ShellId::start().into();
    graph
        .editor()
        .generic()
        .set_child_ref_child_node_id(reference, shell)
        .unwrap();
    assert!(!inc_compound_parents(graph, old).contains(&CompoundId::start()));
    assert!(inc_compound_parents(graph, shell).contains(&CompoundId::start()));
    assert!(graph.validate_relations());
}
fn inc_assert_detached_pcurve(
    graph: &BRepGraph,
    coedge: CoEdgeId,
    edge: EdgeId,
    face0: FaceId,
    old: u32,
) {
    let def = graph.topo().co_edges().definition(coedge);
    assert!(!def.face_id.is_valid());
    assert!(!def.curve2d_rep_id.is_valid());
    assert_eq!(
        graph.topo().geometry().nb_active_co_edge_curves2d(),
        old - 1
    );
    for id in graph.topo().edges().faces_of(edge) {
        assert_ne!(id, face0);
    }
    assert!(graph.validate_relations());
}
