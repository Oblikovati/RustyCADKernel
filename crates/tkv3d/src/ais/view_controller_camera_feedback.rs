//! Camera feedback decisions retain the pre-operation mouse-rotation predicate.
use super::{AisViewController, AisWalkDelta, view_controller_lazy_move::camera_action_view};
use crate::v3d::V3dView;
use crate::v3d::V3dViewSource;
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Failure, Handle},
};
pub(super) struct CameraActionFeedback {
    pub(super) has_echo: bool,
    pub(super) has_shader_echo: bool,
    pub(super) cursor: Vec2<i32>,
    pub(super) update_cursor: bool,
    pub(super) hide_echo: bool,
    pub(super) mouse_rotation: bool,
}
impl AisViewController {
    pub(super) fn camera_action_feedback(
        &self,
        view: &V3dView,
        walk: &AisWalkDelta,
    ) -> Result<CameraActionFeedback, Failure> {
        let echo = view.is_grid_active()? && view.viewer().grid_echo();
        let shader = echo && view.is_shader_grid_active();
        let mouse = self.camera_action_is_mouse_rotation();
        let cursor = self.camera_action_feedback_cursor();
        Ok(CameraActionFeedback {
            has_echo: echo,
            has_shader_echo: shader,
            cursor,
            update_cursor: shader && !walk.is_empty(),
            hide_echo: false,
            mouse_rotation: mouse,
        })
    }
    pub(super) fn finish_camera_action_feedback(
        &mut self,
        context: &super::AisInteractiveContext,
        view: &Handle<dyn V3dViewSource>,
        feedback: &CameraActionFeedback,
    ) -> Result<(), Failure> {
        let actual = camera_action_view(view)?;
        if feedback.hide_echo {
            actual.viewer().hide_grid_echo(view);
            actual.invalidate_immediate();
            self.reset_previous_move_to();
            return Ok(());
        }
        if !feedback.update_cursor {
            return Ok(());
        }
        self.update_camera_action_cursor(context, view, feedback)
    }
}

impl AisViewController {
    fn camera_action_is_mouse_rotation(&self) -> bool {
        let b = &self.render_input;
        (b.orbit_rotation.to_rotate || b.view_rotation.to_rotate || b.z_rotate.to_rotate)
            && self.to_allow_rotation()
    }
    fn camera_action_feedback_cursor(&self) -> Vec2<i32> {
        if self.has_previous_move_to() {
            return *self.previous_move_to();
        }
        self.last_mouse_position()
    }
    fn update_camera_action_cursor(
        &mut self,
        context: &super::AisInteractiveContext,
        view: &Handle<dyn V3dViewSource>,
        feedback: &CameraActionFeedback,
    ) -> Result<(), Failure> {
        let actual = camera_action_view(view)?;
        let (width, height) =
            super::view_controller_camera_primitives::primitive_window_size(actual)?;
        let pixel = feedback.cursor;
        if pixel.x >= 0 && pixel.x < width && pixel.y >= 0 && pixel.y < height {
            self.reset_previous_move_to();
            self.context_lazy_move_to(context, view, &pixel)?;
        }
        Ok(())
    }
}
