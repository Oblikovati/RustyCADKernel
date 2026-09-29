//! BRepGraph behaviour the upstream suites do not reach in phase 1: population,
//! reconstruction and views over the hand-built shapes of the test support module
//! (TKPrim is not ported), and the Rust `Result` errors of native exceptions.
mod support;
use rustycad_tkbrep::brep_graph::{BRepGraph, FaceId, NodeId, NodeKind, SolidId};
use rustycad_tkbrep::brep_graph_inc::populate::{self, BuildStatus, Options};
use rustycad_tkbrep::brep_graph_inc::reconstruct::{self, Cache};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Builder, Compound};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use support::make_box;
use support::ref_test_tools as tools;

fn populated(shape: &rustycad_tkbrep::topods::Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    // The support box lists its face edges out of chain order (not a closed oriented
    // loop), so the wire canonicalisation reports warnings; only failure is refused.
    let status = populate::perform(&mut graph, shape, false, &Options).unwrap();
    assert_ne!(status, BuildStatus::Failed);
    graph
}

#[test]
fn populate_box_counts_every_definition() {
    let graph = populated(&make_box(10.0, 20.0, 30.0));
    let topo = graph.topo();
    assert_eq!(topo.solids().nb(), 1);
    assert_eq!(topo.shells().nb(), 1);
    assert_eq!(topo.faces().nb(), 6);
    assert_eq!(topo.wires().nb(), 6);
    assert_eq!(topo.edges().nb(), 12);
    assert_eq!(topo.vertices().nb(), 8);
    assert_eq!(topo.co_edges().nb(), 24);
    assert!(graph.validate_relations());
}

#[test]
fn populate_compound_shares_located_children_definition_once() {
    let solid = make_box(1.0, 1.0, 1.0);
    let mut moved = Transform3::default();
    moved.set_translation(Vector3::new(5.0, 0.0, 0.0));
    let (builder, mut compound) = (Builder, Compound::default());
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &solid).unwrap();
    builder
        .add(
            &mut compound,
            &solid.located(&Location::new(moved), true).unwrap(),
        )
        .unwrap();
    builder.add(&mut compound, &solid).unwrap();
    let graph = populated(&compound);
    let topo = graph.topo();
    assert_eq!(topo.compounds().nb(), 1);
    assert_eq!(topo.solids().nb(), 1);
    assert_eq!(topo.faces().nb(), 6);
}

#[test]
fn populate_append_keeps_existing_definitions() {
    let solid = make_box(1.0, 2.0, 3.0);
    let mut graph = populated(&solid);
    let status = populate::append(&mut graph, &solid, false, &Options).unwrap();
    assert_ne!(status, BuildStatus::Failed);
    assert_eq!(graph.topo().solids().nb(), 1);
    let mut roots = Vec::new();
    populate::append_flattened(&mut graph, &solid, false, &mut roots, &Options).unwrap();
    assert_eq!(roots.len(), 6);
    assert_eq!(graph.topo().faces().nb(), 6);
}

fn count_of(shape: &rustycad_tkbrep::topods::Shape, kind: ShapeEnum) -> usize {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut map).unwrap();
    map.extent()
}

#[test]
fn reconstruct_box_shares_sub_shapes_and_round_trips() {
    let graph = populated(&make_box(10.0, 20.0, 30.0));
    let solid = reconstruct::node(&graph, SolidId::new(0).into()).unwrap();
    assert_eq!(solid.shape_type().unwrap(), ShapeEnum::Solid);
    assert_eq!(count_of(&solid, ShapeEnum::Face), 6);
    assert_eq!(count_of(&solid, ShapeEnum::Edge), 12);
    assert_eq!(count_of(&solid, ShapeEnum::Vertex), 8);
    let again = populated(&solid);
    assert_eq!(again.topo().co_edges().nb(), 24);
    assert_eq!(again.topo().edges().nb(), 12);
}

#[test]
fn reconstruct_cache_reuses_shapes_and_skips_non_shapes() {
    let graph = populated(&make_box(1.0, 1.0, 1.0));
    let mut cache = Cache::new();
    let face = reconstruct::face_with_cache(&graph, FaceId::new(0), &mut cache).unwrap();
    let same = reconstruct::node_with_cache(&graph, FaceId::new(0).into(), &mut cache).unwrap();
    assert!(face.is_same(&same));
    let co_edge = reconstruct::node(&graph, NodeId::new(NodeKind::CO_EDGE, 0)).unwrap();
    assert!(co_edge.is_null());
    let error = reconstruct::node(&graph, NodeId::new(NodeKind::EDGE, 99)).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

#[test]
fn ref_test_tools_read_the_populated_box() {
    let graph = populated(&make_box(1.0, 1.0, 1.0));
    let (shell, solid) = (
        rustycad_tkbrep::brep_graph::ShellId::new(0),
        SolidId::new(0),
    );
    assert_eq!(tools::count_face_refs_of_shell(&graph, shell), 6);
    assert_eq!(tools::count_shell_refs_of_solid(&graph, solid), 1);
    assert_eq!(tools::count_wire_refs_of_face(&graph, FaceId::new(0)), 1);
    let wire = graph
        .refs()
        .wires()
        .entry(tools::wire_refs_of_face(&graph, FaceId::new(0))[0])
        .child_wire_id;
    assert!(tools::face_uses_wire(&graph, FaceId::new(0), wire));
    assert_eq!(tools::count_co_edges_of_wire(&graph, wire), 4);
    assert_eq!(tools::count_child_refs_of_parent(&graph, solid.into()), 0);
}
