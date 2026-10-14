// SPDX-License-Identifier: GPL-2.0-only
//! Two closed rims per cap and one seam per loop imply V=4,E=6,F=4 and 12 uses.
use super::*;
use rustycad_tkbrep::{
    brep_graph::{BRepGraph, shapes_view::Options, validate},
    brep_graph_inc::populate::PCurveImportBehavior::CumulativeOccurrenceV2,
};

pub(super) fn assert_body(diagnostic: &PrismDiagnostic) {
    let graph = import(diagnostic);
    assert_eq!(graph.topo().faces().nb(), 4);
    assert_eq!(graph.topo().wires().nb(), 6);
    assert_eq!(graph.topo().edges().nb(), 6);
    assert_eq!(graph.topo().vertices().nb(), 4);
    assert_eq!(graph.topo().co_edges().nb(), 12);
    let audit = validate::perform_with_mode(&graph, validate::Mode::Audit);
    assert!(
        audit.issues.is_empty(),
        "actual annular graph issues: {:?}",
        audit.issues
    );
}
fn import(diagnostic: &PrismDiagnostic) -> BRepGraph {
    let options = Options {
        create_auto_product: false,
        track_added_nodes: true,
        ..Options::default()
    };
    let mut graph = BRepGraph::new();
    graph
        .shapes_mut()
        .add_with_options_and_behavior(&diagnostic.body, &options, CumulativeOccurrenceV2)
        .expect("actual annular graph import");
    graph
}
