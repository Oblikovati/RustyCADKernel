//! Native mouse gesture maps and pointer origins for Martensite dispatch.
use super::{AisMouseGesture, AisViewController};
use rustycad_tkernel::ncollection::{DataMap, Vec2};
use rustycad_tkservice::aspect::{
    ASPECT_VKEY_FLAGS_ALT as ALT, ASPECT_VKEY_FLAGS_CTRL as CTRL, ASPECT_VKEY_FLAGS_SHIFT as SHIFT,
    ASPECT_VKEY_MOUSE_LEFT_BUTTON as LEFT, ASPECT_VKEY_MOUSE_MIDDLE_BUTTON as MIDDLE,
    ASPECT_VKEY_MOUSE_RIGHT_BUTTON as RIGHT,
};
pub(super) struct ViewControllerMouseDispatch {
    pub(super) gesture_map: DataMap<u32, AisMouseGesture>,
    pub(super) drag_map: DataMap<u32, AisMouseGesture>,
    pub(super) press_point: Vec2<i32>,
    pub(super) progress_point: Vec2<i32>,
    pub(super) idle_rotation: bool,
    pub(super) stop_drag_on_unclick: bool,
    pub(super) click_threshold: f64,
    pub(super) rotation_threshold: f32,
    pub(super) pan_threshold: f32,
    pub(super) zoom_threshold: f32,
    pub(super) dragging_threshold: f32,
}
impl Default for ViewControllerMouseDispatch {
    fn default() -> Self {
        let mut drag_map = DataMap::default();
        drag_map.bind(LEFT, AisMouseGesture::Drag);
        Self {
            gesture_map: native_mouse_gestures(),
            drag_map,
            press_point: Vec2::new(0, 0),
            progress_point: Vec2::new(0, 0),
            idle_rotation: false,
            stop_drag_on_unclick: false,
            click_threshold: 3.0,
            rotation_threshold: 6.0,
            pan_threshold: 4.0,
            zoom_threshold: 6.0,
            dragging_threshold: 6.0,
        }
    }
}
fn native_mouse_gestures() -> DataMap<u32, AisMouseGesture> {
    use AisMouseGesture as G;
    let mut map = DataMap::default();
    for (key, gesture) in [
        (LEFT, G::RotateOrbit),
        (LEFT | CTRL, G::Zoom),
        (LEFT | SHIFT, G::Pan),
        (LEFT | ALT, G::SelectRectangle),
        (LEFT | ALT | SHIFT, G::SelectRectangle),
        (RIGHT, G::Zoom),
        (RIGHT | CTRL, G::RotateOrbit),
        (MIDDLE, G::Pan),
        (MIDDLE | CTRL, G::Pan),
    ] {
        map.bind(key, gesture);
    }
    map
}
impl AisViewController {
    /// Borrow live button/modifier gesture bindings; `controller.mouse_gesture_map().extent()`.
    pub fn mouse_gesture_map(&self) -> &DataMap<u32, AisMouseGesture> {
        &self.mouse_dispatch.gesture_map
    }
    /// Edit live gesture bindings; `controller.change_mouse_gesture_map().bind(button, gesture)`.
    pub fn change_mouse_gesture_map(&mut self) -> &mut DataMap<u32, AisMouseGesture> {
        &mut self.mouse_dispatch.gesture_map
    }
    pub(super) fn mouse_click_tolerance(&self, emulated: bool) -> f64 {
        let scale = if emulated {
            self.touch_tolerance_scale() as f64
        } else {
            1.0
        };
        scale * self.mouse_dispatch.click_threshold
    }
    pub(super) fn emulated_mouse_tolerance(&self, emulated: bool, threshold: f32) -> f64 {
        if emulated {
            return self.touch_tolerance_scale() as f64 * threshold as f64;
        }
        0.0
    }
    pub(super) fn pointer_maximum_delta(&self, point: Vec2<i32>) -> f64 {
        let delta = point - self.mouse_dispatch.press_point;
        delta.x.abs().max(delta.y.abs()) as f64
    }
}
