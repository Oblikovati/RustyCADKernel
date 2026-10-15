//! The `Aspect_WindowInputListener` mouse and touch members that `AIS_ViewController` re-exports
//! with `using` declarations, as inherent forwards to the listener implementation (whose
//! `UpdateMouseButtons` override does the controller bookkeeping).
use super::AisViewController;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::Failure;
use rustycad_tkservice::aspect::{
    AspectVKeyFlags, AspectVKeyMouse, AspectWindowInputListenerMembers,
};

impl AisViewController {
    /// `PressMouseButton`: adds a button to the pressed ones through `UpdateMouseButtons`;
    /// `controller.press_mouse_button(point, button, 0, false)?`.
    pub fn press_mouse_button(
        &mut self,
        point: Vec2<i32>,
        button: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        is_emulated: bool,
    ) -> Result<bool, Failure> {
        AspectWindowInputListenerMembers::press_mouse_button(
            self,
            point,
            button,
            modifiers,
            is_emulated,
        )
    }
    /// `ReleaseMouseButton`: removes a button from the pressed ones through
    /// `UpdateMouseButtons`; `controller.release_mouse_button(point, button, 0, false)?`.
    pub fn release_mouse_button(
        &mut self,
        point: Vec2<i32>,
        button: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        is_emulated: bool,
    ) -> Result<bool, Failure> {
        AspectWindowInputListenerMembers::release_mouse_button(
            self,
            point,
            button,
            modifiers,
            is_emulated,
        )
    }
    /// `PressedMouseButtons`: the active mouse buttons; `controller.pressed_mouse_buttons()`.
    pub fn pressed_mouse_buttons(&self) -> AspectVKeyMouse {
        AspectWindowInputListenerMembers::pressed_mouse_buttons(self)
    }
    /// `LastMouseFlags`: the modifiers of the last mouse event; `controller.last_mouse_flags()`.
    pub fn last_mouse_flags(&self) -> AspectVKeyFlags {
        AspectWindowInputListenerMembers::last_mouse_flags(self)
    }
    /// `LastMousePosition`: the last mouse position; `controller.last_mouse_position()`.
    pub fn last_mouse_position(&self) -> Vec2<i32> {
        AspectWindowInputListenerMembers::last_mouse_position(self)
    }
    /// `HasTouchPoints`: true while a touch is active; `controller.has_touch_points()`.
    pub fn has_touch_points(&self) -> bool {
        AspectWindowInputListenerMembers::has_touch_points(self)
    }
}
