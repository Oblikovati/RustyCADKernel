//! Controller event sequences replay the pinned native viewer queue behavior.
#[path = "support/view_controller_fixture.rs"]
mod fixture;
use fixture::*;
use rustycad_tkernel::ncollection::{Sequence, Vec2};
use rustycad_tkservice::aspect::*;
use rustycad_tkv3d::ais::*;
#[test]
fn controller_input_roles_start_independent() {
    let c = AisViewController::new().unwrap();
    assert_event(&c, "default-ui", -1);
    assert_snapshot(&parameters(&c), "parameters-default");
    assert_snapshot(&event(&c, AisViewInputBufferType::Gl, -1), "default-gl");
}
#[test]
fn controller_picks_and_polygon_edits_follow_native_transitions() {
    let mut c = AisViewController::new().unwrap();
    c.select_in_viewer(Vec2::new(1, 2), AisSelectionScheme::Add);
    assert_event(&c, "pick-first", -1);
    c.select_in_viewer(Vec2::new(3, 4), AisSelectionScheme::Xor);
    assert_event(&c, "pick-second", -1);
    c.update_rubber_band(Vec2::new(5, 6), Vec2::new(7, 8));
    assert_event(&c, "rectangle", -1);
    check_polygon(&mut c);
    c.select_in_viewer(Vec2::new(15, 16), AisSelectionScheme::Remove);
    assert_event(&c, "pick-transition", -1);
    assert_snapshot(&event(&c, AisViewInputBufferType::Gl, -1), "selection-gl");
}
fn check_polygon(c: &mut AisViewController) {
    c.update_poly_selection(Vec2::new(9, 10), false);
    assert_event(c, "poly-first", -1);
    c.update_poly_selection(Vec2::new(9, 10), true);
    assert_event(c, "poly-duplicate", -1);
    c.update_poly_selection(Vec2::new(11, 12), true);
    assert_event(c, "poly-append", -1);
    c.update_poly_selection(Vec2::new(13, 14), false);
    assert_event(c, "poly-replace", -1);
}
#[test]
fn controller_sequence_selection_snapshots_points_and_retains_apply_flag() {
    for length in 0..=3 {
        check_sequence(length);
    }
}
fn check_sequence(length: i32) {
    let mut c = AisViewController::new().unwrap();
    let mut points = Sequence::default();
    for i in 0..length {
        points.append(Vec2::new(i, -i));
    }
    c.select_points_in_viewer(&points, AisSelectionScheme::ReplaceExtra);
    points.clear(None);
    assert_event(&c, &format!("sequence-{length}"), -1);
    check_sequence_edits(&mut c, length);
}
fn check_sequence_edits(c: &mut AisViewController, length: i32) {
    c.update_rubber_band(Vec2::new(5, 6), Vec2::new(7, 8));
    assert_event(c, &format!("sequence-rectangle-{length}"), -1);
    c.update_poly_selection(Vec2::new(9, 10), false);
    assert_event(c, &format!("sequence-polygon-{length}"), -1);
    c.select_in_viewer(Vec2::new(15, 16), AisSelectionScheme::Remove);
    assert_event(c, &format!("sequence-pick-{length}"), -1);
}
#[test]
fn controller_zoom_merges_only_adjacent_pixels_and_preserves_first_modifiers() {
    let mut c = AisViewController::new().unwrap();
    for (name, x, y, delta, flags) in [
        ("zoom-first", 1, 2, 0.5, ASPECT_VKEY_FLAGS_SHIFT),
        ("zoom-merge", 1, 2, -0.5, ASPECT_VKEY_FLAGS_CTRL),
        ("zoom-next", 3, 4, 2., ASPECT_VKEY_FLAGS_ALT),
        ("zoom-revisit", 1, 2, 0.25, ASPECT_VKEY_FLAGS_CTRL),
    ] {
        let result = c.update_zoom(&AspectScrollDelta::new(Vec2::new(x, y), delta, flags));
        assert_event(&c, name, result as i32);
    }
    check_scroll(&mut c);
}
fn check_scroll(c: &mut AisViewController) {
    let result = c.update_mouse_scroll(&AspectScrollDelta::new(
        Vec2::new(1, 2),
        0.25,
        ASPECT_VKEY_FLAGS_SHIFT,
    ));
    assert_event(c, "scroll-merge", result as i32);
    let result =
        c.update_mouse_scroll(&AspectScrollDelta::with_delta(-0.5, ASPECT_VKEY_FLAGS_NONE));
    assert_event(c, "scroll-undefined", result as i32);
    c.set_allow_zooming(false);
    let result = c.update_zoom(&AspectScrollDelta::with_delta(
        -0.25,
        ASPECT_VKEY_FLAGS_CTRL,
    ));
    assert_event(c, "zoom-disabled", result as i32);
    let result = c.update_mouse_scroll(&AspectScrollDelta::with_delta(0.5, ASPECT_VKEY_FLAGS_ALT));
    assert_event(c, "scroll-disabled", result as i32);
}
#[test]
fn controller_touch_rotation_gates_accumulation_and_restarts_after_reset() {
    let mut c = AisViewController::new().unwrap();
    let result = c.update_z_rotation(0.5);
    assert_event(&c, "z-disabled", result as i32);
    c.set_allow_touch_z_rotation(true);
    let result = c.update_z_rotation(0.5);
    assert_event(&c, "z-first", result as i32);
    let result = c.update_z_rotation(-0.25);
    assert_event(&c, "z-merge", result as i32);
    c.set_allow_touch_z_rotation(false);
    let result = c.update_z_rotation(1.);
    assert_event(&c, "z-paused", result as i32);
    check_rotation_restart(&mut c);
}
fn check_rotation_restart(c: &mut AisViewController) {
    c.change_input_buffer(AisViewInputBufferType::Ui).reset();
    c.set_allow_touch_z_rotation(true);
    let result = c.update_z_rotation(-0.5);
    assert_event(c, "z-restarted", result as i32);
    check_navigation(c);
}
fn check_navigation(c: &mut AisViewController) {
    for (index, mode) in [
        AisNavigationMode::Orbit,
        AisNavigationMode::FirstPersonFlight,
        AisNavigationMode::FirstPersonWalk,
    ]
    .into_iter()
    .enumerate()
    {
        check_navigation_mode(c, index, mode);
    }
    c.reset_previous_move_to();
    assert_event(c, "previous-reset", -1);
}
fn seed_rotations(c: &mut AisViewController) {
    for role in [AisViewInputBufferType::Ui, AisViewInputBufferType::Gl] {
        let b = c.change_input_buffer(role);
        b.orbit_rotation.to_start = true;
        b.orbit_rotation.to_rotate = true;
        b.view_rotation.to_start = true;
        b.view_rotation.to_rotate = true;
    }
}
fn check_navigation_mode(c: &mut AisViewController, index: usize, mode: AisNavigationMode) {
    seed_rotations(c);
    c.set_navigation_mode(mode);
    assert_event(c, &format!("navigation-ui-{index}"), -1);
    assert_snapshot(
        &event(c, AisViewInputBufferType::Gl, -1),
        &format!("navigation-gl-{index}"),
    );
    c.set_navigation_mode(mode);
    assert_event(c, &format!("navigation-repeat-{index}"), -1);
}
#[test]
fn controller_reexported_listener_mouse_members_track_the_buttons() {
    let mut c = AisViewController::new().unwrap();
    assert!(!c.has_touch_points());
    assert_eq!(c.pressed_mouse_buttons(), ASPECT_VKEY_MOUSE_NONE);
    let initial = c.last_mouse_position();
    let left = ASPECT_VKEY_MOUSE_LEFT_BUTTON;
    c.press_mouse_button(Vec2::new(5, 7), left, ASPECT_VKEY_FLAGS_SHIFT, false)
        .unwrap();
    assert_eq!(c.pressed_mouse_buttons(), left);
    assert_eq!(c.last_mouse_flags(), ASPECT_VKEY_FLAGS_SHIFT);
    // The native UpdateMouseButtons override keeps the last position (UpdateMousePosition owns it).
    assert_eq!(c.last_mouse_position(), initial);
    c.release_mouse_button(Vec2::new(6, 8), left, 0, false)
        .unwrap();
    assert_eq!(c.pressed_mouse_buttons(), ASPECT_VKEY_MOUSE_NONE);
    assert_eq!(c.last_mouse_flags(), 0);
}
