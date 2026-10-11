//! Native controller HID dispatch using Martensite's fallible keyboard seam.
use super::AisViewController;
use rustycad_tkernel::standard::Failure;
use rustycad_tkservice::{
    aspect::{
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
