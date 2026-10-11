//! Martensite's narrow keyboard target for native SpaceMouse dispatch.
use super::{AspectVKey, AspectWindowInputListener, AspectWindowInputListenerBase};
use rustycad_tkernel::standard::Failure;
/// Keyboard callbacks needed by HID navigation, independent of window/mouse callbacks.
/// Example: implement this trait for an input queue, then call `dispatch_space_mouse_keys`.
pub trait MartensiteSpaceMouseInput {
    /// Borrow native listener settings; `target.space_mouse_state().keys.modifiers()`.
    fn space_mouse_state(&self) -> &AspectWindowInputListenerBase;
    /// Edit cached report state; `target.space_mouse_state_mut().mouse3d_is_quadric = false`.
    fn space_mouse_state_mut(&mut self) -> &mut AspectWindowInputListenerBase;
    /// Dispatch an opposing key pair; `target.space_mouse_axis(left, right, 1.0, 0.5)?`.
    fn space_mouse_axis(
        &mut self,
        negative: AspectVKey,
        positive: AspectVKey,
        time: f64,
        pressure: f64,
    ) -> Result<(), Failure>;
    /// Dispatch a button press; `target.space_mouse_press(key, 1.0)?`.
    fn space_mouse_press(&mut self, key: AspectVKey, time: f64) -> Result<(), Failure>;
    /// Dispatch a button release; `target.space_mouse_release(key, 1.0)?`.
    fn space_mouse_release(&mut self, key: AspectVKey, time: f64) -> Result<(), Failure>;
}
impl<T: AspectWindowInputListener + ?Sized> MartensiteSpaceMouseInput for T {
    fn space_mouse_state(&self) -> &AspectWindowInputListenerBase {
        self.input_base()
    }
    fn space_mouse_state_mut(&mut self) -> &mut AspectWindowInputListenerBase {
        self.input_base_mut()
    }
    fn space_mouse_axis(
        &mut self,
        negative: AspectVKey,
        positive: AspectVKey,
        time: f64,
        pressure: f64,
    ) -> Result<(), Failure> {
        self.key_from_axis(negative, positive, time, pressure)
    }
    fn space_mouse_press(&mut self, key: AspectVKey, time: f64) -> Result<(), Failure> {
        self.key_down(key, time, 1.0)
    }
    fn space_mouse_release(&mut self, key: AspectVKey, time: f64) -> Result<(), Failure> {
        self.key_up(key, time)
    }
}
