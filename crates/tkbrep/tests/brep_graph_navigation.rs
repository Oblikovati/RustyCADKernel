//! BRepGraph navigation behaviour over the support box (TKPrim is not ported): the
//! definition and reference iterators, the reverse iterators, the related iterator,
//! the child and parent explorers, `BRepGraph_Tool` and `BRepGraph_CopyRemap`. The
//! upstream suites of these classes wait for the editor (docs/tkbrep-progress.md);
//! these tests pin the box invariants and the assembly placements the native probe
//! (`br_bgraph_walk`) compares bit for bit.
mod support;
use rustycad_tkbrep::brep_graph::child_explorer::{self, ChildExplorer};
use rustycad_tkbrep::brep_graph::defs_iterator::{
    DefsCoEdgeOfWire, DefsEdgeOfWire, DefsFaceOfShell, DefsShellOfSolid, DefsVertexOfEdge,
    DefsWireOfFace,
};
use rustycad_tkbrep::brep_graph::iterator::{EdgeIterator, FaceIterator, RootProductIterator};
use rustycad_tkbrep::brep_graph::parent_explorer::{self, ParentExplorer};
use rustycad_tkbrep::brep_graph::refs_iterator::{CoEdgesOfWire, FaceRefIterator, RefsFaceOfShell};
use rustycad_tkbrep::brep_graph::related_iterator::{RelatedIterator, RelationKind};
use rustycad_tkbrep::brep_graph::reverse_iterator::{
    EdgesOfVertex, RefsEdgesOfVertex, ShellsOfFace, SolidsOfShell,
};
use rustycad_tkbrep::brep_graph::{
    BRepGraph, EdgeId, FaceId, NodeId, NodeKind, ShellId, SolidId, VertexId, tool,
};
use rustycad_tkbrep::brep_graph_inc::populate::{self, BuildStatus, Options};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use support::make_box;

fn box_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    let status =
        populate::perform(&mut graph, &make_box(10.0, 20.0, 30.0), false, &Options).unwrap();
    assert_ne!(status, BuildStatus::Failed);
    graph
}

fn kinds_of(instances: impl Iterator<Item = NodeId>) -> Vec<i32> {
    instances.map(|node| node.kind.0).collect()
}

#[test]
fn dense_iterators_visit_every_definition() {
    let graph = box_graph();
    assert_eq!(FaceIterator::new(&graph).count(), 6);
    assert_eq!(EdgeIterator::new(&graph).count(), 12);
    assert_eq!(FaceRefIterator::new(&graph).count(), 6);
    assert_eq!(RootProductIterator::new(&graph).count(), 0);
    let mut iterator = FaceIterator::starting_at(&graph, FaceId::new(4));
    assert_eq!(iterator.current_id(), FaceId::new(4));
    iterator.next();
    iterator.next();
    assert!(!iterator.more());
}

#[test]
fn defs_and_refs_walk_the_box_one_level_at_a_time() {
    let graph = box_graph();
    let shells: Vec<ShellId> = collect_ids(DefsShellOfSolid::new(&graph, SolidId::new(0)));
    assert_eq!(shells, vec![ShellId::new(0)]);
    let faces = DefsFaceOfShell::new(&graph, ShellId::new(0));
    assert_eq!(faces.count(), 6);
    assert_eq!(RefsFaceOfShell::new(&graph, ShellId::new(0)).count(), 6);
    for face in 0..6 {
        let wires: Vec<_> = collect_ids(DefsWireOfFace::new(&graph, FaceId::new(face)));
        assert_eq!(wires.len(), 1);
        assert_eq!(DefsCoEdgeOfWire::new(&graph, wires[0]).count(), 4);
        assert_eq!(DefsEdgeOfWire::new(&graph, wires[0]).count(), 4);
        assert_eq!(CoEdgesOfWire::new(&graph, wires[0]).count(), 4);
    }
    for edge in 0..12 {
        assert_eq!(DefsVertexOfEdge::new(&graph, EdgeId::new(edge)).count(), 2);
    }
}

// The ids of a DefsOfParent walk (its `Iterator` yields definitions).
fn collect_ids<P: rustycad_tkbrep::brep_graph::parent_traits::ParentTraits>(
    mut iterator: rustycad_tkbrep::brep_graph::defs_iterator::DefsOfParent<'_, P>,
) -> Vec<P::Child> {
    let mut ids = Vec::new();
    while iterator.more() {
        ids.push(iterator.current_id());
        iterator.next();
    }
    ids
}

