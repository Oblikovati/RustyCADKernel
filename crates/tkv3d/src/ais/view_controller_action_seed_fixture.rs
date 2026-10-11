//! Native scenario combinations preserve all unrelated UI/GL requests.
use super::view_controller_action_scene_fixture::ActionScene;
use super::view_controller_transfer_fixture::seed_queue;
use super::*;
use crate::prs_mgr::PrsMgrDisplayStatus;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::*;
const ACTION_MODES: [AisNavigationMode; 3] = [
    AisNavigationMode::Orbit,
    AisNavigationMode::FirstPersonFlight,
    AisNavigationMode::FirstPersonWalk,
];
pub(super) fn seed_action_scene(scene: &mut ActionScene, index: usize) {
    let c = &mut scene.controller;
    c.set_navigation_mode(ACTION_MODES[(index / 120) % 3]);
    seed_queue(&mut c.ui_input, index as i32, 0);
    seed_queue(&mut c.render_input, index as i32, 1);
    clear_camera_actions(&mut c.render_input);
    seed_camera_motion(c, index % 24);
    c.input_base.mouse_position_last = Vec2::new(320, 240);
    c.previous_move_to = Vec2::new(-1, -1);
    c.set_ask_next_frame(false);
    seed_action_rotation(c, &scene.view.camera());
    seed_action_scroll(c, index % 24);
    seed_action_policy(c, index);
    register_action_anchors(scene, index % 24);
    scene.controller.camera_action_state.had_computed_hlr = index % 24 == 19;
    scene.controller.view_animation().set_view(&scene.view);
    scene.controller.view_animation().start(false).unwrap();
}
fn clear_camera_actions(b: &mut AisViewInputBuffer) {
    b.orientation.to_set_view_orient = false;
    b.orientation.to_fit_all = false;
    b.panning.to_start = false;
    b.panning.to_pan = false;
    b.orbit_rotation.to_start = false;
    b.orbit_rotation.to_rotate = false;
    b.view_rotation.to_start = false;
    b.view_rotation.to_rotate = false;
    b.z_rotate.to_rotate = false;
    b.is_new_gesture = false;
    b.zoom_actions.clear(None);
}
fn seed_camera_motion(c: &mut AisViewController, scenario: usize) {
    let b = &mut c.render_input;
    b.orientation.to_set_view_orient = scenario == 1;
    b.orientation.to_fit_all = scenario == 2;
    b.panning.to_start = [3, 4, 18, 20].contains(&scenario);
    b.panning.to_pan = [3, 4, 5, 6, 18, 20].contains(&scenario);
    b.panning.point_start = if scenario == 4 {
        Vec2::new(-1, -1)
    } else {
        Vec2::new(320, 240)
    };
    b.panning.delta = Vec2::new(17, -9);
    seed_action_rotation_requests(b, scenario);
    b.z_rotate.to_rotate = [7, 18, 21].contains(&scenario);
    b.z_rotate.angle = 0.07;
    b.is_new_gesture = scenario == 19;
    if scenario == 5 {
        c.set_panning_anchor_point(&Point3::new(7.0, -3.0, 20.0));
    }
}
fn seed_action_rotation_requests(b: &mut AisViewInputBuffer, scenario: usize) {
    b.orbit_rotation.to_start = [8, 10, 18, 21].contains(&scenario);
    b.orbit_rotation.to_rotate = [9, 10, 18, 21].contains(&scenario);
    b.view_rotation.to_start = [11, 13, 18, 21].contains(&scenario);
    b.view_rotation.to_rotate = [12, 13, 18, 21].contains(&scenario);
    b.orbit_rotation.point_start = Vec2::new(137.0, 91.0);
    b.view_rotation.point_start = Vec2::new(137.0, 91.0);
    b.orbit_rotation.point_to = Vec2::new(217.0, 151.0);
    b.view_rotation.point_to = Vec2::new(217.0, 151.0);
}
fn seed_action_rotation(
    c: &mut AisViewController,
    camera: &rustycad_tkservice::graphic3d::Graphic3dCamera,
) {
    let state = &mut c.rotation_state;
    state.pivot = Point3::new(7.0, -3.0, 20.0);
    state.start_up = camera.up();
    state.start_direction = camera.direction();
    state.start_eye = camera.eye();
    state.start_center = camera.center();
    state.current_roll = 0.21;
}
fn seed_action_policy(c: &mut AisViewController, index: usize) {
    let scenario = index % 24;
    c.set_allow_panning(scenario != 20);
    c.set_allow_rotation(scenario != 21);
    c.set_allow_zooming(scenario != 22);
    c.set_lock_orbit_z_up(scenario.is_multiple_of(2));
    c.set_show_pan_anchor_point((index / 120).is_multiple_of(2));
    c.set_show_rotate_center((index / 240).is_multiple_of(2));
    c.set_allow_z_focus((index / 480).is_multiple_of(2));
    c.set_stick_to_ray_on_zoom((index / 720).is_multiple_of(2));
}
fn seed_action_scroll(c: &mut AisViewController, scenario: usize) {
    if [14, 18, 22].contains(&scenario) {
        c.render_input
            .zoom_actions
            .append(AspectScrollDelta::new(Vec2::new(320, 240), 40.0, 0));
    }
    if scenario == 15 {
        c.render_input
            .zoom_actions
            .append(AspectScrollDelta::with_delta(-50.0, 0));
    }
    seed_action_focus_scroll(c, scenario);
}
fn seed_action_focus_scroll(c: &mut AisViewController, scenario: usize) {
    if scenario == 16 {
        c.render_input.zoom_actions.append(AspectScrollDelta::new(
            Vec2::new(320, 240),
            0.25,
            ASPECT_VKEY_FLAGS_CTRL,
        ));
    }
    if scenario == 17 {
        seed_action_multi_scroll(c);
    }
    if scenario == 23 {
        c.render_input
            .zoom_actions
            .append(AspectScrollDelta::new(Vec2::new(640, 480), 5.0, 0));
    }
}
fn seed_action_multi_scroll(c: &mut AisViewController) {
    c.render_input.zoom_actions.append(AspectScrollDelta::new(
        Vec2::new(320, 240),
        20.0,
        ASPECT_VKEY_FLAGS_CTRL,
    ));
    c.render_input
        .zoom_actions
        .append(AspectScrollDelta::new(Vec2::new(137, 91), 0.25, 0));
    c.render_input
        .zoom_actions
        .append(AspectScrollDelta::with_delta(-100.0, 0));
}
fn register_action_anchors(scene: &ActionScene, scenario: usize) {
    if scenario != 19 {
        return;
    }
    let state = &scene.controller.camera_action_state;
    for anchor in [&state.rotate_anchor, &state.pan_anchor] {
        register_action_anchor(scene, anchor);
    }
}
fn register_action_anchor(
    scene: &ActionScene,
    anchor: &rustycad_tkernel::standard::Handle<AisPoint>,
) {
    scene
        .context
        .display_with_modes(&anchor.cast(), 0, -1, false, PrsMgrDisplayStatus::Displayed)
        .unwrap();
}

pub(super) fn action_walk(scenario: usize) -> AisWalkDelta {
    let mut walk = AisWalkDelta::new();
    if scenario == 18 {
        walk[AisWalkRotation::Yaw].value = 0.15;
        walk[AisWalkRotation::Pitch].value = -0.1;
        let roll = &mut walk[AisWalkRotation::Roll];
        roll.value = -0.3;
        roll.pressure = 0.25;
        roll.duration = 0.03;
    }
    if scenario == 23 {
        walk[AisWalkTranslation::Forward].value = 0.2;
        walk.set_defined(true);
    }
    walk
}
