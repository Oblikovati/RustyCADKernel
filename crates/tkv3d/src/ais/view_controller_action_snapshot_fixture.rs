//! Snapshot real scene and retained state in the pinned native column order.
use super::view_controller_action_scene_fixture::ActionScene;
use super::view_controller_camera_fixture::append_primitive_camera;
use super::view_controller_transfer_fixture::queue_snapshot;
use super::*;
use crate::{
    prs_mgr::PrsMgrPresentableObjectMembers, select_mgr::SelectMgrViewerSelectorResultMembers,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::{Graphic3dCViewXr, Graphic3dCamera};
use std::sync::atomic::Ordering;
pub(super) fn action_snapshot(scene: &ActionScene, result: i32) -> Vec<f64> {
    let mut values = vec![f64::from(result)];
    append_action_views(&mut values, scene);
    append_action_cameras(&mut values, scene);
    append_action_context(&mut values, scene);
    append_action_controller(&mut values, &scene.controller);
    append_action_anchor(
        &mut values,
        scene,
        &scene.controller.camera_action_state.rotate_anchor,
    );
    append_action_anchor(
        &mut values,
        scene,
        &scene.controller.camera_action_state.pan_anchor,
    );
    values.extend(queue_snapshot(&scene.controller.ui_input));
    values.extend(queue_snapshot(&scene.controller.render_input));
    values
}
fn append_action_views(values: &mut Vec<f64>, scene: &ActionScene) {
    for view in [&scene.view, &scene.second] {
        values.extend(
            [
                view.is_invalidated(),
                view.is_invalidated_immediate(),
                view.computed_mode(),
            ]
            .map(f64::from),
        );
    }
    append_action_grid_state(values, scene);
}
fn append_action_grid_state(values: &mut Vec<f64>, scene: &ActionScene) {
    values.extend(
        [
            scene.view.is_grid_active().unwrap(),
            scene.view.is_shader_grid_active(),
            scene.viewer.grid_echo(),
        ]
        .map(f64::from),
    );
    append_action_animation(values, scene);
}
fn append_action_animation(values: &mut Vec<f64>, scene: &ActionScene) {
    values.extend(
        [
            scene.controller.view_animation().is_stopped(),
            scene.controller.view_animation().target_view().is_null(),
        ]
        .map(f64::from),
    );
}
fn append_action_cameras(values: &mut Vec<f64>, scene: &ActionScene) {
    append_primitive_camera(values, &scene.view.camera());
    let cview = scene.view.get().unwrap().view();
    append_action_optional_camera(values, &cview.base_xr_camera());
    append_action_optional_camera(values, &cview.posed_xr_camera());
}
fn append_action_optional_camera(values: &mut Vec<f64>, camera: &Handle<Graphic3dCamera>) {
    values.push(f64::from(!camera.is_null()));
    if let Some(camera) = camera.get() {
        append_primitive_camera(values, camera);
    } else {
        values.extend([0.0; 21]);
    }
}
fn append_action_context(values: &mut Vec<f64>, scene: &ActionScene) {
    let detected = scene.context.detected_interactive();
    values.extend([
        f64::from(scene.context.nb_selected()),
        f64::from(scene.context.main_selector().nb_picked()),
        f64::from(detected == scene.primary),
        f64::from(detected == scene.secondary),
    ]);
    values.push(f64::from(scene.callback.calls.load(Ordering::Relaxed)));
    let center = *scene.callback.last_center.read();
    values.extend([center.x(), center.y(), center.z()]);
    let echo = scene.viewer.state.read().grid_echo_last_vert.coord_f64();
    values.extend([echo.0, echo.1, echo.2]);
}
fn append_action_controller(values: &mut Vec<f64>, c: &AisViewController) {
    values.extend([
        f64::from(c.camera_action_state.had_computed_hlr),
        f64::from(c.previous_move_to.x()),
        f64::from(c.previous_move_to.y()),
        f64::from(c.to_ask_next_frame()),
    ]);
    let p = c.panning_anchor_point();
    values.extend([p.x(), p.y(), p.z()]);
    let state = &c.rotation_state;
    for p in [state.pivot, state.start_eye, state.start_center] {
        values.extend([p.x(), p.y(), p.z()]);
    }
    for d in [state.start_up, state.start_direction] {
        values.extend([d.x(), d.y(), d.z()]);
    }
    values.push(state.current_roll);
}
fn append_action_anchor(values: &mut Vec<f64>, scene: &ActionScene, anchor: &Handle<AisPoint>) {
    values.extend(
        [
            anchor.has_interactive_context(),
            scene.context.is_displayed(&anchor.cast()),
            anchor.is_mutable(),
        ]
        .map(f64::from),
    );
    values.extend([
        f64::from(anchor.z_layer()),
        f64::from(anchor.has_transformation()),
    ]);
    let p = anchor.local_transformation().translation_part();
    values.extend([p.x(), p.y(), p.z()]);
}
