//! Exact native node/count replay distinguishes live iteration from cloned view snapshots.
use super::view_source_fixture::*;
use super::view_source_list_fixture::*;
use super::*;
use rustycad_tkernel::standard::{Handle, HandleReference};
use rustycad_tkservice::null_driver::NullGraphicDriver;

#[test]
fn valid_live_node_mutations_replay_pinned_native_callback_and_count_rows() {
    let rows: Vec<String> = [1, 2]
        .into_iter()
        .flat_map(capture_live_probe_case)
        .collect();
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/v3d-view-live-list-native.txt")
    );
}
fn capture_live_probe_case(policy: i32) -> Vec<String> {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let trace = fresh_callback_trace();
    let views: Vec<Handle<dyn V3dViewSource>> = (1..=4)
        .map(|id| mapped_probe_view(&viewer, id, &trace))
        .collect();
    viewer.set_view_off_one(&views[3]);
    arm_list_mutation(&trace, policy, &views);
    let mut rows = vec![format!("case {policy}")];
    rows.extend(probe_live_active_iteration(&viewer));
    rows.push(probe_view_count_row(&viewer, &views));
    rows
}
#[test]
fn list_and_deprecated_node_aliases_have_same_live_slot_identity_without_ownership() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let trace = fresh_callback_trace();
    let first = mapped_probe_view(&viewer, 1, &trace);
    let second = mapped_probe_view(&viewer, 2, &trace);
    viewer.init_active_views();
    let list = viewer.active_views_ref();
    let cursor = viewer.active_view_iterator();
    let direct = cursor.value_ref();
    let retained = viewer.active_view_ref();
    assert_eq!(direct.reference_address(), retained.reference_address());
    assert_eq!(first.ref_count(), 3);
    *viewer.state.write().active_views.first_mut().unwrap() = second.clone();
    assert_eq!(retained.object_address(), second.object_address());
    assert_eq!(list.len(), 2);
    assert_eq!((first.ref_count(), second.ref_count()), (2, 4));
}

fn arm_list_mutation(trace: &ViewCallbackTrace, policy: i32, views: &[Handle<dyn V3dViewSource>]) {
    *trace.list_mutation.write() = ProbeListMutation {
        policy,
        removed: views[1].downgrade(),
        appended: views[3].downgrade(),
        replacement: views[1].downgrade(),
    };
}
fn probe_view_count_row(viewer: &V3dViewer, views: &[Handle<dyn V3dViewSource>]) -> String {
    format!(
        "counts {} {} {} {} {}",
        viewer.active_views_ref().len(),
        views[0].ref_count(),
        views[1].ref_count(),
        views[2].ref_count(),
        views[3].ref_count()
    )
}

#[test]
fn defined_list_aliases_keep_field_and_node_identity_without_extra_owners() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    assert!(viewer.defined_views_ref().is_empty());
    let trace = fresh_callback_trace();
    let first = mapped_probe_view(&viewer, 1, &trace);
    let chain = viewer.defined_views_ref();
    let duplicate = viewer.defined_views_ref();
    assert_eq!(chain.reference_address(), duplicate.reference_address());
    assert_defined_node_aliases(&viewer, &first);
}
fn assert_defined_node_aliases(viewer: &V3dViewer, first: &Handle<dyn V3dViewSource>) {
    viewer.init_defined_views();
    let mut cursor = viewer.defined_view_iterator();
    assert!(cursor.node_address().is_some());
    assert_eq!(
        cursor.value_ref().reference_address(),
        viewer.defined_view_ref().reference_address()
    );
    assert_eq!(cursor.value_ref().object_address(), first.object_address());
    cursor.next();
    assert!(!cursor.more());
    assert!(cursor.node_address().is_none());
    assert_eq!(first.ref_count(), 3);
}
