//! Native pan anchors use start-pixel picking and center fallback before immediate camera motion.
use super::{
    AisInteractiveContext, AisInteractiveObjectMembers, AisNavigationMode, AisViewController,
    view_controller_camera_feedback::CameraActionFeedback,
};
use crate::v3d::V3dView;
use rustycad_tkernel::{
    ncollection::Vec2,
    precision::{infinite, is_infinite},
    standard::Failure,
};
use rustycad_tkmath::gp::Point3;
impl AisViewController {
    pub(super) fn dispatch_camera_action_pan(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
        feedback: &mut CameraActionFeedback,
    ) -> Result<(), Failure> {
        if self.navigation_mode() == AisNavigationMode::FirstPersonWalk {
            return Ok(());
        }
        self.start_camera_action_pan(context, view)?;
        self.show_camera_action_pan(context)?;
        self.handle_panning(view)?;
        self.handle_z_rotate(view)?;
        self.retain_camera_pan_feedback(feedback);
        Ok(())
    }
    fn start_camera_action_pan(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
    ) -> Result<(), Failure> {
        if !self.render_input.panning.to_start || !self.to_allow_panning() {
            return Ok(());
        }
        let point = self.new_camera_action_pan_point(context, view)?;
        self.set_panning_anchor_point(&point);
        Ok(())
    }
    fn pick_camera_action_pan_anchor(
        &mut self,
        point: &mut Point3,
        context: &AisInteractiveContext,
        view: &V3dView,
    ) -> Result<(), Failure> {
        let pixel = self.render_input.panning.point_start;
        if pixel.x >= 0 && pixel.y >= 0 {
            self.pick_point(point, context, view, &pixel, false)?;
        }
        if is_infinite(point.x()) {
            let center = camera_action_window_center(view)?;
            self.pick_point(point, context, view, &center, false)?;
        }
        Ok(())
    }
    fn show_camera_action_pan(&self, context: &AisInteractiveContext) -> Result<(), Failure> {
        let anchor = &self.camera_action_state.pan_anchor;
        let visible = self.to_show_pan_anchor_point()
            && self.has_panning_anchor_point()
            && self.render_input.panning.to_pan;
        if visible && !self.render_input.is_new_gesture && !anchor.has_interactive_context() {
            self.display_camera_action_anchor(context, anchor)?;
        }
        Ok(())
    }
}
pub(super) fn camera_action_window_center(view: &V3dView) -> Result<Vec2<i32>, Failure> {
    let (width, height) = super::view_controller_camera_primitives::primitive_window_size(view)?;
    Ok(Vec2::new(width / 2, height / 2))
}

impl AisViewController {
    fn retain_camera_pan_feedback(&self, feedback: &mut CameraActionFeedback) {
        if feedback.has_shader_echo && self.render_input.panning.to_pan && self.to_allow_panning() {
            feedback.update_cursor = true;
        }
        if feedback.has_shader_echo
            && self.render_input.z_rotate.to_rotate
            && self.to_allow_rotation()
        {
            feedback.hide_echo = true;
        }
    }
}

impl AisViewController {
    fn new_camera_action_pan_point(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
    ) -> Result<Point3, Failure> {
        let mut point = Point3::new(infinite(), 0.0, 0.0);
        if !view.camera().is_orthographic() {
            self.pick_camera_action_pan_anchor(&mut point, context, view)?;
            if !is_infinite(point.x()) && self.to_show_pan_anchor_point() {
                self.locate_camera_action_anchor(
                    context,
                    &self.camera_action_state.pan_anchor,
                    point,
                )?;
            }
        }
        Ok(point)
    }
}
