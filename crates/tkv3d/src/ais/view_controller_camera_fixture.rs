//! Actual cameras, queues and XR pose state mirror the pinned CPU native oracle.
use super::{
    AisAnimationMembers as _, AisAnimationSource as _, AisViewController,
    view_controller_transfer_fixture::*,
};
use crate::v3d::{V3dView, V3dViewer};
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkservice::{aspect::*, graphic3d::*, null_driver::NullGraphicDriver};
const PRIMITIVE_PROJECTIONS: [Graphic3dCameraProjection; 5] = [
    Graphic3dCameraProjection::Orthographic,
    Graphic3dCameraProjection::Perspective,
    Graphic3dCameraProjection::Stereo,
    Graphic3dCameraProjection::MonoLeftEye,
    Graphic3dCameraProjection::MonoRightEye,
];
pub(super) struct PrimitiveScene {
    _viewer: Handle<V3dViewer>,
    pub(super) view: Handle<V3dView>,
    pub(super) controller: AisViewController,
}
pub(super) fn primitive_scene(index: usize) -> PrimitiveScene {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    configure_primitive_view(&view, index);
    configure_primitive_xr(&view, index);
    let mut controller = AisViewController::new();
    configure_primitive_controller(&mut controller, index);
    controller.view_animation().set_view(&view);
    controller.view_animation().start(false).unwrap();
    PrimitiveScene {
        _viewer: viewer,
        view,
        controller,
    }
}
fn configure_primitive_view(view: &V3dView, index: usize) {
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(640, 480);
    view.set_window(&window.cast(), 0).unwrap();
    view.set_immediate_update(false);
    let camera = view.camera();
    camera
        .set_eye_and_center(&Point3::new(0.0, 0.0, 200.0), &Point3::default())
        .unwrap();
    camera.set_scale(200.0);
    camera.set_aspect(4.0 / 3.0);
    configure_primitive_projection(&camera, index);
}
fn configure_primitive_projection(camera: &Graphic3dCamera, index: usize) {
    camera.set_projection_type(PRIMITIVE_PROJECTIONS[index % 5]);
    let focus = [0.25, 1.0, 1.95][(index / 400) % 3];
    let focus_type = if (index / 400) % 3 == 1 {
        Graphic3dCameraFocusType::Absolute
    } else {
        Graphic3dCameraFocusType::Relative
    };
    camera.set_z_focus(focus_type, focus);
}
fn configure_primitive_xr(view: &V3dView, index: usize) {
    if (index / 1200).is_multiple_of(2) {
        return;
    }
    let cview = view.view();
    let session = Handle::new(AspectOpenVRSession::default());
    session
        .xr_session_base()
        .fields
        .write()
        .head_pose
        .set_translation(Vector3::new(1.25, -2.5, 0.75));
    cview.set_xr_session(&session.cast());
    cview.set_base_xr_camera(&Handle::new(view.camera().copy_members()));
    cview.set_posed_xr_camera(&Handle::new(view.camera().copy_members()));
    cview.setup_xr_posed_camera();
}
fn configure_primitive_controller(c: &mut AisViewController, index: usize) {
    seed_queue(&mut c.ui_input, index as i32, 0);
    seed_queue(&mut c.render_input, index as i32, 1);
    c.render_input.panning.to_pan = (index / 25).is_multiple_of(2);
    c.render_input.panning.delta = Vec2::new(17, -9);
    c.render_input.z_rotate.to_rotate = (index / 50).is_multiple_of(2);
    c.render_input.z_rotate.angle = (((index / 5) % 5) as i32 - 2) as f64 * 0.07;
    if !(index / 50).is_multiple_of(2) {
        c.set_panning_anchor_point(&Point3::new(7.0, -3.0, 20.0));
    }
    configure_primitive_gates(c, index);
}
fn configure_primitive_gates(c: &mut AisViewController, index: usize) {
    let allow = (index / 100).is_multiple_of(2);
    c.set_allow_panning(allow);
    c.set_allow_rotation(allow);
    c.set_allow_zooming(allow);
    c.set_allow_z_focus(allow);
    c.set_min_zoom_distance(if (index / 200).is_multiple_of(2) {
        1.0
    } else {
        250.0
    });
}
pub(super) fn primitive_scroll(index: usize) -> AspectScrollDelta {
    let point = if (index / 25).is_multiple_of(2) {
        Vec2::new(-1, -1)
    } else {
        Vec2::new(137, 91)
    };
    let delta = [-100.0, 0.0, 0.25, 40.0, 500.0][(index / 5) % 5];
    AspectScrollDelta::new(point, delta, 256)
}
pub(super) fn apply_primitive(index: usize, scene: &mut PrimitiveScene) {
    let result = apply_primitive_result(index, scene);
    result.expect("native camera primitive succeeded");
}
pub(super) fn apply_primitive_result(
    index: usize,
    scene: &mut PrimitiveScene,
) -> Result<(), Failure> {
    let scroll = primitive_scroll(index);
    let hit = (!(index / 50).is_multiple_of(2)).then(|| Point3::new(7.0, -3.0, 20.0));
    match index / 2400 {
        0 => scene.controller.handle_panning(&scene.view),
        1 => scene.controller.handle_z_rotate(&scene.view),
        2 => scene
            .controller
            .handle_zoom(&scene.view, &scroll, hit.as_ref()),
        3 => scene.controller.handle_z_focus_scroll(&scene.view, &scroll),
        operation => panic!("camera operation={operation}; expected 0..=3"),
    }
}
pub(super) fn primitive_snapshot(scene: &PrimitiveScene) -> Vec<f64> {
    let c = &scene.controller;
    let mut values = vec![
        f64::from(scene.view.is_invalidated()),
        f64::from(c.view_animation().is_stopped()),
        f64::from(c.view_animation().target_view().is_null()),
        f64::from(c.has_panning_anchor_point()),
    ];
    let anchor = c.panning_anchor_point();
    values.extend([anchor.x(), anchor.y(), anchor.z()]);
    append_primitive_camera(&mut values, &scene.view.camera());
    append_primitive_xr(&mut values, &scene.view);
    values.extend(queue_snapshot(&c.ui_input));
    values.extend(queue_snapshot(&c.render_input));
    values
}
fn append_primitive_xr(values: &mut Vec<f64>, view: &V3dView) {
    let camera = view.view().base_xr_camera();
    values.push(f64::from(!camera.is_null()));
    if let Some(camera) = camera.get() {
        append_primitive_camera(values, camera);
    } else {
        values.extend([0.0; 21]);
    }
}
fn append_primitive_camera(values: &mut Vec<f64>, c: &Graphic3dCamera) {
    for p in [c.center(), c.eye()] {
        values.extend([p.x(), p.y(), p.z()]);
    }
    for d in [c.up(), c.direction()] {
        values.extend([d.x(), d.y(), d.z()]);
    }
    values.extend([
        c.scale(),
        c.distance(),
        c.z_focus(),
        f64::from(c.z_focus_type() as i32),
        c.aspect(),
        f64::from(c.projection_type() as i32),
        c.z_near(),
        c.z_far(),
        c.fov_y(),
    ]);
}
