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
    assert_null_perform_clears(&mut graph);
}
fn assert_null_perform_clears(graph: &mut BRepGraph) {
    use rustycad_tkbrep::brep_graph_inc::populate::{self, PCurveImportBehavior::LegacyLocalV1};
    let result = populate::perform_with_behavior(
        graph,
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

#[test]
fn cumulative_v2_translated_face_retains_authored_pcurve() {
    use rustycad_tkbrep::brep_graph_inc::populate::PCurveImportBehavior::CumulativeOccurrenceV2;
    let shape = translated(&single_use_face());
    let options = Options {
        create_auto_product: false,
        ..Options::default()
    };
    let mut graph = BRepGraph::new();
    graph
        .shapes_mut()
        .add_with_options_and_behavior(&shape, &options, CumulativeOccurrenceV2)
        .unwrap();
    assert_eq!(
        baseline_row("v2", &graph),
        "v2: stored=true range=0,1 reversed=false\n"
    );
}
#[path = "graph_pcurve_placement/contracts.rs"]
mod contracts;
#[path = "graph_pcurve_placement/nested.rs"]
mod nested;
#[path = "graph_pcurve_placement/occurrences.rs"]
mod occurrences;
#[path = "graph_pcurve_placement/seam.rs"]
mod seam;

#[path = "graph_pcurve_placement/source_snapshot.rs"]
mod source_snapshot;
