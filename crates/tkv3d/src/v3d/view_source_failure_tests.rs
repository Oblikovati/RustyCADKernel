//! Typed native failures preserve exact entry state and stop later operations of void callbacks.
use super::view_source_fixture::*;
use super::*;
use rustycad_tkernel::standard::{Failure, Handle, catch_native_failure};
use rustycad_tkservice::{graphic3d::Graphic3dClipPlane, null_driver::NullGraphicDriver};
#[test]
fn derived_void_failures_replay_native_state_order_and_intrusive_counts() {
    let rows: Vec<String> = (1..=4).flat_map(capture_view_failure).collect();
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/v3d-view-failures-native.txt")
    );
}
fn capture_view_failure(operation: i32) -> Vec<String> {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let trace = fresh_callback_trace();
    let view = mapped_probe_view(&viewer, 1, &trace);
    let plane = Handle::new(Graphic3dClipPlane::default());
    view.add_clip_plane(&plane);
    view.invalidate_immediate();
    trace.rows.write().clear();
    *trace.failing_operation.write() = Some(view_failure_operation(operation));
    let failure = dispatch_view_failure(&viewer, &view, &plane, operation).unwrap_err();
    let mut rows = vec![format!("failure {operation} {}", failure.message())];
    rows.push(view_failure_state_row(operation, &view, &plane));
    rows.extend(trace.rows.read().iter().cloned());
    rows
}
fn view_failure_operation(operation: i32) -> &'static str {
    match operation {
        1 => "redraw",
        2 => "immediate",
        3 => "add",
        4 => "remove",
        _ => unreachable!("received operation={operation}; expected 1..=4"),
    }
}
fn dispatch_view_failure(
    viewer: &V3dViewer,
    view: &Handle<dyn V3dViewSource>,
    plane: &Handle<Graphic3dClipPlane>,
    operation: i32,
) -> Result<(), Failure> {
    catch_native_failure(|| {
        match operation {
            1 => viewer.redraw(),
            2 => viewer.redraw_immediate(),
            3 => view.add_clip_plane(plane),
            4 => view.remove_clip_plane(plane),
            _ => unreachable!("received operation={operation}; expected 1..=4"),
        }
        Ok(())
    })
}

fn view_failure_state_row(
    operation: i32,
    view: &Handle<dyn V3dViewSource>,
    plane: &Handle<Graphic3dClipPlane>,
) -> String {
    // Query length before measuring counts: items() owns temporary handles unlike native Size().
    let plane_count = view.clip_planes().items().len();
    format!(
        "state {operation} {} {plane_count} {} {}",
        view.is_invalidated_immediate() as u8,
        view.ref_count(),
        plane.ref_count()
    )
}
