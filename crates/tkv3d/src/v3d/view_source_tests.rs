//! Genuine subclass dispatch retains native ownership and live caller references.
use super::view_source_fixture::*;
use super::*;
use rustycad_tkernel::standard::{Handle, HandleFieldRef, RuntimeTyped};
use rustycad_tkservice::{graphic3d::Graphic3dClipPlane, null_driver::NullGraphicDriver};

#[test]
fn native_view_interface_and_concrete_share_descriptor_and_owned_base() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let trace = fresh_callback_trace();
    let view = mapped_probe_view(&viewer, 1, &trace);
    assert_eq!(view.ref_count(), 3);
    assert!(std::ptr::eq(
        V3dView::static_type(),
        <dyn V3dViewSource>::static_type()
    ));
    assert!(view.view::<NativeStyleViewProbe>().get().is_some());
    assert!(view.view::<V3dView>().get().is_some());
    assert!(std::ptr::eq(
        view.get().unwrap().view_base(),
        view.view::<V3dView>().get().unwrap()
    ));
}
#[test]
fn viewer_redraw_dispatches_derived_override_without_snapshot_reference_counts() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let trace = fresh_callback_trace();
    let view = mapped_probe_view(&viewer, 1, &trace);
    trace.rows.write().clear();
    viewer.redraw();
    viewer.redraw_immediate();
    assert_eq!(*trace.rows.read(), ["1 redraw 3", "1 immediate 3"]);
    assert_eq!(view.ref_count(), 3);
}
#[test]
fn clip_override_rereads_actual_caller_slot_before_inherited_insert() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let trace = fresh_callback_trace();
    let view = mapped_probe_view(&viewer, 1, &trace);
    let original = Handle::new(Graphic3dClipPlane::default());
    *trace.plane_slot.write() = original.clone();
    *trace.replace_on_add.write() = true;
    let argument = HandleFieldRef::new(&trace.plane_slot, |slot| slot);
    view.add_clip_plane(&argument);
    let expected_identity = trace.replacement.object_address();
    assert_eq!(argument.object_address(), expected_identity);
    let inserted = view.clip_planes().items();
    assert_eq!(inserted, std::slice::from_ref(&trace.replacement));
    drop(inserted);
    view.remove_clip_plane(&argument);
    assert!(view.clip_planes().items().is_empty());
    assert_eq!(original.ref_count(), 1);
    assert_eq!(trace.replacement.ref_count(), 2);
}

#[test]
fn derived_callbacks_and_plane_counts_replay_pinned_native_rows() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let trace = fresh_callback_trace();
    let view = mapped_probe_view(&viewer, 1, &trace);
    trace.rows.write().clear();
    viewer.redraw();
    viewer.redraw_immediate();
    let mut rows = native_plane_callback_rows(&view, &trace);
    rows.extend(trace.rows.read().iter().cloned());
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/v3d-view-source-native.txt")
    );
}
fn native_plane_callback_rows(
    view: &Handle<dyn V3dViewSource>,
    trace: &std::sync::Arc<ViewCallbackTrace>,
) -> Vec<String> {
    let first = Handle::new(Graphic3dClipPlane::default());
    *trace.plane_slot.write() = first.clone();
    *trace.replace_on_add.write() = true;
    let slot = HandleFieldRef::new(&trace.plane_slot, |value| value);
    view.add_clip_plane(&slot);
    let identity = (slot.object_address() == trace.replacement.object_address()) as u8;
    let stored = (view.clip_planes().items()[0] == trace.replacement) as u8;
    let mut rows = vec![format!(
        "slot {identity} {stored} {} {}",
        first.ref_count(),
        trace.replacement.ref_count()
    )];
    view.remove_clip_plane(&slot);
    rows.push(final_plane_callback_row(view, &first, trace));
    rows
}

fn final_plane_callback_row(
    view: &Handle<dyn V3dViewSource>,
    first: &Handle<Graphic3dClipPlane>,
    trace: &ViewCallbackTrace,
) -> String {
    let plane_count = view.clip_planes().items().len();
    format!(
        "final {plane_count} {} {} {}",
        first.ref_count(),
        trace.replacement.ref_count(),
        view.ref_count()
    )
}
