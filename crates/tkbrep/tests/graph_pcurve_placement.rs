// SPDX-License-Identifier: GPL-2.0-only
//! Owned import replay baseline; no live OCCT or geometric acceptance claim.
use rustycad_tkbrep::{
    brep_graph::{BRepGraph, CoEdgeId, shapes_view::Options, tool::co_edge},
    topods::Shape,
};
#[path = "graph_pcurve_placement/fixtures.rs"]
mod fixtures;
use fixtures::*;
#[test]
fn legacy_local_v1_matches_owned_placement_baseline() {
    let face = single_use_face();
    let moved = translated(&face);
    let actual = baseline_row("identity/no-product", &imported(&face, false))
        + &baseline_row("translated/no-product", &imported(&moved, false))
        + &baseline_row("translated/auto-product", &imported(&moved, true));
    assert_eq!(actual, include_str!("fixtures/graph-pcurve-import-v1.txt"));
}
fn baseline_row(name: &str, graph: &BRepGraph) -> String {
    assert_eq!(graph.topo().co_edges().nb(), 1);
    let coedge = CoEdgeId::new(0);
    let definition = graph.topo().co_edges().definition(coedge);
    let range = co_edge::range(graph, coedge);
    format!(
        "{name}: stored={} range={},{} reversed={}\n",
        co_edge::has_pcurve(graph, coedge),
        range.0,
        range.1,
        definition.orientation.is_reversed
    )
}
#[test]
fn explicit_legacy_entrypoints_retain_replay_and_null_status() {
    use rustycad_tkbrep::brep_graph_inc::populate::{self, PCurveImportBehavior::LegacyLocalV1};
    let moved = translated(&single_use_face());
    let mut graph = BRepGraph::new();
    let options = Options {
        create_auto_product: false,
        ..Options::default()
    };
    graph
        .shapes_mut()
        .add_with_options_and_behavior(&moved, &options, LegacyLocalV1)
        .unwrap();
    let expected = baseline_row("legacy", &graph);
    populate::perform_with_behavior(&mut graph, &moved, false, &populate::Options, LegacyLocalV1)
        .unwrap();
    assert_eq!(baseline_row("legacy", &graph), expected);
    assert_legacy_append_entries(&moved, &expected);
    let result = populate::perform_with_behavior(
        &mut graph,
        &Shape::default(),
        false,
        &populate::Options,
        LegacyLocalV1,
    )
    .unwrap();
    assert_eq!(result, populate::BuildStatus::Failed);
    assert_eq!(graph.topo().co_edges().nb(), 0);
}
fn assert_legacy_append_entries(shape: &Shape, expected: &str) {
    use rustycad_tkbrep::brep_graph_inc::populate::{self, PCurveImportBehavior::LegacyLocalV1};
    let mut graph = BRepGraph::new();
    populate::append_with_behavior(&mut graph, shape, false, &populate::Options, LegacyLocalV1)
        .unwrap();
    assert_eq!(baseline_row("legacy", &graph), expected);
    let mut graph = BRepGraph::new();
    let mut roots = Vec::new();
    populate::append_flattened_with_behavior(
        &mut graph,
        shape,
        false,
        &mut roots,
        &populate::Options,
        LegacyLocalV1,
    )
    .unwrap();
    assert_eq!(baseline_row("legacy", &graph), expected);
    assert_eq!(roots.len(), 1);
}
