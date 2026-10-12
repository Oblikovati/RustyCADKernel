//! Single contact uses native mode acceleration and precise-device tolerance.
use super::view_controller_touch_synthesis::invalidate_touch_view;
use super::{AisNavigationMode, AisViewController};
use crate::v3d::V3dViewSource;
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Failure, Handle},
};
use rustycad_tkservice::aspect::AspectTouch;
impl AisViewController {
    pub(super) fn synthesize_touch_rotation(
        &mut self,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        self.start_touch_rotation(view)?;
        let touch = *self
            .input_base
            .touch_points
            .find_from_index(1)
            .expect("one touch");
        let tolerance = self.touch_rotation_tolerance(&touch);
        if touch.delta().x.abs() + touch.delta().y.abs() > tolerance {
            self.queue_touch_rotation(touch.to);
            self.advance_synthesized_touch_origins(1);
        }
        Ok(())
    }
    fn start_touch_rotation(&mut self, view: &Handle<dyn V3dViewSource>) -> Result<(), Failure> {
        if !self.touch_dispatch.update_start_rotation {
            return Ok(());
        }
        if self.navigation_mode == AisNavigationMode::Orbit {
            self.render_input.orbit_rotation.to_start = true;
            self.render_input.orbit_rotation.point_start = self.touch_dispatch.start_rotation;
        } else {
            self.render_input.view_rotation.to_start = true;
            self.render_input.view_rotation.point_start = self.touch_dispatch.start_rotation;
        }
        self.touch_dispatch.update_start_rotation = false;
        invalidate_touch_view(view)
    }
    fn touch_rotation_tolerance(&self, touch: &AspectTouch) -> f64 {
        if touch.is_precise_device {
            return rustycad_tkmath::gp::resolution();
        }
        f64::from(self.touch_tolerance_scale()) * f64::from(self.mouse_dispatch.rotation_threshold)
    }
    fn queue_touch_rotation(&mut self, point: Vec2<f64>) {
        let acceleration = self.touch_rotation_acceleration();
        if self.navigation_mode == AisNavigationMode::Orbit {
            let rotation = &mut self.render_input.orbit_rotation;
            rotation.to_rotate = true;
            rotation.point_to =
                rotation.point_start + (point - rotation.point_start) * acceleration;
        } else {
            let rotation = &mut self.render_input.view_rotation;
            rotation.to_rotate = true;
            rotation.point_to =
                rotation.point_start + (point - rotation.point_start) * acceleration;
        }
        self.render_input.dragging.to_move = true;
        self.render_input.dragging.point_to = Vec2::new(point.x as i32, point.y as i32);
    }
    fn touch_rotation_acceleration(&self) -> f64 {
        if self.navigation_mode == AisNavigationMode::FirstPersonWalk {
            return f64::from(self.mouse_acceleration());
        }
        f64::from(self.orbit_acceleration())
    }
}