#[test]
fn reverse_iterators_find_the_box_parents() {
    let graph = box_graph();
    let topo = graph.topo();
    for edge in 0..12 {
        let edge = EdgeId::new(edge);
        assert_eq!(topo.edges().faces_of(edge).count(), 2);
        assert_eq!(topo.edges().wires_of(edge).count(), 2);
        assert!(tool::edge::is_manifold(&graph, edge));
    }
    for vertex in 0..8 {
        let vertex = VertexId::new(vertex);
        let edges = topo.vertices().edges(vertex);
        assert_eq!(EdgesOfVertex::new(&graph, edges).count(), 3);
        assert_eq!(RefsEdgesOfVertex::new(&graph, edges, vertex).count(), 3);
    }
    let face_refs = &topo.faces().relations(FaceId::new(0)).parent_face_ref_ids;
    assert_eq!(ShellsOfFace::new(&graph, face_refs).count(), 1);
    let shell_refs = &topo
        .shells()
        .relations(ShellId::new(0))
        .parent_shell_ref_ids;
    assert_eq!(SolidsOfShell::new(&graph, shell_refs).count(), 1);
}

#[test]
fn related_nodes_of_a_face_edge_and_vertex() {
    let graph = box_graph();
    let related: Vec<(NodeId, RelationKind)> = related_of(&graph, FaceId::new(0).into());
    let count = |kind: RelationKind| related.iter().filter(|(_, r)| *r == kind).count();
    assert_eq!(count(RelationKind::BoundaryEdge), 4);
    assert_eq!(count(RelationKind::AdjacentFace), 4);
    assert_eq!(count(RelationKind::OuterWire), 1);
    let edge = related_of(&graph, EdgeId::new(0).into());
    assert_eq!(edge.len(), 4);
    assert_eq!(related_of(&graph, VertexId::new(0).into()).len(), 3);
    assert!(related_of(&graph, SolidId::new(0).into()).is_empty());
}

fn related_of(graph: &BRepGraph, node: NodeId) -> Vec<(NodeId, RelationKind)> {
    let mut iterator = RelatedIterator::new(graph, node).unwrap();
    let mut found = Vec::new();
    while iterator.more() {
        found.push((iterator.current(), iterator.current_relation()));
        iterator.next().unwrap();
    }
    found
}

#[test]
fn child_explorer_visits_each_use() {
    let graph = box_graph();
    let solid: NodeId = SolidId::new(0).into();
    let faces = ChildExplorer::with_target(&graph, solid, NodeKind::FACE).unwrap();
    assert_eq!(faces.count(), 6);
    let vertices = ChildExplorer::with_target(&graph, solid, NodeKind::VERTEX).unwrap();
    assert_eq!(vertices.count(), 48);
    let direct =
        ChildExplorer::with_mode(&graph, solid, child_explorer::TraversalMode::DirectChildren)
            .unwrap();
    let kinds = kinds_of(direct.map(|instance| instance.unwrap().def_id));
    assert_eq!(kinds, vec![NodeKind::SHELL.0]);
    let everything = ChildExplorer::new(&graph, solid).unwrap().count();
    // shell, 6 faces, 6 wires, 24 coedges, 24 edges, 48 vertices.
    assert_eq!(everything, 1 + 6 + 6 + 24 + 24 + 48);
    let pruned = ChildExplorer::with_avoid(
        &graph,
        solid,
        Some(NodeKind::WIRE),
        true,
        child_explorer::TraversalMode::Recursive,
    )
    .unwrap()
    .count();
    assert_eq!(pruned, 1 + 6 + 6);
}

#[test]
fn child_explorer_reports_paths_and_links() {
    let graph = box_graph();
    let mut explorer = ChildExplorer::with_target(&graph, SolidId::new(0), NodeKind::EDGE).unwrap();
    assert!(explorer.more());
    assert_eq!(explorer.current_parent().kind, NodeKind::CO_EDGE);
    assert_eq!(
        explorer.current_link_kind(),
        child_explorer::LinkKind::Structural
    );
    assert!(!explorer.current_ref().is_valid());
    assert_eq!(explorer.current_usage_path().size(), 6);
    assert_eq!(explorer.node_of(NodeKind::FACE).kind, NodeKind::FACE);
    assert_eq!(explorer.node_at(0).kind, NodeKind::SHELL);
    assert!(!explorer.node_at(-2).is_valid());
    assert_eq!(explorer.depth(), 6);
    explorer.next().unwrap();
    assert!(explorer.more());
}

