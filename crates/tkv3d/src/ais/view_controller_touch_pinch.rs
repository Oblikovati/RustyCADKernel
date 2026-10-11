//! Two contacts retain origins until any native pinch component exceeds its tolerance.
use super::{AisViewController, view_controller_touch_synthesis::invalidate_touch_view};
use crate::v3d::V3dView;
use rustycad_tkernel::{
    ncollection::Vec2,
    precision::confusion,
    standard::{Failure, Handle},
};
use rustycad_tkservice::aspect::{AspectScrollDelta, AspectTouch};
struct TouchPinchMotion {
    first: AspectTouch,
    last: AspectTouch,
    center: Vec2<f64>,
    deviation: Vec2<f64>,
    size_delta: f64,
}
impl TouchPinchMotion {
    fn new(first: AspectTouch, last: AspectTouch) -> Self {
        let center = Vec2::new(
            (first.from.x + last.from.x) / 2.0,
            (first.from.y + last.from.y) / 2.0,
        );
        let end = Vec2::new(
            (first.to.x + last.to.x) / 2.0,
            (first.to.y + last.to.y) / 2.0,
        );
        Self {
            first,
            last,
            center,
            deviation: end - center,
            size_delta: (first.to - last.to).modulus() - (first.from - last.from).modulus(),
        }
    }
    fn rotation_angle(&self) -> f64 {
        let a1 = self.first.from.y - self.last.from.y;
        let b1 = self.last.from.x - self.first.from.x;
        let a2 = self.first.to.y - self.last.to.y;
        let b2 = self.last.to.x - self.first.to.x;
        let denominator = a1 * a2 + b1 * b2;
        if denominator <= confusion() {
            return 0.0;
        }
        ((a1 * b2 - a2 * b1) / denominator).atan()
    }
}
impl AisViewController {
    pub(super) fn synthesize_touch_pinch(&mut self, view: &Handle<V3dView>) -> Result<(), Failure> {
        let touches = &self.input_base.touch_points;
        let motion = TouchPinchMotion::new(
            *touches.find_from_index(1).expect("two touches"),
            *touches.find_from_index(2).expect("two touches"),
        );
        let rotation = self.queue_pinch_z_rotation(&motion);
        let zoom = self.queue_touch_pinch_zoom(&motion);
        let pan = self.queue_touch_pinch_pan(&motion, view)?;
        if rotation || zoom || pan {
            self.advance_synthesized_touch_origins(2);
        }
        Ok(())
    }
    fn queue_pinch_z_rotation(&mut self, motion: &TouchPinchMotion) -> bool {
        if !self.to_allow_touch_z_rotation() {
            return false;
        }
        let angle = motion.rotation_angle();
        if angle.abs() > f64::from(self.touch_dispatch.z_rotation_threshold) {
            self.render_input.z_rotate.to_rotate = true;
            self.render_input.z_rotate.angle = angle;
            return true;
        }
        false
    }
    fn queue_touch_pinch_zoom(&mut self, motion: &TouchPinchMotion) -> bool {
        let tolerance =
            f64::from(self.touch_tolerance_scale()) * f64::from(self.mouse_dispatch.zoom_threshold);
        if motion.size_delta.abs() > tolerance {
            let delta = motion.size_delta * f64::from(self.touch_dispatch.zoom_ratio);
            let point = Vec2::new(motion.center.x as i32, motion.center.y as i32);
            self.render_input
                .zoom_actions
                .append(AspectScrollDelta::new(point, delta, 0));
            return true;
        }
        false
    }
    fn queue_touch_pinch_pan(
        &mut self,
        motion: &TouchPinchMotion,
        view: &Handle<V3dView>,
    ) -> Result<bool, Failure> {
        let tolerance = if motion.first.is_precise_device {
            rustycad_tkmath::gp::resolution()
        } else {
            f64::from(self.touch_tolerance_scale()) * f64::from(self.mouse_dispatch.pan_threshold)
        };
        if motion.deviation.x.abs() + motion.deviation.y.abs() > tolerance {
            self.start_touch_pinch_pan(view)?;
            self.render_input.panning.to_pan = true;
            self.render_input.panning.delta =
                Vec2::new(motion.deviation.x as i32, (-motion.deviation.y) as i32);
            return Ok(true);
        }
        Ok(false)
    }
    fn start_touch_pinch_pan(&mut self, view: &Handle<V3dView>) -> Result<(), Failure> {
        if !self.touch_dispatch.update_start_pan {
            return Ok(());
        }
        self.render_input.panning.to_start = true;
        let start = self.touch_dispatch.start_pan;
        self.render_input.panning.point_start = Vec2::new(start.x as i32, start.y as i32);
        self.touch_dispatch.update_start_pan = false;
        invalidate_touch_view(view)
    }
}
