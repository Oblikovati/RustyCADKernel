//! OCCT 3d097a0 merges only adjacent zooms at the same pixel, preserving the first modifiers.
use super::AisViewController;
use rustycad_tkservice::aspect::AspectScrollDelta;
impl AisViewController {
    /// Queue or merge a zoom, returning true only for a new action; `controller.update_zoom(&delta)`.
    pub fn update_zoom(&mut self, delta: &AspectScrollDelta) -> bool {
        if let Some(last) = self.ui_input.zoom_actions.last_mut()
            && last.point == delta.point
        {
            last.delta += delta.delta;
            return false;
        }
        self.ui_input.zoom_actions.append(*delta);
        true
    }
    /// Scale wheel input by the native factor 15; `controller.update_mouse_scroll(&delta)`.
    pub fn update_mouse_scroll(&mut self, delta: &AspectScrollDelta) -> bool {
        let mut scaled = *delta;
        scaled.delta *= f64::from(super::view_controller_fields::SCROLL_ZOOM_RATIO);
        self.update_zoom(&scaled)
    }
    /// Queue a touch Z rotation only when allowed; `controller.update_z_rotation(0.25)`.
    pub fn update_z_rotation(&mut self, angle: f64) -> bool {
        if !self.to_allow_touch_z_rotation() {
            return false;
        }
        let rotation = &mut self.ui_input.z_rotate;
        rotation.angle = if rotation.to_rotate {
            rotation.angle + angle
        } else {
            angle
        };
        let created = !rotation.to_rotate;
        rotation.to_rotate = true;
        created
    }
}