#[test]
fn parent_explorer_walks_every_path() {
    let graph = box_graph();
    let vertex: NodeId = VertexId::new(0).into();
    let solids = ParentExplorer::with_target(&graph, vertex, NodeKind::SOLID).unwrap();
    // 3 edges, each used by 2 coedges, each in a wire of a face of the one shell.
    assert_eq!(solids.count(), 6);
    let direct = ParentExplorer::with_mode(
        &graph,
        vertex,
        parent_explorer::TraversalMode::DirectParents,
    )
    .unwrap();
    assert_eq!(
        kinds_of(direct.map(|instance| instance.unwrap().def_id)),
        vec![NodeKind::EDGE.0; 3]
    );
    let mut explorer = ParentExplorer::new(&graph, FaceId::new(0).into()).unwrap();
    assert_eq!(explorer.current().def_id.kind, NodeKind::SHELL);
    assert_eq!(
        explorer.current_link_kind(),
        parent_explorer::LinkKind::Reference
    );
    assert!(explorer.current_ref().is_valid());
    explorer.next().unwrap();
    assert_eq!(explorer.current().def_id.kind, NodeKind::SOLID);
    assert!(explorer.is_current_branch_root());
    explorer.next().unwrap();
    assert!(!explorer.more());
    assert!(!explorer.current().is_valid());
}

#[test]
fn tool_reads_the_box_geometry() {
    let graph = box_graph();
    assert!(tool::shell::is_closed(&graph, ShellId::new(0)));
    assert_eq!(tool::shell::nb_faces(&graph, ShellId::new(0)), 6);
    for edge in 0..12 {
        let edge = EdgeId::new(edge);
        assert!(tool::edge::has_curve(&graph, edge));
        assert!(!tool::edge::degenerated(&graph, edge));
        assert!(!tool::edge::is_closed(&graph, edge));
        let faces: Vec<FaceId> = graph.topo().edges().faces_of(edge).collect();
        assert!(tool::edge::find_co_edge_id(&graph, edge, faces[0]).is_valid());
        let pcurve = tool::edge::find_pcurve_co_edge_id_oriented(
            &graph,
            edge,
            faces[1],
            Orientation::Reversed,
        );
        assert!(tool::co_edge::has_pcurve(&graph, pcurve));
        assert!(!tool::edge::is_seam_on_face(&graph, edge, faces[0]));
    }
    for face in 0..6 {
        let face = FaceId::new(face);
        let outer = tool::face::outer_wire(&graph, face).unwrap();
        assert!(outer.is_valid());
        assert!(tool::wire::is_outer(&graph, outer).unwrap());
        assert_eq!(tool::wire::face_of(&graph, outer), face);
        assert_eq!(tool::wire::nb_distinct_edges(&graph, outer), 4);
        assert!(tool::face::has_surface(&graph, face));
    }
    for co_edge in 0..24 {
        let co_edge = rustycad_tkbrep::brep_graph::CoEdgeId::new(co_edge);
        assert!(tool::co_edge::same_range(&graph, co_edge));
        assert!(tool::co_edge::same_parameter(&graph, co_edge).unwrap());
        assert!(!tool::co_edge::is_seam(&graph, co_edge));
        assert!(
            tool::co_edge::pcurve_adaptor(&graph, co_edge)
                .unwrap()
                .is_initialized()
        );
    }
}

#[test]
fn tool_usages_apply_their_location() {
    let graph = box_graph();
    let edge = EdgeId::new(0);
    let start = tool::edge::start_vertex_id(&graph, edge);
    let usage = tool::vertex::usage(&graph, start);
    assert!(usage.is_valid());
    let mut moved = usage.clone();
    let mut shift = Transform3::default();
    shift.set_translation(Vector3::new(0.0, 0.0, 5.0));
    moved.location = Location::new(shift);
    let here = tool::vertex::pnt_usage(&graph, &usage);
    let there = tool::vertex::pnt_usage(&graph, &moved);
    assert_eq!(there.z() - here.z(), 5.0);
    assert_eq!(tool::vertex::pnt_ref(&graph, start), here);
    assert_eq!(tool::vertex::nb_edges(&graph, usage.def_id), 3);
}
