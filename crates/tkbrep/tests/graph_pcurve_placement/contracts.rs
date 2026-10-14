// SPDX-License-Identifier: GPL-2.0-only
//! Bound import contracts: typed failure, input preservation, deterministic operation-local selection.
use super::{
    baseline_row,
    fixtures::{imported_v2, single_use_face, translated},
};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_graph::BRepGraph,
    brep_graph_inc::populate::{self, PCurveImportBehavior},
    topods::{Face, Shape, TFace},
};
use rustycad_tkernel::standard::Handle;
#[test]
fn cumulative_v2_preserves_original_typed_failure_and_source() {
    let mut face = Face::default();
    face.set_tshape(&Handle::new(TFace::new()).into_cast());
    let expected = BRepTool::surface_with_location(&face).unwrap_err();
    let before = face.tshape().clone();
    let mut graph = BRepGraph::new();
    let actual = populate::perform_with_behavior(
        &mut graph,
        &face,
        false,
        &populate::Options,
        PCurveImportBehavior::CumulativeOccurrenceV2,
    )
    .unwrap_err();
    assert_eq!(actual.exception_type(), expected.exception_type());
    assert_eq!(actual.message(), expected.message());
    assert_eq!(face.tshape(), &before);
    assert_eq!(face.nb_children(), 0);
}
#[test]
fn cumulative_v2_null_append_does_not_clear_existing_topology() {
    let mut graph = imported_v2(&translated(&single_use_face()), false);
    let before = baseline_row("retained", &graph);
    let status = populate::append_with_behavior(
        &mut graph,
        &Shape::default(),
        false,
        &populate::Options,
        PCurveImportBehavior::CumulativeOccurrenceV2,
    )
    .unwrap();
    assert_eq!(status, populate::BuildStatus::Failed);
    assert_eq!(baseline_row("retained", &graph), before);
}
fn owned_snapshot() -> String {
    let source = translated(&single_use_face());
    let location = source.location().clone();
    let topology = source.tshape().clone();
    let before = super::source_snapshot::capture_source_values(&source);
    let graph = imported_v2(&source, false);
    assert_eq!(
        super::source_snapshot::capture_source_values(&source),
        before
    );
    assert_eq!(source.location(), &location);
    assert_eq!(source.tshape(), &topology);
    assert_eq!(source.nb_children(), 1);
    baseline_row("v2", &graph)
}
#[test]
fn cumulative_v2_thread_replay_and_policy_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PCurveImportBehavior>();
    let expected = owned_snapshot();
    let joins: Vec<_> = (0..8).map(|_| std::thread::spawn(owned_snapshot)).collect();
    for join in joins {
        assert_eq!(join.join().unwrap(), expected);
    }
}
#[test]
fn cumulative_v2_owned_process_snapshot() {
    let snapshot = owned_snapshot();
    assert_eq!(snapshot, "v2: stored=true range=0,1 reversed=false\n");
    println!("{{\"pcurve_import_behavior\":2,\"stored\":true,\"range\":[0,1],\"reversed\":false}}");
}
#[test]
fn cumulative_v2_append_reuses_legacy_definitions_until_explicit_rebuild() {
    let shape = translated(&single_use_face());
    let mut graph = super::fixtures::imported(&shape, false);
    let before = baseline_row("legacy", &graph);
    let behavior = PCurveImportBehavior::CumulativeOccurrenceV2;
    populate::append_with_behavior(&mut graph, &shape, false, &populate::Options, behavior)
        .unwrap();
    assert_eq!(baseline_row("legacy", &graph), before);
    populate::perform_with_behavior(&mut graph, &shape, false, &populate::Options, behavior)
        .unwrap();
    assert_eq!(
        baseline_row("rebuilt", &graph),
        "rebuilt: stored=true range=0,1 reversed=false\n"
    );
}
