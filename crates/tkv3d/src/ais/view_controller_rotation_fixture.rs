//! Actual native cameras, open headset I/O, shared start state and complete request buffers.
use super::view_controller_camera_fixture::{
    PrimitiveScene, append_primitive_camera, primitive_scene,
};
use super::view_controller_navigation_xr_fixture::NavigationOpenSession;
use super::view_controller_transfer_fixture::{queue_snapshot, seed_queue};
use super::*;
use crate::v3d::V3dView;
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
use rustycad_tkservice::{aspect::*, graphic3d::*};
const ROTATION_PROJECTIONS: [Graphic3dCameraProjection; 5] = [
    Graphic3dCameraProjection::Orthographic,
    Graphic3dCameraProjection::Perspective,
    Graphic3dCameraProjection::Stereo,
    Graphic3dCameraProjection::MonoLeftEye,
    Graphic3dCameraProjection::MonoRightEye,
];
const ROTATION_PIXELS: [Vec2<f64>; 12] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(80.0, 0.0),
    Vec2::new(0.0, 60.0),
    Vec2::new(80.0, 60.0),
    Vec2::new(-80.0, -60.0),
    Vec2::new(10000.0, -10000.0),
    Vec2::new(0.0, 1000.0),
    Vec2::new(0.0, -1000.0),
    Vec2::new(1e-14, 1e-14),
    Vec2::new(1e-9, 0.0),
    Vec2::new(0.0, 1e-9),
    Vec2::new(320.0, 240.0),
];
const ROTATION_EXTRAS: [[f64; 3]; 9] = [
    [0.0, 0.0, 0.0],
    [0.1, 0.0, 0.21],
    [0.0, 0.1, 0.21],
    [-0.2, -0.3, -0.14],
    [0.2, 0.4, 0.42],
    [0.0, 10.0, 0.21],
    [0.0, -10.0, 0.21],
    [1e-14, 1e-14, 0.21],
    [1e-9, 0.0, 0.21],
];
pub(super) fn rotation_local(index: usize) -> usize {
    if index < 1920 {
        return index;
    }
    index - 1920
}
pub(super) fn rotation_motion(index: usize) -> usize {
    let motion_count = if index < 1920 { 12 } else { 9 };
    (rotation_local(index) / 20) % motion_count
}
pub(super) fn rotation_options(index: usize) -> usize {
    let option_stride = if index < 1920 { 240 } else { 180 };
    rotation_local(index) / option_stride
}
pub(super) fn rotation_scene(index: usize) -> PrimitiveScene {
    let mut scene = primitive_scene(0);
    configure_rotation_camera(&scene.view, index);
    configure_rotation_xr(&scene.view, index);
    scene.controller = AisViewController::new().unwrap();
    seed_rotation_requests(&mut scene.controller, index);
    seed_retained_rotation_state(&mut scene.controller, &scene.view.camera());
    scene.controller.view_animation().set_view(&scene.view);
    scene.controller.view_animation().start(false).unwrap();
    scene
}
fn configure_rotation_camera(view: &V3dView, index: usize) {
    let camera = view.camera();
    camera
        .set_eye_and_center(
            &Point3::new(20.0, 30.0, 100.0),
            &Point3::new(1.0, -2.0, 3.0),
        )
        .unwrap();
    camera.set_up(&Direction3::new(0.0, 1.0, 0.2).unwrap());
    camera.set_projection_type(ROTATION_PROJECTIONS[rotation_local(index) % 5]);
}
fn configure_rotation_xr(view: &V3dView, index: usize) {
    if rotation_options(index) % 4 < 2 {
        return;
    }
    let session = Handle::new(NavigationOpenSession::default());
    session
        .xr_session_base()
        .fields
        .write()
        .head_pose
        .set_translation(Vector3::new(1.25, -2.5, 0.75));
    let cview = view.view();
    cview.set_xr_session(&session.cast());
    cview.set_base_xr_camera(&Handle::new(view.camera().copy_members()));
    cview.set_posed_xr_camera(&Handle::new(view.camera().copy_members()));
    cview.setup_xr_posed_camera();
}
fn seed_rotation_requests(c: &mut AisViewController, index: usize) {
    seed_queue(&mut c.ui_input, index as i32, 0);
    seed_queue(&mut c.render_input, index as i32, 1);
    let flags = (rotation_local(index) / 5) % 4;
    seed_rotation_buffer(
        &mut c.render_input,
        flags,
        ROTATION_PIXELS[rotation_motion(index)],
    );
    c.set_allow_rotation(rotation_options(index) < 4);
    c.set_ask_next_frame(false);
}
fn seed_rotation_buffer(buffer: &mut AisViewInputBuffer, flags: usize, delta: Vec2<f64>) {
    let start = Vec2::new(137.0, 91.0);
    let orbit = &mut buffer.orbit_rotation;
    orbit.to_start = flags & 1 != 0;
    orbit.to_rotate = flags & 2 != 0;
    orbit.point_start = start;
    orbit.point_to = start + delta;
    let view = &mut buffer.view_rotation;
    view.to_start = flags & 1 != 0;
    view.to_rotate = flags & 2 != 0;
    view.point_start = start;
    view.point_to = start + delta;
}
fn seed_retained_rotation_state(c: &mut AisViewController, camera: &Graphic3dCamera) {
    let state = &mut c.rotation_state;
    state.pivot = Point3::new(7.0, -3.0, 20.0);
    state.start_up = camera.up();
    state.start_direction = camera.direction();
    state.start_eye = camera.eye();
    state.start_center = camera.center();
    state.current_roll = 0.21;
}
pub(super) fn prepare_rotation_frame(scene: &mut PrimitiveScene, step: usize) {
    scene.view.get().unwrap().view().redraw();
    if step < 2 {
        return;
    }
    let b = &mut scene.controller.render_input;
    b.orbit_rotation.to_start = false;
    b.view_rotation.to_start = false;
    b.orbit_rotation.point_to += Vec2::new(13.0, -17.0);
    b.view_rotation.point_to += Vec2::new(13.0, -17.0);
}
pub(super) fn apply_rotation(index: usize, scene: &mut PrimitiveScene) -> Result<(), Failure> {
    if index < 1920 {
        return scene.controller.handle_orbit_rotation(
            &scene.view,
            &Point3::new(9.0, -5.0, 12.0),
            rotation_options(index).is_multiple_of(2),
        );
    }
    let [yaw, pitch, roll] = ROTATION_EXTRAS[rotation_motion(index)];
    scene.controller.handle_view_rotation(
        &scene.view,
        yaw,
        pitch,
        roll,
        !rotation_options(index).is_multiple_of(2),
    )
}
pub(super) fn rotation_snapshot(scene: &PrimitiveScene, failed: bool) -> Vec<f64> {
    let c = &scene.controller;
    let mut values = [
        failed,
        scene.view.is_invalidated(),
        c.view_animation().is_stopped(),
        c.view_animation().target_view().is_null(),
    ]
    .map(f64::from)
    .to_vec();
    append_primitive_camera(&mut values, &scene.view.camera());
    let view = scene.view.get().unwrap().view();
    append_rotation_optional_camera(&mut values, &view.base_xr_camera());
    append_rotation_optional_camera(&mut values, &view.posed_xr_camera());
    append_rotation_state(&mut values, c);
    values.extend(queue_snapshot(&c.ui_input));
    values.extend(queue_snapshot(&c.render_input));
    values
}
fn append_rotation_optional_camera(values: &mut Vec<f64>, camera: &Handle<Graphic3dCamera>) {
    values.push(f64::from(!camera.is_null()));
    if let Some(camera) = camera.get() {
        append_primitive_camera(values, camera);
    } else {
        values.extend([0.0; 21]);
    }
}
fn append_rotation_state(values: &mut Vec<f64>, c: &AisViewController) {
    let state = &c.rotation_state;
    for p in [state.pivot, state.start_eye, state.start_center] {
        values.extend([p.x(), p.y(), p.z()]);
    }
    for d in [state.start_up, state.start_direction] {
        values.extend([d.x(), d.y(), d.z()]);
    }
    values.extend([state.current_roll, f64::from(c.to_ask_next_frame())]);
}
