//! Compare complete owned mouse state and queued requests with committed native snapshots.
use super::view_controller_mouse_fixture::*;
#[test]
fn native_mouse_button_and_motion_snapshots_match() {
    for fixture in [
        include_str!("../../tests/fixtures/view-controller-mouse-0-native.txt"),
        include_str!("../../tests/fixtures/view-controller-mouse-1-native.txt"),
        include_str!("../../tests/fixtures/view-controller-mouse-2-native.txt"),
    ] {
        replay_mouse_fixture(fixture);
    }
}
fn replay_mouse_fixture(fixture: &str) {
    let mut controller = mouse_case(0);
    for line in fixture.lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|n| n.parse().unwrap())
            .collect();
        if row[1] == 0.0 {
            controller = mouse_case(row[0] as usize);
        }
        let result = mouse_action(&mut controller, &row);
        assert_eq!(
            mouse_snapshot(&controller, result),
            row[8..],
            "case {} step {}",
            row[0],
            row[1]
        );
    }
}
#[test]
fn default_mouse_bindings_are_live_and_independent() {
    use super::*;
    use rustycad_tkservice::aspect::*;
    let mut c = AisViewController::new().unwrap();
    let other = AisViewController::new().unwrap();
    assert_eq!(c.mouse_gesture_map().extent(), 9);
    assert_left_binding(&c, AisMouseGesture::RotateOrbit);
    c.change_mouse_gesture_map()
        .bind(ASPECT_VKEY_MOUSE_LEFT_BUTTON, AisMouseGesture::Pan);
    assert_left_binding(&c, AisMouseGesture::Pan);
    assert_left_binding(&other, AisMouseGesture::RotateOrbit);
}
#[test]
fn mouse_double_click_accepts_exact_interval_and_restarts_after_expiration() {
    let (mut c, clock) = super::view_controller_keyboard_fixture::prepare(0.0, 0.0);
    assert!(!c.advance_mouse_click_counter());
    clock.set(0.4);
    assert!(c.advance_mouse_click_counter());
    clock.set(0.5);
    assert!(!c.advance_mouse_click_counter());
    clock.set(0.9001);
    assert!(!c.advance_mouse_click_counter());
    assert_eq!(c.click_state.counter, 1);
    c.set_mouse_double_click_interval(f64::NAN);
    assert!(c.advance_mouse_click_counter());
}
#[test]
fn release_pause_failure_preserves_native_pre_exception_state() {
    use rustycad_tkernel::ncollection::Vec2;
    use rustycad_tkservice::aspect::ASPECT_VKEY_MOUSE_LEFT_BUTTON as LEFT;
    let mut c = failing_mouse_release_case();
    c.update_mouse_buttons(Vec2::new(0, 0), LEFT, 0, false)
        .unwrap();
    assert!(
        c.update_mouse_buttons(Vec2::new(0, 0), 0, 0, false)
            .is_err()
    );
    assert_eq!(
        (c.click_state.single_button, c.input_base.mouse_pressed),
        (LEFT as i32, LEFT)
    );
    assert_eq!(c.click_state.counter, 1);
    assert!(c.click_state.timer.is_started() && c.ui_input.selection.points.is_empty());
}
#[test]
fn zoom_coalescing_advances_pointer_even_when_update_returns_false() {
    use rustycad_tkernel::ncollection::Vec2;
    use rustycad_tkservice::aspect::ASPECT_VKEY_MOUSE_RIGHT_BUTTON as RIGHT;
    let mut c = mouse_case(100);
    c.update_mouse_buttons(Vec2::new(0, 0), RIGHT, 0, false)
        .unwrap();
    assert!(
        c.update_mouse_position(Vec2::new(10, 0), RIGHT, 0, false)
            .unwrap()
    );
    assert!(
        !c.update_mouse_position(Vec2::new(20, 0), RIGHT, 0, false)
            .unwrap()
    );
    assert_eq!(c.mouse_dispatch.progress_point, Vec2::new(20, 0));
    assert!(c.ui_input.dragging.to_move);
    assert_eq!(c.ui_input.zoom_actions.len(), 1);
    assert_eq!(c.ui_input.zoom_actions.last().unwrap().delta, 20.0);
}
#[test]
fn touch_or_disabled_policy_suppresses_mouse_hover() {
    use rustycad_tkernel::ncollection::Vec2;
    let mut c = mouse_case(100);
    c.add_touch_point(1, Vec2::new(0.0, 0.0), false).unwrap();
    assert!(
        !c.update_mouse_position(Vec2::new(10, 0), 0, 0, false)
            .unwrap()
    );
    c.remove_touch_point(1, false).unwrap();
    c.set_allow_highlight(false);
    c.ui_input.move_to.to_hilight = false;
    assert!(
        !c.update_mouse_position(Vec2::new(10, 0), 0, 0, false)
            .unwrap()
    );
    assert!(!c.ui_input.move_to.to_hilight);
}

fn assert_left_binding(c: &super::AisViewController, gesture: super::AisMouseGesture) {
    use rustycad_tkservice::aspect::ASPECT_VKEY_MOUSE_LEFT_BUTTON as LEFT;
    assert_eq!(c.mouse_gesture_map().seek(&LEFT), Some(&gesture));
}

fn failing_mouse_release_case() -> super::AisViewController {
    use super::view_controller_click_tests::ClickPauseRecorder;
    use super::*;
    use rustycad_tkernel::standard::Handle;
    let mut c = mouse_case(100);
    let animation = Handle::new(ClickPauseRecorder::new(true));
    animation.start(false).unwrap();
    c.set_objects_animation(&animation.cast::<dyn AisAnimationSource>());
    c.set_pause_objects_animation(true);
    c
}
