//! Actual retained scenes and stopped named clocks replay native navigation frames.
use super::view_controller_camera_fixture::append_primitive_camera as append_navigation_camera;
use super::view_controller_keyboard_fixture::{FrozenNavigationClock, frame, prepare};
use super::view_controller_keyboard_fixture::{component_values, walk_queries};
use super::view_controller_navigation_xr_fixture::NavigationOpenSession;
use super::view_controller_transfer_fixture::{queue_snapshot, seed_queue};
use super::*;
use crate::v3d::{V3dView, V3dViewer};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
use rustycad_tkservice::{aspect::*, graphic3d::*, null_driver::NullGraphicDriver};
use std::sync::Arc;
pub(super) const MOTION_KEYS: [AspectVKeyBasic; 12] = [
    AspectVKeyBasic::NavForward,
    AspectVKeyBasic::NavBackward,
    AspectVKeyBasic::NavSlideLeft,
    AspectVKeyBasic::NavSlideRight,
    AspectVKeyBasic::NavSlideUp,
    AspectVKeyBasic::NavSlideDown,
    AspectVKeyBasic::NavLookLeft,
    AspectVKeyBasic::NavLookRight,
    AspectVKeyBasic::NavLookUp,
    AspectVKeyBasic::NavLookDown,
    AspectVKeyBasic::NavRollCCW,
    AspectVKeyBasic::NavRollCW,
];
const MOTION_MODES: [AisNavigationMode; 3] = [
    AisNavigationMode::Orbit,
    AisNavigationMode::FirstPersonFlight,
    AisNavigationMode::FirstPersonWalk,
];
pub(super) struct NavigationScene {
    _viewer: Handle<V3dViewer>,
    _context: Handle<AisInteractiveContext>,
    pub(super) view: Handle<V3dView>,
    pub(super) controller: AisViewController,
    clock: Arc<FrozenNavigationClock>,
}
pub(super) fn navigation_scene(index: usize) -> NavigationScene {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    configure_navigation_view(&view, index);
    let context = AisInteractiveContext::new(&viewer).unwrap();
    display_navigation_cloud(&context, index);
    setup_navigation_xr(&view, index);
    let (mut controller, clock) = prepare(1.0, 2.0);
    seed_navigation_controller(&mut controller, index);
    controller.view_animation().set_view(&view);
    controller.view_animation().start(false).unwrap();
    NavigationScene {
        _viewer: viewer,
        _context: context,
        view,
        controller,
        clock,
    }
}
fn configure_navigation_view(view: &V3dView, index: usize) {
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(640, 480);
    view.set_window(&window.cast(), 0).unwrap();
    view.set_immediate_update(false);
    configure_navigation_camera(&view.camera(), index);
    view.view()
        .set_unit_factor(if (index / 360).is_multiple_of(2) {
            2.0
        } else {
            0.001
        })
        .unwrap();
}
fn display_navigation_cloud(context: &AisInteractiveContext, index: usize) {
    if (index / 45).is_multiple_of(2) {
        return;
    }
    let points = Handle::new(Graphic3dArrayOfPoints::new(2, 0).unwrap());
    points.add_vertex_f64(-2.0, -3.0, -4.0).unwrap();
    points.add_vertex_f64(1.0, 5.0, 7.0).unwrap();
    let object = AisPointCloud::new().unwrap();
    object.set_points(&points).unwrap();
    context.display(&object.cast(), false).unwrap();
}
fn setup_navigation_xr(view: &V3dView, index: usize) {
    if index < 1440 {
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
fn seed_navigation_controller(c: &mut AisViewController, index: usize) {
    c.set_navigation_mode(MOTION_MODES[index % 3]);
    seed_queue(&mut c.ui_input, index as i32, 0);
    seed_queue(&mut c.render_input, index as i32, 1);
    let policy = (index / 180) % 8;
    c.render_input.orbit_rotation.to_start = policy == 1;
    c.render_input.orbit_rotation.to_rotate = policy == 2;
    c.set_allow_panning(policy != 3);
    c.set_allow_rotation(policy != 4);
    c.set_allow_zooming(policy != 5);
    c.set_lock_orbit_z_up(policy != 6);
    c.set_invert_pitch(policy == 7);
    c.set_walk_speed_absolute(1.75);
    c.set_walk_speed_relative(0.125);
    seed_navigation_keys(c, index);
}
fn seed_navigation_keys(c: &mut AisViewController, index: usize) {
    let key = (index / 3) % 15;
    for item in active_navigation_keys(key) {
        c.key_down(item as AspectVKey, 1.25, 0.25).unwrap();
    }
    if key == 13 {
        c.key_down(AspectVKeyBasic::NavJump as AspectVKey, 1.25, 1.0)
            .unwrap();
    }
    if key == 14 {
        c.set_thrust_speed(0.5);
    }
    if !(index / 90).is_multiple_of(2) {
        c.key_down(AspectVKeyBasic::Shift as AspectVKey, 1.25, 1.0)
            .unwrap();
    }
}
fn active_navigation_keys(key: usize) -> Vec<AspectVKeyBasic> {
    if key < 12 {
        return vec![MOTION_KEYS[key]];
    }
    if key == 12 {
        return [0, 2, 4, 6, 8, 11].map(|rank| MOTION_KEYS[rank]).to_vec();
    }
    Vec::new()
}
pub(super) fn apply_navigation_frame(
    scene: &mut NavigationScene,
    index: usize,
    step: usize,
) -> AisWalkDelta {
    prepare_navigation_frame(scene, step);
    release_navigation_keys(&mut scene.controller, index, step);
    scene
        .controller
        .handle_navigation_keys(&Handle::default(), &scene.view)
        .unwrap()
}
pub(super) fn navigation_snapshot(scene: &NavigationScene, walk: &AisWalkDelta) -> Vec<f64> {
    let mut values = navigation_walk_values(walk);
    let c = &scene.controller;
    values.extend(
        [
            scene.view.is_invalidated(),
            c.view_animation().is_stopped(),
            c.view_animation().target_view().is_null(),
        ]
        .map(f64::from),
    );
    append_navigation_camera(&mut values, &scene.view.camera());
    let view = scene.view.get().unwrap().view();
    append_navigation_optional_camera(&mut values, &view.base_xr_camera());
    append_navigation_optional_camera(&mut values, &view.posed_xr_camera());
    append_navigation_book(&mut values, c);
    values.extend(queue_snapshot(&c.ui_input));
    values.extend(queue_snapshot(&c.render_input));
    values
}

fn navigation_walk_values(walk: &AisWalkDelta) -> Vec<f64> {
    let mut values = component_values(walk);
    values.extend(walk_queries(walk));
    values
}
fn append_navigation_optional_camera(values: &mut Vec<f64>, camera: &Handle<Graphic3dCamera>) {
    values.push(f64::from(!camera.is_null()));
    if let Some(camera) = camera.get() {
        append_navigation_camera(values, camera);
    } else {
        values.extend([0.0; 21]);
    }
}
fn append_navigation_book(values: &mut Vec<f64>, c: &AisViewController) {
    values.extend([
        f64::from(c.to_ask_next_frame()),
        f64::from(c.navigation_state.has_thrust),
        c.navigation_state.last_events_time,
        c.event_time(),
        f64::from(c.keys().modifiers()),
    ]);
    append_navigation_key_flags(values, c);
}
fn append_navigation_key_flags(values: &mut Vec<f64>, c: &AisViewController) {
    for key in MOTION_KEYS.into_iter().chain([
        AspectVKeyBasic::Shift,
        AspectVKeyBasic::NavJump,
        AspectVKeyBasic::NavCrouch,
    ]) {
        values.extend(
            [
                c.keys().is_key_down(key as AspectVKey).unwrap(),
                c.keys().is_free_key(key as AspectVKey).unwrap(),
            ]
            .map(f64::from),
        );
    }
}

fn configure_navigation_camera(camera: &Graphic3dCamera, index: usize) {
    camera
        .set_eye_and_center(
            &Point3::new(20.0, 30.0, 100.0),
            &Point3::new(1.0, -2.0, 3.0),
        )
        .unwrap();
    camera.set_up(&Direction3::new(0.0, 1.0, 0.2).unwrap());
    camera.set_scale(200.0);
    camera.set_aspect(4.0 / 3.0);
    camera.set_z_focus(Graphic3dCameraFocusType::Relative, 0.25);
    camera.set_projection_type(if (index / 90).is_multiple_of(2) {
        Graphic3dCameraProjection::Orthographic
    } else {
        Graphic3dCameraProjection::Perspective
    });
}
fn release_navigation_keys(c: &mut AisViewController, index: usize, step: usize) {
    if step != 1 {
        return;
    }
    for key in active_navigation_keys((index / 3) % 15) {
        c.key_up(key as AspectVKey, 2.25).unwrap();
    }
}

pub(super) fn prepare_navigation_frame(scene: &mut NavigationScene, step: usize) {
    scene.view.get().unwrap().view().redraw();
    scene.controller.set_ask_next_frame(false);
    frame(
        &mut scene.controller,
        &scene.clock,
        1.0 + step as f64,
        2.0 + step as f64,
    );
}
