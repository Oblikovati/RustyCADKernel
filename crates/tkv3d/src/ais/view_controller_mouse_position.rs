//! Native pointer routing keeps gesture requests distinct from repaint return values.
use super::{
    AisMouseGesture as G, AisNavigationMode as N, AisViewController, AisViewSelectionTool,
};
use rustycad_tkernel::{ncollection::Vec2, standard::Failure};
use rustycad_tkservice::aspect::{AspectVKeyFlags, AspectVKeyMouse};
impl AisViewController {
    /// Queue native motion, selection and idle-walk rotation; `controller.update_mouse_position(point, buttons, modifiers, false)?`.
    pub fn update_mouse_position(
        &mut self,
        point: Vec2<i32>,
        buttons: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        emulated: bool,
    ) -> Result<bool, Failure> {
        self.input_base.mouse_position_last = point;
        self.confirm_pointer_drag(point, emulated);
        let mut delta = point - self.mouse_dispatch.progress_point;
        self.update_idle_mouse_rotation(point, emulated, &mut delta);
        let mut changed = false;
        if self.input_base.mouse_modifiers != modifiers {
            changed = self.update_mouse_buttons(point, buttons, modifiers, emulated)?;
        }
        changed = self.dispatch_mouse_motion(point, delta, emulated) || changed;
        changed = self.highlight_pointer_motion(point, buttons, emulated) || changed;
        Ok(changed)
    }
    fn confirm_pointer_drag(&mut self, point: Vec2<i32>, emulated: bool) {
        if self.click_state.single_button <= 0
            || !matches!(
                self.pointer_maximum_delta(point)
                    .partial_cmp(&self.mouse_click_tolerance(emulated)),
                Some(std::cmp::Ordering::Equal | std::cmp::Ordering::Greater)
            )
        {
            return;
        }
        self.click_state.timer.stop();
        self.click_state.counter = 0;
        self.click_state.single_button = -1;
        self.mouse_dispatch.stop_drag_on_unclick = true;
        self.ui_input.dragging.to_confirm = true;
    }
    fn update_idle_mouse_rotation(
        &mut self,
        point: Vec2<i32>,
        emulated: bool,
        delta: &mut Vec2<i32>,
    ) {
        if !emulated && self.navigation_mode() == N::FirstPersonWalk {
            if !self.mouse_dispatch.idle_rotation
                || self.click_state.active_gesture != G::RotateView
            {
                self.begin_idle_mouse_rotation(point);
                *delta = Vec2::new(0, 0);
            }
            return;
        }
        if self.mouse_dispatch.idle_rotation && self.click_state.active_gesture == G::RotateView {
            self.click_state.active_gesture = G::None;
        }
        self.mouse_dispatch.idle_rotation = false;
    }
    fn begin_idle_mouse_rotation(&mut self, point: Vec2<i32>) {
        self.mouse_dispatch.idle_rotation = true;
        self.click_state.active_gesture = G::RotateView;
        self.mouse_dispatch.press_point = point;
        self.mouse_dispatch.progress_point = point;
        self.touch_dispatch.update_start_rotation = false;
        self.ui_input.view_rotation.to_start = true;
        self.ui_input.view_rotation.point_start = Vec2::new(point.x as f64, point.y as f64);
        self.ui_input.view_rotation.to_rotate = false;
    }
    fn dispatch_mouse_motion(
        &mut self,
        point: Vec2<i32>,
        delta: Vec2<i32>,
        emulated: bool,
    ) -> bool {
        match self.click_state.active_gesture {
            G::SelectRectangle | G::ZoomWindow => self.move_mouse_rectangle(point),
            G::SelectLasso => {
                self.update_poly_selection(point, true);
                true
            }
            G::RotateOrbit | G::RotateView => self.move_mouse_rotation(point, delta, emulated),
            G::Zoom | G::ZoomVertical => self.move_mouse_zoom(point, delta, emulated),
            G::Pan => self.move_mouse_pan(point, delta, emulated),
            G::Drag => self.move_mouse_drag(point, delta, emulated),
            G::None => false,
        }
    }
    fn move_mouse_rectangle(&mut self, point: Vec2<i32>) -> bool {
        self.update_rubber_band(self.mouse_dispatch.press_point, point);
        if self.click_state.active_gesture == G::ZoomWindow {
            self.ui_input.selection.tool = AisViewSelectionTool::ZoomWindow;
        }
        true
    }
    fn highlight_pointer_motion(&mut self, point: Vec2<i32>, buttons: u32, emulated: bool) -> bool {
        if buttons != 0
            || self.navigation_mode() == N::FirstPersonWalk
            || emulated
            || !self.input_base.touch_points.is_empty()
            || !self.to_allow_highlight()
        {
            return false;
        }
        self.ui_input.move_to.to_hilight = true;
        self.ui_input.move_to.point = point;
        true
    }
}
