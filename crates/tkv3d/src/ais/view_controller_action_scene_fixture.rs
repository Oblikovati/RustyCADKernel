//! Real retained views, points, anchors, grids and detection mirror native scenes.
use crate::v3d::V3dViewSource;
use super::view_controller_action_callback_fixture::ActionCallbackObject;
use super::view_controller_navigation_xr_fixture::NavigationOpenSession;
use super::*;
use crate::{prs_mgr::PrsMgrDisplayStatus, v3d::*};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkservice::{aspect::*, graphic3d::*, null_driver::NullGraphicDriver};
use std::sync::atomic::Ordering;
const ACTION_PROJECTIONS: [Graphic3dCameraProjection; 5] = [
    Graphic3dCameraProjection::Orthographic,
    Graphic3dCameraProjection::Perspective,
    Graphic3dCameraProjection::Stereo,
    Graphic3dCameraProjection::MonoLeftEye,
    Graphic3dCameraProjection::MonoRightEye,
];
pub(super) struct ActionScene {
    pub(super) viewer: Handle<V3dViewer>,
    pub(super) view: Handle<dyn V3dViewSource>,
    pub(super) second: Handle<dyn V3dViewSource>,
    pub(super) context: Handle<AisInteractiveContext>,
    pub(super) primary: Handle<AisPoint>,
    pub(super) secondary: Handle<AisPoint>,
    pub(super) callback: Handle<ActionCallbackObject>,
    pub(super) controller: AisViewController,
}
pub(super) fn action_scene(index: usize) -> ActionScene {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = action_primary_view(&viewer, index);
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let primary = action_point(Point3::default());
    let callback = ActionCallbackObject::new();
    let scene = ActionScene {
        viewer: viewer.clone(),
        view,
        second: action_mapped_view(&viewer),
        context,
        primary,
        secondary: action_point(Point3::new(35.0, 15.0, -5.0)),
        callback,
        controller: AisViewController::new().unwrap(),
    };
    initialize_action_scene(&scene, index);
    scene
}
fn initialize_action_scene(scene: &ActionScene, index: usize) {
    display_action_geometry(
        &scene.context,
        &scene.primary,
        &scene.secondary,
        &scene.callback,
    );
    configure_action_grid(&scene.viewer, &scene.view, index);
    configure_action_xr(&scene.view, index);
    scene
        .view
        .set_computed_mode(!(index / 24).is_multiple_of(2));
    scene.callback.calls.store(0, Ordering::Relaxed);
    *scene.callback.last_center.write() = Point3::default();
}

fn action_mapped_view(viewer: &Handle<V3dViewer>) -> Handle<dyn V3dViewSource> {
    let view = viewer.create_view().unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(640, 480);
    view.set_window(&window.cast(), 0).unwrap();
    view.set_immediate_update(false);
    view.camera().set_scale(200.0);
    view.camera().set_aspect(4.0 / 3.0);
    view.camera()
        .set_z_focus(Graphic3dCameraFocusType::Relative, 0.25);
    view
}
fn action_primary_view(viewer: &Handle<V3dViewer>, index: usize) -> Handle<dyn V3dViewSource> {
    let view = action_mapped_view(viewer);
    configure_action_camera(&view, index);
    view
}
fn configure_action_camera(view: &V3dView, index: usize) {
    view.camera()
        .set_eye_and_center(&Point3::new(0.0, 0.0, 200.0), &Point3::default())
        .unwrap();
    view.camera()
        .set_projection_type(ACTION_PROJECTIONS[(index / 24) % 5]);
}
fn action_point(point: Point3) -> Handle<AisPoint> {
    let geometry = Handle::new(GeomCartesianPoint::new(point.x(), point.y(), point.z()));
    AisPoint::new(&geometry.cast()).unwrap()
}
fn display_action_geometry(
    context: &AisInteractiveContext,
    primary: &Handle<AisPoint>,
    secondary: &Handle<AisPoint>,
    callback: &Handle<ActionCallbackObject>,
) {
    context.display(&primary.cast(), false).unwrap();
    context.display(&secondary.cast(), false).unwrap();
    context
        .display_with_modes(&callback.cast(), 0, -1, false, PrsMgrDisplayStatus::None)
        .unwrap();
}
fn configure_action_grid(viewer: &V3dViewer, view: &V3dView, index: usize) {
    let grid = (index / 360) % 4;
    if matches!(grid, 1 | 3) {
        viewer
            .activate_grid(AspectGridType::Rectangular, AspectGridDrawMode::Lines)
            .unwrap();
    }
    if grid == 2 {
        view.grid_display(&AspectGridParams::default());
    }
    if grid == 3 {
        viewer.set_grid_echo(false);
    }
}
fn configure_action_xr(view: &V3dView, index: usize) {
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
pub(super) fn reset_action_redraw(scene: &ActionScene) {
    scene.view.get().unwrap().view().redraw();
    scene.second.get().unwrap().view().redraw();
    scene.view.redraw_immediate();
    scene.second.redraw_immediate();
}
