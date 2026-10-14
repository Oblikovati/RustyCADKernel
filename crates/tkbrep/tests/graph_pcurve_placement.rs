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
