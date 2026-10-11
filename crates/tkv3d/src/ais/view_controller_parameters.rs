//! Native controller policy values; OCCT 3d097a0 does not clamp setters.
use super::{AisRotationMode, AisViewController};
macro_rules! controller_parameters {
    ($(($getter:ident, $setter:ident, $kind:ty, $initial:expr)),* $(,)?) => {
        pub(super) struct ViewControllerParameters { $(pub(super) $getter: $kind),* }
        impl Default for ViewControllerParameters {
            fn default() -> Self { Self { $($getter: $initial),* } }
        }
        impl AisViewController {
            $(
                #[doc = concat!("Read native input policy; `controller.", stringify!($getter), "()`.")]
                pub fn $getter(&self) -> $kind { self.parameters.$getter }
                #[doc = concat!("Set native input policy without clamping; `controller.", stringify!($setter), "(value)`.")]
                pub fn $setter(&mut self, value: $kind) { self.parameters.$getter = value; }
            )*
        }
    };
}
controller_parameters! {
    (to_pause_objects_animation, set_pause_objects_animation, bool, false),
    (is_continuous_redraw, set_continuous_redraw, bool, false),
    (to_show_pan_anchor_point, set_show_pan_anchor_point, bool, true),
    (to_show_rotate_center, set_show_rotate_center, bool, true),
    (to_lock_orbit_z_up, set_lock_orbit_z_up, bool, false),
    (to_allow_touch_z_rotation, set_allow_touch_z_rotation, bool, false),
    (to_allow_rotation, set_allow_rotation, bool, true),
    (to_allow_panning, set_allow_panning, bool, true),
    (to_allow_zooming, set_allow_zooming, bool, true),
    (to_allow_z_focus, set_allow_z_focus, bool, true),
    (to_allow_highlight, set_allow_highlight, bool, true),
    (to_allow_dragging, set_allow_dragging, bool, true),
    (to_stick_to_ray_on_zoom, set_stick_to_ray_on_zoom, bool, true),
    (to_stick_to_ray_on_rotation, set_stick_to_ray_on_rotation, bool, true),
    (to_invert_pitch, set_invert_pitch, bool, false),
    (to_display_xr_aux_devices, set_display_xr_aux_devices, bool, false),
    (to_display_xr_hands, set_display_xr_hands, bool, true),
    (mouse_acceleration, set_mouse_acceleration, f32, 1.0),
    (orbit_acceleration, set_orbit_acceleration, f32, 1.0),
    (walk_speed_absolute, set_walk_speed_absolute, f32, 1.5),
    (walk_speed_relative, set_walk_speed_relative, f32, 0.1),
    (thrust_speed, set_thrust_speed, f32, 0.0),
    (touch_tolerance_scale, set_touch_tolerance_scale, f32, 1.0),
    (mouse_double_click_interval, set_mouse_double_click_interval, f64, 0.4),
    (min_zoom_distance, set_min_zoom_distance, f64, 1.0),
    (rotation_mode, set_rotation_mode, AisRotationMode, AisRotationMode::BndBoxActive),
}
