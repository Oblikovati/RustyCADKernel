//! Native virtual SetCamera argument aliases/counts and partial failures require no owning snapshots.
use super::{camera_argument_fixture::*, *};
use crate::v3d::*;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::{graphic3d::Graphic3dCamera, null_driver::NullGraphicDriver};
use std::panic::{AssertUnwindSafe, catch_unwind};
pub(super) struct CameraArgumentScene {
    _viewer: Handle<V3dViewer>,
    view: Handle<dyn V3dViewSource>,
    first: Handle<Graphic3dCamera>,
    pub(super) second:
        std::sync::Arc<rustycad_tkernel::standard::SharedState<Handle<Graphic3dCamera>>>,
    pub(super) volume: Handle<CameraArgumentFrustum>,
    manager: SelectMgrSelectingVolumeManager,
}
#[test]
fn native_selecting_volume_camera_argument_replacement_failure_and_retry() {
    let rows: Vec<String> = (0..3).flat_map(camera_argument_rows).collect();
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/selector-camera-argument-native.txt")
    );
}
fn camera_argument_rows(mode: i32) -> Vec<String> {
    let scene = camera_argument_scene(mode);
    (0..2)
        .map(|frame| camera_argument_row(mode, frame, &scene))
        .collect()
}
pub(super) fn camera_argument_scene(mode: i32) -> CameraArgumentScene {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    view.set_immediate_update(false);
    let first = view.camera();
    let second = std::sync::Arc::new(rustycad_tkernel::standard::SharedState::new(Handle::new(
        first.copy_members(),
    )));
    let volume = CameraArgumentFrustum::new(&view, &second, mode);
    let mut manager = SelectMgrSelectingVolumeManager::new();
    manager.init_selecting_volume(&volume.cast());
    CameraArgumentScene {
        _viewer: viewer,
        view,
        first,
        second,
        volume,
        manager,
    }
}
fn camera_argument_row(mode: i32, frame: i32, scene: &CameraArgumentScene) -> String {
    let failure = apply_camera_argument(scene);
    let mut values = vec![mode, frame, failure];
    values.extend(*scene.volume.records.read());
    values.extend(camera_argument_state(scene));
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}
fn camera_argument_state(scene: &CameraArgumentScene) -> [i32; 5] {
    // Native Camera returns a field reference; copied convenience getters would add temporary owners.
    let active = scene.manager.active_volume().observe();
    let camera = active.intersector_base().camera.read().observe();
    [
        i32::from(camera.object_address() == scene.second.read().object_address()),
        i32::from(camera.is_null()),
        scene.first.ref_count(),
        scene.second.read().ref_count(),
        scene.volume.ref_count(),
    ]
}
pub(super) fn apply_camera_argument(scene: &CameraArgumentScene) -> i32 {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        scene.manager.set_camera(scene.view.camera_ref().as_ref())
    }));
    camera_argument_failure(outcome)
}
fn camera_argument_failure(
    outcome: Result<Result<(), Failure>, Box<dyn std::any::Any + Send>>,
) -> i32 {
    match outcome {
        Ok(Ok(())) => 0,
        Ok(Err(failure)) => {
            assert!(failure.message().contains("fault=1"));
            1
        }
        Err(payload) => {
            assert_eq!(payload.downcast_ref::<u32>(), Some(&93));
            2
        }
    }
}
