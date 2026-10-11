//! Gesture movement preserves native tolerances, acceleration and accumulated requests.
use super::{AisMouseGesture as G, AisNavigationMode as N, AisViewController};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkservice::aspect::AspectScrollDelta;
impl AisViewController {
    pub(super) fn move_mouse_rotation(
        &mut self,
        point: Vec2<i32>,
        delta: Vec2<i32>,
        emulated: bool,
    ) -> bool {
        if !self.to_allow_rotation() {
            return false;
        }
        self.start_pointer_rotation_if_needed();
        if !self.exceeds_mouse_tolerance(delta, emulated, self.mouse_dispatch.rotation_threshold) {
            return false;
        }
        self.queue_accelerated_mouse_rotation(point);
        self.queue_pointer_drag_move(point);
        true
    }
    fn exceeds_mouse_tolerance(&self, delta: Vec2<i32>, emulated: bool, threshold: f32) -> bool {
        mouse_motion_exceeds(delta, self.emulated_mouse_tolerance(emulated, threshold))
    }
    fn queue_accelerated_mouse_rotation(&mut self, point: Vec2<i32>) {
        let target = self.accelerated_mouse_rotation(point);
        if self.click_state.active_gesture == G::RotateOrbit {
            self.ui_input.orbit_rotation.to_rotate = true;
            self.ui_input.orbit_rotation.point_to = target;
        } else {
            self.ui_input.view_rotation.to_rotate = true;
            self.ui_input.view_rotation.point_to = target;
        }
    }
    fn start_pointer_rotation_if_needed(&mut self) {
        if !self.touch_dispatch.update_start_rotation {
            return;
        }
        let point = self.mouse_dispatch.press_point;
        let start = Vec2::new(point.x as f64, point.y as f64);
        if self.click_state.active_gesture == G::RotateOrbit {
            self.ui_input.orbit_rotation.to_start = true;
            self.ui_input.orbit_rotation.point_start = start;
        } else {
            self.ui_input.view_rotation.to_start = true;
            self.ui_input.view_rotation.point_start = start;
        }
        self.touch_dispatch.update_start_rotation = false;
    }
    pub(super) fn move_mouse_zoom(
        &mut self,
        point: Vec2<i32>,
        delta: Vec2<i32>,
        emulated: bool,
    ) -> bool {
        if !self.to_allow_zooming() {
            return false;
        }
        let tolerance = self.emulated_mouse_tolerance(emulated, self.mouse_dispatch.zoom_threshold);
        let scroll = self.mouse_scroll_component(delta);
        if scroll.abs().partial_cmp(&tolerance) != Some(std::cmp::Ordering::Greater) {
            return false;
        }
        let changed = self.update_zoom(&AspectScrollDelta::with_delta(scroll, 0));
        self.queue_pointer_drag_move(point);
        changed
    }
    fn mouse_scroll_component(&self, delta: Vec2<i32>) -> f64 {
        if self.click_state.active_gesture == G::Zoom {
            return delta.x as f64;
        }
        delta.y as f64
    }
    pub(super) fn move_mouse_pan(
        &mut self,
        point: Vec2<i32>,
        mut delta: Vec2<i32>,
        emulated: bool,
    ) -> bool {
        if !self.to_allow_panning() {
            return false;
        }
        if !self.exceeds_mouse_tolerance(delta, emulated, self.mouse_dispatch.pan_threshold) {
            return false;
        }
        self.start_pointer_pan_if_needed();
        delta.y = -delta.y;
        self.accumulate_mouse_pan(delta);
        self.queue_pointer_drag_move(point);
        true
    }
    fn accumulate_mouse_pan(&mut self, delta: Vec2<i32>) {
        if self.ui_input.panning.to_pan {
            self.ui_input.panning.delta += delta;
            return;
        }
        self.ui_input.panning.to_pan = true;
        self.ui_input.panning.delta = delta;
    }
    fn start_pointer_pan_if_needed(&mut self) {
        if !self.touch_dispatch.update_start_pan {
            return;
        }
        self.ui_input.panning.to_start = true;
        self.ui_input.panning.point_start = self.mouse_dispatch.press_point;
        self.touch_dispatch.update_start_pan = false;
    }
    pub(super) fn move_mouse_drag(
        &mut self,
        point: Vec2<i32>,
        delta: Vec2<i32>,
        emulated: bool,
    ) -> bool {
        if !self.to_allow_dragging() {
            return false;
        }
        let tolerance =
            self.emulated_mouse_tolerance(emulated, self.mouse_dispatch.dragging_threshold);
        let displacement = (delta.x.abs() + delta.y.abs()) as f64;
        if displacement.partial_cmp(&tolerance) != Some(std::cmp::Ordering::Greater) {
            return false;
        }
        self.ui_input.view_rotation.to_rotate = true;
        self.ui_input.view_rotation.point_to = self.accelerated_mouse_rotation(point);
        self.queue_pointer_drag_move(point);
        true
    }
    fn accelerated_mouse_rotation(&self, point: Vec2<i32>) -> Vec2<f64> {
        let acceleration = if self.navigation_mode() == N::FirstPersonWalk {
            self.mouse_acceleration()
        } else {
            self.orbit_acceleration()
        } as f64;
        let press = self.mouse_dispatch.press_point;
        let delta = point - press;
        Vec2::new(press.x as f64, press.y as f64)
            + Vec2::new(delta.x as f64, delta.y as f64) * acceleration
    }
    fn queue_pointer_drag_move(&mut self, point: Vec2<i32>) {
        self.ui_input.dragging.to_move = true;
        self.ui_input.dragging.point_to = point;
        self.mouse_dispatch.progress_point = point;
    }
}
fn mouse_motion_manhattan(delta: Vec2<i32>) -> f64 {
    (delta.x as f64).abs() + (delta.y as f64).abs()
}

fn mouse_motion_exceeds(delta: Vec2<i32>, tolerance: f64) -> bool {
    mouse_motion_manhattan(delta).partial_cmp(&tolerance) == Some(std::cmp::Ordering::Greater)
}
