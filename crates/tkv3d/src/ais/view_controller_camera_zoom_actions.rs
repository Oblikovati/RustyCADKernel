//! Scroll consumption occurs only after every actual zoom/focus operation succeeds.
use super::{
    AisInteractiveContext, AisViewController,
    view_controller_camera_feedback::CameraActionFeedback,
    view_controller_camera_pan::camera_action_window_center,
};
use crate::v3d::V3dView;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::{ASPECT_VKEY_FLAGS_CTRL, AspectScrollDelta};
impl AisViewController {
    pub(super) fn dispatch_camera_action_zoom(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
        feedback: &mut CameraActionFeedback,
    ) -> Result<(), Failure> {
        if self.render_input.zoom_actions.is_empty() {
            return Ok(());
        }
        let actions: Vec<AspectScrollDelta> =
            self.render_input.zoom_actions.iter().copied().collect();
        for scroll in actions {
            self.apply_camera_action_scroll(context, view, scroll, feedback)?;
        }
        // Retain the entire queue on failure, including earlier successfully applied scrolls.
        self.render_input.zoom_actions.clear(None);
        Ok(())
    }
    fn apply_camera_action_scroll(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
        mut scroll: AspectScrollDelta,
        feedback: &mut CameraActionFeedback,
    ) -> Result<(), Failure> {
        if self.is_camera_action_focus_scroll(view, &scroll) {
            return self.handle_z_focus_scroll(view, &scroll);
        }
        if !self.to_allow_zooming() {
            return Ok(());
        }
        retain_camera_zoom_cursor(feedback, &scroll);
        let hit = self.pick_camera_action_zoom(context, view, &mut scroll)?;
        self.handle_zoom(view, &scroll, hit.as_ref())?;
        feedback.update_cursor = feedback.has_echo;
        Ok(())
    }
    fn pick_camera_action_zoom(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
        scroll: &mut AspectScrollDelta,
    ) -> Result<Option<Point3>, Failure> {
        if view.camera().is_orthographic() {
            return Ok(None);
        }
        let mut point = Point3::default();
        let stick = self.to_stick_to_ray_on_zoom();
        if self.pick_camera_action_scroll_point(&mut point, context, view, scroll, stick)? {
            return Ok(Some(point));
        }
        self.pick_camera_action_zoom_center(context, view, scroll, &mut point, stick)
    }
    fn pick_camera_action_zoom_center(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
        scroll: &mut AspectScrollDelta,
        point: &mut Point3,
        stick: bool,
    ) -> Result<Option<Point3>, Failure> {
        let center = camera_action_window_center(view)?;
        if self.pick_point(point, context, view, &center, stick)? {
            scroll.reset_point();
            return Ok(Some(*point));
        }
        Ok(None)
    }
}

impl AisViewController {
    fn is_camera_action_focus_scroll(&self, view: &V3dView, scroll: &AspectScrollDelta) -> bool {
        let control = scroll.flags & ASPECT_VKEY_FLAGS_CTRL != 0;
        self.to_allow_z_focus() && control && view.camera().is_stereo()
    }
    fn pick_camera_action_scroll_point(
        &mut self,
        point: &mut Point3,
        context: &AisInteractiveContext,
        view: &V3dView,
        scroll: &AspectScrollDelta,
        stick: bool,
    ) -> Result<bool, Failure> {
        if !scroll.has_point() {
            return Ok(false);
        }
        self.pick_point(point, context, view, &scroll.point, stick)
    }
}

fn retain_camera_zoom_cursor(feedback: &mut CameraActionFeedback, scroll: &AspectScrollDelta) {
    if scroll.has_point() {
        feedback.cursor = scroll.point;
    }
}
