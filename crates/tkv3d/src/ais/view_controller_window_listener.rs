//! The complete fallible window listener routes events through the native controller paths.
use super::AisViewController;
use rustycad_tkernel::{ncollection::Vec2, standard::Failure};
use rustycad_tkservice::{
    aspect::{
        AspectScrollDelta, AspectVKey, AspectVKeyFlags, AspectVKeyMouse, AspectWindowInputListener,
        AspectWindowInputListenerBase,
    },
    wnt::WntHidSpaceMouse,
};
impl AspectWindowInputListener for AisViewController {
    fn input_base(&self) -> &AspectWindowInputListenerBase {
        AisViewController::input_base(self)
    }
    fn input_base_mut(&mut self) -> &mut AspectWindowInputListenerBase {
        AisViewController::input_base_mut(self)
    }
    fn process_expose(&mut self) {
        AisViewController::process_expose(self)
    }
    fn process_configure(&mut self, is_resized: bool) {
        AisViewController::process_configure(self, is_resized)
    }
    fn process_input(&mut self) {
        AisViewController::process_input(self)
    }
    fn process_focus(&mut self, is_activated: bool) {
        AisViewController::process_focus(self, is_activated)
    }
    fn process_close(&mut self) {
        AisViewController::process_close(self)
    }
    fn key_down(&mut self, key: AspectVKey, time: f64, pressure: f64) -> Result<(), Failure> {
        AisViewController::key_down(self, key, time, pressure)
    }
    fn key_up(&mut self, key: AspectVKey, time: f64) -> Result<(), Failure> {
        AisViewController::key_up(self, key, time)
    }
    fn key_from_axis(
        &mut self,
        negative: AspectVKey,
        positive: AspectVKey,
        time: f64,
        pressure: f64,
    ) -> Result<(), Failure> {
        AisViewController::key_from_axis(self, negative, positive, time, pressure)
    }
    fn update_mouse_scroll(&mut self, delta: &AspectScrollDelta) -> bool {
        AisViewController::update_mouse_scroll(self, delta)
    }
    fn update_mouse_buttons(
        &mut self,
        point: Vec2<i32>,
        buttons: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        is_emulated: bool,
    ) -> Result<bool, Failure> {
        AisViewController::update_mouse_buttons(self, point, buttons, modifiers, is_emulated)
    }
    fn update_mouse_position(
        &mut self,
        point: Vec2<i32>,
        buttons: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        is_emulated: bool,
    ) -> Result<bool, Failure> {
        AisViewController::update_mouse_position(self, point, buttons, modifiers, is_emulated)
    }
    fn update_3d_mouse(&mut self, event: &WntHidSpaceMouse<'_>) -> Result<bool, Failure> {
        AisViewController::update_3d_mouse(self, event)
    }
    fn add_touch_point(
        &mut self,
        id: usize,
        point: Vec2<f64>,
        clear_before: bool,
    ) -> Result<(), Failure> {
        AisViewController::add_touch_point(self, id, point, clear_before)
    }
    fn remove_touch_point(
        &mut self,
        id: usize,
        clear_select_points: bool,
    ) -> Result<bool, Failure> {
        AisViewController::remove_touch_point(self, id, clear_select_points)
    }
    fn update_touch_point(&mut self, id: usize, point: Vec2<f64>) -> Result<(), Failure> {
        AisViewController::update_touch_point(self, id, point)
    }
}
