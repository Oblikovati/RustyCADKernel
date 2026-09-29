//! BRepGraph behaviour the upstream suites do not reach in phase 1: population,
//! reconstruction and views over the hand-built shapes of the test support module
//! (TKPrim is not ported), and the Rust `Result` errors of native exceptions.
mod support;
use rustycad_tkbrep::brep_graph::BRepGraph;
use rustycad_tkbrep::brep_graph_inc::populate::{self, BuildStatus, Options};
use rustycad_tkbrep::topods::{Builder, Compound};
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use support::make_box;

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
