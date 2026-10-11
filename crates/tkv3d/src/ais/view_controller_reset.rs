//! Focus loss resets native input bookkeeping without acknowledging other gesture requests.
use super::{AisMouseGesture, AisViewController};
use rustycad_tkservice::aspect::{ASPECT_VKEY_FLAGS_NONE, ASPECT_VKEY_MOUSE_NONE};
impl AisViewController {
    /// Clear cached keys/buttons and request drag abort; `controller.reset_view_input()`.
    pub fn reset_view_input(&mut self) {
        self.keys().reset();
        self.input_base.mouse_pressed = ASPECT_VKEY_MOUSE_NONE;
        self.input_base.mouse_modifiers = ASPECT_VKEY_FLAGS_NONE;
        self.click_state.single_button = -1;
        self.ui_input.dragging.to_abort = true;
        self.click_state.active_gesture = AisMouseGesture::None;
        self.click_state.timer.stop();
        self.click_state.counter = 0;
    }
    /// Reset cached input only on focus loss; `controller.process_focus(false)`.
    pub fn process_focus(&mut self, activated: bool) {
        if !activated {
            self.reset_view_input();
        }
    }
    /// Preserve accumulated input on expose, matching the native empty callback; `controller.process_expose()`.
    pub fn process_expose(&mut self) {}
    /// Preserve accumulated input on resize, matching the native empty callback; `controller.process_configure(true)`.
    pub fn process_configure(&mut self, _is_resized: bool) {}
    /// Leave events queued until explicit flush, matching the native empty callback; `controller.process_input()`.
    pub fn process_input(&mut self) {}
    /// Preserve accumulated input on close, matching the native empty callback; `controller.process_close()`.
    pub fn process_close(&mut self) {}
}
