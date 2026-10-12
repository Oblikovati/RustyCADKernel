//! Exact native counts distinguish const-camera reference operations from intentional owning-set overloads.
use super::{view_camera_alias_fixture::*, *};
use rustycad_tkernel::standard::{Handle, SharedState, catch_native_failure};
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
#[path = "cview_bounds_camera_callback_fixture.rs"]
mod fixture;
use fixture::*;
#[test]
fn native_cview_bounds_and_zoom_keep_virtual_selected_slot_across_window_callbacks() {
    let rows: Vec<String> = (0..18).flat_map(bounds_camera_callback_rows).collect();
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/cview-bounds-camera-callback-native.txt")
    );
}
fn bounds_camera_callback_rows(case: i32) -> Vec<String> {
    let selection = fresh_camera_alias_control();
    let driver = camera_alias_driver(&NullGraphicDriver::new_handle(), &selection);
    let viewer = V3dViewer::new(&driver).unwrap();
    let view = viewer.create_view().unwrap();
    *selection.alternate_selected.write() = case / 9 != 0;
    let original = view.camera();
    let replacement = Handle::new(original.copy_members());
    let control = bounds_camera_window_control(case % 3, &view, &replacement, &selection);
    let _window = configure_bounds_callback_window(&view, &control);
    (0..2)
        .map(|frame| {
            bounds_camera_callback_frame(case, frame, &view, &control, (&original, &replacement))
        })
        .collect()
}
fn bounds_camera_window_control(
    mode: i32,
    view: &Handle<dyn V3dViewSource>,
    replacement: &Handle<Graphic3dCamera>,
    selection: &Arc<CameraAliasControl>,
) -> Arc<BoundsCameraWindowControl> {
    Arc::new(BoundsCameraWindowControl {
        armed: AtomicBool::new(false),
        mode,
        view: view.downgrade(),
        replacement: replacement.downgrade(),
        selection: selection.clone(),
        records: SharedState::new([0; 3]),
    })
}
fn configure_bounds_callback_window(
    view: &Handle<dyn V3dViewSource>,
    control: &Arc<BoundsCameraWindowControl>,
) -> Handle<BoundsCameraCallbackWindow> {
    let window = bounds_camera_callback_window(control);
    view.set_window(&window.cast(), 0).unwrap();
    view.set_immediate_update(false);
    control.armed.store(true, Ordering::Relaxed);
    control
        .selection
        .bounds_getter_calls
        .store(0, Ordering::Relaxed);
    window
}
fn bounds_camera_callback_frame(
    case: i32,
    frame: i32,
    view: &Handle<dyn V3dViewSource>,
    control: &BoundsCameraWindowControl,
    cameras: (&Handle<Graphic3dCamera>, &Handle<Graphic3dCamera>),
) -> String {
    let result = catch_unwind(AssertUnwindSafe(|| {
        catch_native_failure(|| {
            run_bounds_camera_operation(case / 3 % 3, view.view_ref().observe().get().unwrap())
        })
    }));
    let (failure, value) = classify_bounds_camera_result(result);
    let mut row = vec![case, frame, failure, value];
    row.extend(bounds_camera_record_columns(control, cameras));
    row.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}
fn bounds_camera_record_columns(
    control: &BoundsCameraWindowControl,
    cameras: (&Handle<Graphic3dCamera>, &Handle<Graphic3dCamera>),
) -> [i32; 7] {
    let records = control.records.read();
    [
        records[0],
        records[1],
        records[2],
        cameras.0.ref_count(),
        cameras.1.ref_count(),
        control
            .selection
            .bounds_getter_calls
            .load(Ordering::Relaxed),
        i32::from(*control.selection.alternate_selected.read()),
    ]
}
fn run_bounds_camera_operation(
    method: i32,
    view: &dyn Graphic3dCView,
) -> Result<i32, rustycad_tkernel::standard::Failure> {
    match method {
        0 => Ok(i32::from(view.min_max_values(false)?.is_void())),
        1 => Ok(view.consider_zoom_persistence_objects()? as i32),
        _ => Ok(i32::from(view.min_max_values_of(&[], false)?.is_void())),
    }
}
fn classify_bounds_camera_result(
    result: Result<Result<i32, rustycad_tkernel::standard::Failure>, Box<dyn std::any::Any + Send>>,
) -> (i32, i32) {
    match result {
        Ok(Ok(value)) => (0, value),
        Ok(Err(failure)) => {
            assert!(failure.message().contains("bounds Size fault=1"));
            (1, -1)
        }
        Err(payload) => {
            assert_eq!(payload.downcast_ref::<u32>(), Some(&73));
            (2, -1)
        }
    }
}
