//! Native controller HID dispatch using Martensite's fallible keyboard seam.
use super::AisViewController;
use rustycad_tkernel::standard::Failure;
use rustycad_tkservice::{
    aspect::{
        AspectVKey, AspectWindowInputListenerBase, MartensiteSpaceMouseInput,
        dispatch_space_mouse_keys, dispatch_space_mouse_rotation, dispatch_space_mouse_translation,
    },
    wnt::WntHidSpaceMouse,
};
impl AisViewController {
    /// Queue translation, allowed rotation and HID buttons in native order; `controller.update_3d_mouse(&report)?`.
    pub fn update_3d_mouse(&mut self, event: &WntHidSpaceMouse<'_>) -> Result<bool, Failure> {
        let translation = dispatch_space_mouse_translation(self, event)?;
        let rotation = self.to_allow_rotation() && dispatch_space_mouse_rotation(self, event)?;
        let keys = dispatch_space_mouse_keys(self, event)?;
        Ok(translation || rotation || keys)
    }
}
impl MartensiteSpaceMouseInput for AisViewController {
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
