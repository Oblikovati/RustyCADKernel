//! Identity and generation mirrors from pinned BRepGraph_Compact_Test.cxx.
use super::*;

fn identity_box() -> BRepGraph {
    let graph = compact_graph(&make_box(10.0, 20.0, 30.0));
    assert!(!graph.is_empty());
    assert!(graph.topo().faces().nb() >= 2);
    graph
}

fn surviving_wire_ref(graph: &BRepGraph) -> WireRefId {
    let relations = graph.topo().faces().relations(FaceId::new(1));
    assert!(!relations.wire_ref_ids.is_empty());
    relations.wire_ref_ids[0]
}

fn surviving_coedge(graph: &BRepGraph) -> CoEdgeId {
    let wire = graph
        .refs()
        .wires()
        .entry(surviving_wire_ref(graph))
        .child_wire_id;
    let relations = graph.topo().wires().relations(wire);
    assert!(!relations.co_edge_ids.is_empty());
    relations.co_edge_ids[0]
}

fn remove_first_face(graph: &mut BRepGraph) -> compact::CompactResult {
    graph
        .editor()
        .generic()
        .remove_node(FaceId::start().into())
        .unwrap();
    compact::perform(graph).unwrap()
}

#[test]
fn BRepGraph_CompactTest__CoEdgeUID_AfterCompaction() {
    let mut graph = identity_box();
    assert!(graph.topo().wires().nb() >= 1);
    let coedge = surviving_coedge(&graph);
    assert!(coedge.is_valid());
    let uid = graph.uids().of_node(coedge.into());
    assert!(uid.is_valid());
    let result = remove_first_face(&mut graph);
    assert!(result.nb_nodes_before > result.nb_nodes_after);
    let resolved = graph.uids().node_id_from(&uid);
    assert!(resolved.is_valid());
    assert_eq!(resolved.kind, NodeKind::CO_EDGE);
}

fn surviving_face_ref(graph: &BRepGraph) -> FaceRefId {
    (0..graph.topo().shells().nb())
        .flat_map(|index| {
            graph
                .topo()
                .shells()
                .relations(ShellId::new(index))
                .face_ref_ids
                .clone()
        })
        .find(|reference| graph.refs().faces().entry(*reference).child_face_id.index != 0)
        .unwrap_or_default()
}

fn surviving_shell_ref(graph: &BRepGraph) -> ShellRefId {
    (0..graph.topo().solids().nb())
        .flat_map(|index| {
            graph
                .topo()
                .solids()
                .relations(SolidId::new(index))
                .shell_ref_ids
                .clone()
        })
        .next()
        .unwrap_or_default()
}

fn reference_uids(graph: &BRepGraph) -> Vec<(RefUid, RefKind)> {
    let vertex = graph
        .topo()
        .edges()
        .definition(EdgeId::start())
        .start_vertex_ref_id;
    let references: [RefId; 4] = [
        vertex.into(),
        surviving_wire_ref(graph).into(),
        surviving_face_ref(graph).into(),
        surviving_shell_ref(graph).into(),
    ];
    references
        .into_iter()
        .filter(|reference| reference.is_valid())
        .map(|reference| (graph.uids().of_ref(reference), reference.kind))
        .collect()
}

#[test]
fn BRepGraph_CompactTest__UIDRoundTrip_RefUIDs_AfterCompaction() {
    let mut graph = identity_box();
    let references = reference_uids(&graph);
    let coedge_uid = graph.uids().of_node(surviving_coedge(&graph).into());
    remove_first_face(&mut graph);
    for (uid, kind) in references.into_iter().filter(|(uid, _)| uid.is_valid()) {
        let resolved = graph.uids().ref_id_from(&uid);
        assert!(resolved.is_valid());
        assert_eq!(resolved.kind, kind);
    }
    if coedge_uid.is_valid() {
        let resolved = graph.uids().node_id_from(&coedge_uid);
        assert!(resolved.is_valid());
        assert_eq!(resolved.kind, NodeKind::CO_EDGE);
    }
}

fn mutate_geometry_generation(graph: &mut BRepGraph, offset: f64) {
    let point = Point3::new(10.0 + offset, 20.0 + offset, 30.0 + offset);
    graph
        .editor()
        .vertices()
        .set_point(VertexId::start(), point)
        .unwrap();
    graph
        .editor()
        .edges()
        .set_tolerance(EdgeId::start(), offset / 10.0)
        .unwrap();
    graph
        .editor()
        .faces()
        .set_tolerance(FaceId::start(), offset / 100.0)
        .unwrap();
}

fn mark_container_generations(graph: &mut BRepGraph) {
    let mut wire = graph.editor().wires().mutate(WireId::start()).unwrap();
    wire.mark_dirty();
    drop(wire);
    let mut shell = graph.editor().shells().mutate(ShellId::start()).unwrap();
    shell.mark_dirty();
    drop(shell);
    let mut solid = graph.editor().solids().mutate(SolidId::start()).unwrap();
    solid.mark_dirty();
}

fn mutate_six_topology_kinds(graph: &mut BRepGraph) {
    for step in 1..=3 {
        mutate_geometry_generation(graph, f64::from(step));
        mark_container_generations(graph);
    }
}

fn assert_topology_generations(graph: &BRepGraph) {
    let topo = graph.topo();
    assert_eq!(
        topo.vertices().definition(VertexId::start()).base.own_gen,
        3
    );
    assert_eq!(topo.edges().definition(EdgeId::start()).base.own_gen, 3);
    assert_eq!(topo.wires().definition(WireId::start()).base.own_gen, 3);
    assert_eq!(topo.faces().definition(FaceId::start()).base.own_gen, 3);
    assert_eq!(topo.shells().definition(ShellId::start()).base.own_gen, 3);
    assert_eq!(topo.solids().definition(SolidId::start()).base.own_gen, 3);
}

#[test]
fn BRepGraph_CompactTest__OwnGen_PreservedForAllTopologyKindsAfterCompact() {
    let mut graph = identity_box();
    mutate_six_topology_kinds(&mut graph);
    assert_topology_generations(&graph);
    compact::perform(&mut graph).unwrap();
    assert_topology_generations(&graph);
}
