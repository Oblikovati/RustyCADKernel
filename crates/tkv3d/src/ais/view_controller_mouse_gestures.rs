//! Primary and secondary gesture activation preserve native policy and ordering.
use super::{AisMouseGesture as G, AisSelectionScheme, AisViewController};
use rustycad_tkernel::ncollection::Vec2;
impl AisViewController {
    pub(super) fn activate_mouse_gesture(
        &mut self,
        point: Vec2<i32>,
        buttons: u32,
        modifiers: u32,
    ) {
        self.mouse_dispatch.idle_rotation = false;
        self.click_state.active_gesture = G::None;
        self.reset_mouse_press_origin(point, buttons);
        if let Some(gesture) = self
            .mouse_gesture_map()
            .seek(&(buttons | modifiers))
            .copied()
        {
            self.click_state.active_gesture = gesture;
            self.activate_primary_mouse_gesture(point, gesture);
        }
        self.activate_secondary_mouse_drag(point, buttons | modifiers);
    }
    fn reset_mouse_press_origin(&mut self, point: Vec2<i32>, buttons: u32) {
        if buttons == 0 {
            return;
        }
        self.mouse_dispatch.press_point = point;
        self.mouse_dispatch.progress_point = point;
    }
    fn activate_primary_mouse_gesture(&mut self, point: Vec2<i32>, gesture: G) {
        match gesture {
            G::RotateView | G::RotateOrbit => self.activate_pointer_rotation(),
            G::Pan => self.activate_pointer_pan(),
            G::Zoom | G::ZoomVertical | G::ZoomWindow => {
                if !self.to_allow_zooming() {
                    self.click_state.active_gesture = G::None;
                }
            }
            G::SelectLasso => self.update_poly_selection(point, true),
            G::Drag => {
                if self.to_allow_dragging() {
                    self.begin_pointer_drag(point);
                } else {
                    self.click_state.active_gesture = G::None;
                }
            }
            G::None | G::SelectRectangle => (),
        }
    }
    fn activate_pointer_rotation(&mut self) {
        if !self.to_allow_rotation() {
            self.click_state.active_gesture = G::None;
            return;
        }
        self.touch_dispatch.update_start_rotation = true;
    }
    fn activate_pointer_pan(&mut self) {
        if !self.to_allow_panning() {
            self.click_state.active_gesture = G::None;
            return;
        }
        self.touch_dispatch.update_start_pan = true;
    }
    fn activate_secondary_mouse_drag(&mut self, point: Vec2<i32>, binding: u32) {
        if self.mouse_dispatch.drag_map.seek(&binding) != Some(&G::Drag)
            || !self.to_allow_dragging()
        {
            return;
        }
        self.begin_pointer_drag(point);
        if self.click_state.active_gesture == G::None {
            self.click_state.active_gesture = G::Drag;
        }
    }
    fn begin_pointer_drag(&mut self, point: Vec2<i32>) {
        self.ui_input.dragging.to_start = true;
        self.ui_input.dragging.point_start = point;
    }
    pub(super) fn finish_mouse_selection(&mut self, previous: G, binding: u32) {
        if !matches!(
            previous,
            G::SelectRectangle | G::SelectLasso | G::ZoomWindow
        ) {
            return;
        }
        self.ui_input.selection.to_apply_tool = true;
        self.ui_input.selection.scheme = self
            .mouse_selection_schemes()
            .seek(&binding)
            .copied()
            .unwrap_or(AisSelectionScheme::Replace);
    }
}
