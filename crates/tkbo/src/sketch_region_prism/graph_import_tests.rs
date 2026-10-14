// SPDX-License-Identifier: GPL-2.0-only
use super::{
    tests::{cubic_receipt, progress},
    *,
};
use rustycad_tkbrep::{
    brep_graph::{BRepGraph, CoEdgeId, NodeId, shapes_view::Options, validate},
    topods::Shape,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
fn imported(
    shape: &Shape,
    behavior: rustycad_tkbrep::brep_graph_inc::populate::PCurveImportBehavior,
) -> BRepGraph {
    let mut graph = BRepGraph::new();
    let options = Options {
        create_auto_product: false,
        track_added_nodes: true,
        ..Options::default()
    };
    graph
        .shapes_mut()
        .add_with_options_and_behavior(shape, &options, behavior)
        .unwrap();
    graph
}
fn last_cap(receipt: &PrismReceipt) -> &Shape {
    let bindings = &receipt.diagnostic.as_ref().unwrap().bindings;
    let matches: Vec<_> = bindings
        .iter()
        .filter(|b| {
            b.native.role == PrismImageRole::Last
                && b.native.query.shape_type().unwrap() == ShapeEnum::Face
        })
        .collect();
    assert_eq!(matches.len(), 1);
    &matches[0].context.occurrence
}
#[test]
fn legacy_prism_import_missing_pcurve_is_bound_to_last_cap() {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let graph = imported(
        &receipt.diagnostic.as_ref().unwrap().body,
        Default::default(),
    );
    let cap_id = graph.shapes().find_node(last_cap(&receipt));
    let audit = validate::perform_with_mode(&graph, validate::Mode::Audit);
    assert_eq!(audit.issues.len(), 1);
    let coedge = CoEdgeId::from_node_id(audit.issues[0].node_id);
    let issue_face: NodeId = graph.topo().co_edges().definition(coedge).face_id.into();
    assert_eq!(
        cap_id, issue_face,
        "missing use belongs to authored last cap"
    );
    assert_eq!(
        audit.issues[0].description.as_bytes(),
        b"CoEdgeDef has no Curve2D representation"
    );
}

#[test]
fn cumulative_v2_actual_prism_retains_all_six_authored_trims() {
    use rustycad_tkbrep::brep_graph_inc::populate::PCurveImportBehavior::CumulativeOccurrenceV2;
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let graph = imported(
        &receipt.diagnostic.as_ref().unwrap().body,
        CumulativeOccurrenceV2,
    );
    assert_eq!(graph.topo().faces().nb(), 3);
    assert_eq!(graph.topo().wires().nb(), 3);
    assert_eq!(graph.topo().edges().nb(), 3);
    assert_eq!(graph.topo().co_edges().nb(), 6);
    let audit = validate::perform_with_mode(&graph, validate::Mode::Audit);
    assert!(
        audit.issues.is_empty(),
        "actual prism graph issues: {:?}",
        audit.issues
    );
}
#[test]
fn cumulative_v2_actual_last_cap_alone_retains_stored_pcurve() {
    use rustycad_tkbrep::brep_graph_inc::populate::PCurveImportBehavior::CumulativeOccurrenceV2;
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let graph = imported(last_cap(&receipt), CumulativeOccurrenceV2);
    assert_eq!(graph.topo().co_edges().nb(), 1);
    let audit = validate::perform_with_mode(&graph, validate::Mode::Audit);
    assert!(
        audit.issues.is_empty(),
        "last cap graph issues: {:?}",
        audit.issues
    );
}
