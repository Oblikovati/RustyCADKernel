//! Native click/drag tracking is evaluated before gesture bindings change.
use super::{AisNavigationMode, AisViewController};
use rustycad_tkernel::{ncollection::Vec2, standard::Failure};
use rustycad_tkservice::aspect::{
    ASPECT_VKEY_MOUSE_LEFT_BUTTON as LEFT, ASPECT_VKEY_MOUSE_MIDDLE_BUTTON as MIDDLE,
    ASPECT_VKEY_MOUSE_RIGHT_BUTTON as RIGHT, AspectVKeyFlags, AspectVKeyMouse,
};
impl AisViewController {
    /// Queue native button transitions and click/drag bookkeeping; `controller.update_mouse_buttons(point, buttons, modifiers, false)?`.
    pub fn update_mouse_buttons(
        &mut self,
        point: Vec2<i32>,
        buttons: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        emulated: bool,
    ) -> Result<bool, Failure> {
        let changed = self.track_mouse_buttons(point, buttons, modifiers, emulated)?;
        let previous = self.click_state.active_gesture;
        let previous_binding = self.input_base.mouse_pressed | self.input_base.mouse_modifiers;
        self.input_base.mouse_modifiers = modifiers;
        self.input_base.mouse_pressed = buttons;
        if emulated || self.navigation_mode() != AisNavigationMode::FirstPersonWalk {
            self.activate_mouse_gesture(point, buttons, modifiers);
        }
        Ok(self.finish_pointer_transition(previous, previous_binding) || changed)
    }
    fn finish_pointer_transition(
        &mut self,
        previous: super::AisMouseGesture,
        binding: u32,
    ) -> bool {
        if previous == self.click_state.active_gesture {
            return false;
        }
        self.finish_mouse_selection(previous, binding);
        self.ui_input.is_new_gesture = true;
        true
    }
    fn track_mouse_buttons(
        &mut self,
        point: Vec2<i32>,
        buttons: u32,
        modifiers: u32,
        emulated: bool,
    ) -> Result<bool, Failure> {
        if buttons == 0 && self.click_state.single_button > 0 {
            return self.release_single_mouse_button(point, modifiers, emulated);
        }
        if buttons == 0 {
            return Ok(self.release_other_mouse_buttons());
        }
        if self.click_state.single_button == -1 {
            self.press_single_mouse_button(point, buttons, emulated);
            return Ok(false);
        }
        Ok(self.abort_repeated_mouse_press())
    }
    fn abort_repeated_mouse_press(&mut self) -> bool {
        self.click_state.single_button = 0;
        self.ui_input.dragging.to_abort = true;
        true
    }
    fn release_single_mouse_button(
        &mut self,
        point: Vec2<i32>,
        modifiers: u32,
        emulated: bool,
    ) -> Result<bool, Failure> {
        let changed = if self.pointer_maximum_delta(point) < self.mouse_click_tolerance(emulated) {
            let double_click = self.advance_mouse_click_counter();
            self.dispatch_released_mouse_click(point, modifiers, double_click)?
        } else {
            self.cancel_released_mouse_click();
            true
        };
        self.click_state.single_button = -1;
        Ok(changed)
    }
    fn dispatch_released_mouse_click(
        &mut self,
        point: Vec2<i32>,
        modifiers: u32,
        twice: bool,
    ) -> Result<bool, Failure> {
        self.update_mouse_click(
            point,
            self.click_state.single_button as u32,
            modifiers,
            twice,
        )
    }
    fn cancel_released_mouse_click(&mut self) {
        self.click_state.timer.stop();
        self.click_state.counter = 0;
        self.mouse_dispatch.stop_drag_on_unclick = false;
        self.ui_input.dragging.to_stop = true;
    }
    fn release_other_mouse_buttons(&mut self) -> bool {
        self.click_state.single_button = -1;
        if !self.mouse_dispatch.stop_drag_on_unclick {
            return false;
        }
        self.mouse_dispatch.stop_drag_on_unclick = false;
        self.ui_input.dragging.to_stop = true;
        true
    }
    fn press_single_mouse_button(&mut self, point: Vec2<i32>, buttons: u32, emulated: bool) {
        self.click_state.single_button = first_native_mouse_button(buttons);
        if self.click_state.single_button == 0 {
            return;
        }
        if self.click_state.counter == 1
            && self.pointer_maximum_delta(point) >= self.mouse_click_tolerance(emulated)
        {
            self.click_state.timer.stop();
            self.click_state.counter = 0;
        }
        self.mouse_dispatch.press_point = point;
    }
    fn mouse_click_timer_valid(&self) -> bool {
        // Native tests elapsed > interval then negates; NaN therefore keeps a running timer valid.
        self.click_state.timer.is_started()
            && self
                .click_state
                .timer
                .elapsed_time()
                .partial_cmp(&self.mouse_double_click_interval())
                != Some(std::cmp::Ordering::Greater)
    }
    pub(super) fn advance_mouse_click_counter(&mut self) -> bool {
        self.click_state.counter += 1;
        let timer_valid = self.mouse_click_timer_valid();
        let double_click = self.click_state.counter == 2 && timer_valid;
        if !timer_valid {
            self.click_state.counter = 1;
        }
        self.click_state.counter %= 2;
        self.click_state.timer.stop();
        self.click_state.timer.reset();
        self.click_state.timer.start();
        if double_click {
            self.click_state.counter = 0;
        }
        double_click
    }
}
fn first_native_mouse_button(buttons: u32) -> i32 {
    for button in [LEFT, RIGHT, MIDDLE] {
        if buttons & button == button {
            return button as i32;
        }
    }
    0
}
