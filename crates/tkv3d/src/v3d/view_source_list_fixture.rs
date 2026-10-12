//! Valid native list mutation keeps the current node and original strong view owners alive.
use super::view_source_fixture::*;
use super::*;
use rustycad_tkernel::standard::{Handle, HandleReference};

pub(super) fn apply_probe_list_mutation(view: &NativeStyleViewProbe) {
    let viewer = view.base.viewer();
    let policy = view.trace.list_mutation.read().policy;
    match policy {
        1 => mutate_other_probe_node(&viewer, &view.trace),
        2 => replace_current_probe_node(&viewer, &view.trace),
        _ => {}
    }
}
fn mutate_other_probe_node(viewer: &V3dViewer, trace: &ViewCallbackTrace) {
    let commands = trace.list_mutation.read();
    viewer.set_view_off_one(&*commands.removed.observe());
    viewer
        .set_view_on_one(&*commands.appended.observe())
        .unwrap();
}
fn replace_current_probe_node(viewer: &V3dViewer, trace: &ViewCallbackTrace) {
    let commands = trace.list_mutation.read();
    let mut fields = viewer.state.write();
    // Native const_cast edits a nonconst owned list's value, preserving the current node itself.
    *fields.active_views.first_mut().unwrap() = commands.replacement.observe().clone();
}
pub(super) fn probe_live_active_iteration(viewer: &V3dViewer) -> Vec<String> {
    let mut cursor = viewer.active_view_iterator();
    let mut rows = Vec::new();
    while cursor.more() {
        let alias = cursor.value_ref();
        let identity = alias.reference_address();
        let before = alias.observe();
        before.redraw();
        let after = alias.observe();
        rows.push(format!(
            "node {} {} {} {}",
            probe_view_identity(&before),
            probe_view_identity(&after),
            (identity == alias.reference_address()) as u8,
            after.ref_count()
        ));
        cursor.next();
    }
    rows
}
pub(super) fn probe_view_identity(view: &Handle<dyn V3dViewSource>) -> i32 {
    let projected = view.view::<NativeStyleViewProbe>();
    projected.get().unwrap().identity
}
