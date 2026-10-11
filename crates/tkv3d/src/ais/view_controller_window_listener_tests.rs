//! Dynamic window callbacks retain controller behavior and typed animation failures.
use super::view_controller_click_tests::ClickPauseRecorder;
use super::{AisAnimationSource, AisViewController, AisViewInputBufferType};
use rustycad_tkernel::{ncollection::Vec2, standard::Handle};
use rustycad_tkservice::aspect::{
    ASPECT_VKEY_MOUSE_LEFT_BUTTON as LEFT, AspectScrollDelta, AspectVKeyBasic as Key,
    AspectWindowInputListener, AspectWindowInputListenerMembers,
};
use rustycad_tkservice::wnt::WntHidSpaceMouse;

#[test]
fn dynamic_listener_routes_keyboard_and_window_callbacks() {
    let mut controller = AisViewController::new().unwrap();
    let listener: &mut dyn AspectWindowInputListener = &mut controller;
    listener.process_expose();
    listener.process_configure(true);
    listener.process_input();
    listener.process_close();
    listener.key_down(Key::A as u32, 0.0, 1.0).unwrap();
    listener.key_up(Key::A as u32, 0.5).unwrap();
    listener
        .key_from_axis(Key::Left as u32, Key::Right as u32, 0.5, 1.0)
        .unwrap();
    assert!(listener.keys().is_key_down(Key::Right as u32).unwrap());
    listener.process_focus(false);
    assert!(!listener.keys().is_key_down(Key::Right as u32).unwrap());
    listener.process_focus(true);
    assert_eq!(listener.pressed_mouse_buttons(), 0);
}

#[test]
fn dynamic_listener_routes_mouse_and_touch_controller_requests() {
    let mut controller = AisViewController::new().unwrap();
    exercise_mouse_and_touch(&mut controller);
    let ui = controller.input_buffer(AisViewInputBufferType::Ui);
    assert!(ui.dragging.to_start);
    assert!(ui.dragging.to_stop);
    assert!(!ui.zoom_actions.is_empty());
}
fn exercise_mouse_and_touch(listener: &mut dyn AspectWindowInputListener) {
    let point = Vec2::new(10, 20);
    listener.press_mouse_button(point, LEFT, 0, false).unwrap();
    assert_eq!(listener.pressed_mouse_buttons(), LEFT);
    listener
        .update_mouse_position(Vec2::new(30, 40), LEFT, 0, false)
        .unwrap();
    listener
        .release_mouse_button(Vec2::new(30, 40), LEFT, 0, false)
        .unwrap();
    assert_eq!(listener.last_mouse_position(), Vec2::new(30, 40));
    assert!(listener.update_mouse_scroll(&AspectScrollDelta::new(point, 1.0, 0)));
    listener
        .add_touch_point(5, Vec2::new(2.0, 3.0), true)
        .unwrap();
    listener.update_touch_point(5, Vec2::new(3.0, 4.0)).unwrap();
    assert!(listener.has_touch_points());
    assert!(listener.remove_touch_point(5, false).unwrap());
    assert!(!listener.remove_touch_point(5, false).unwrap());
}
fn failing_listener() -> AisViewController {
    let mut controller = AisViewController::new().unwrap();
    let animation = Handle::new(ClickPauseRecorder::new(true));
    animation.start(false).unwrap();
    controller.set_objects_animation(&animation.cast::<dyn AisAnimationSource>());
    controller.set_pause_objects_animation(true);
    controller
}
#[test]
fn dynamic_mouse_release_propagates_animation_failure_before_button_commit() {
    let mut controller = failing_listener();
    let listener: &mut dyn AspectWindowInputListener = &mut controller;
    listener
        .press_mouse_button(Vec2::new(4, 8), LEFT, 0, false)
        .unwrap();
    assert!(
        listener
            .release_mouse_button(Vec2::new(4, 8), LEFT, 0, false)
            .is_err()
    );
    assert_eq!(listener.pressed_mouse_buttons(), LEFT);
    assert!(
        controller
            .input_buffer(AisViewInputBufferType::Ui)
            .selection
            .points
            .is_empty()
    );
}
#[test]
fn dynamic_touch_release_propagates_animation_failure_after_removal() {
    let mut controller = failing_listener();
    let listener: &mut dyn AspectWindowInputListener = &mut controller;
    listener
        .add_touch_point(1, Vec2::new(4.0, 8.0), false)
        .unwrap();
    assert!(listener.remove_touch_point(1, false).is_err());
    assert!(!listener.has_touch_points());
    assert!(
        controller
            .input_buffer(AisViewInputBufferType::Ui)
            .dragging
            .to_stop
    );
}
#[test]
fn dynamic_listener_retains_shared_hid_callbacks() {
    let mut controller = AisViewController::new().unwrap();
    let listener: &mut dyn AspectWindowInputListener = &mut controller;
    let bytes = [3, 2, 0, 0, 0];
    let report = WntHidSpaceMouse::new(0xC62B, &bytes);
    assert!(listener.update_3d_mouse(&report).unwrap());
    assert!(listener.keys().is_key_down(Key::ViewFitAll as u32).unwrap());
}
