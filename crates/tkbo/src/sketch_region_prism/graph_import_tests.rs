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
fn imported(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    let options = Options {
        create_auto_product: false,
        track_added_nodes: true,
        ..Options::default()
    };
    graph
        .shapes_mut()
        .add_with_options(shape, &options)
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
    let graph = imported(&receipt.diagnostic.as_ref().unwrap().body);
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
