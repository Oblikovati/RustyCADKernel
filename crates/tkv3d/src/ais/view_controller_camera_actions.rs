//! Full camera-action sequencing preserves native flags, presentation callbacks and failures.
use super::{
    AisInteractiveContext, AisViewController, AisWalkDelta,
    view_controller_camera_feedback::CameraActionFeedback,
    view_controller_lazy_move::camera_action_view,
};
use crate::v3d::V3dView;
use crate::v3d::V3dViewSource;
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
};
impl AisViewController {
    /// Dispatch camera requests and actual scene feedback; `controller.handle_camera_actions(&context,&view,&walk)?`.
    pub fn handle_camera_actions(
        &mut self,
        context: &AisInteractiveContext,
        view: &Handle<dyn V3dViewSource>,
        walk: &AisWalkDelta,
    ) -> Result<(), Failure> {
        rustycad_tkernel::standard::catch_native_failure(|| {
            self.dispatch_camera_actions(context, view, walk)
        })
    }
    fn dispatch_camera_actions(
        &mut self,
        context: &AisInteractiveContext,
        view: &Handle<dyn V3dViewSource>,
        walk: &AisWalkDelta,
    ) -> Result<(), Failure> {
        let actual = camera_action_view(view)?;
        let mut feedback = self.camera_action_feedback(actual, walk)?;
        self.apply_camera_orientation(actual, &mut feedback)?;
        recompute_camera_action_objects(context, actual)?;
        self.restore_camera_action_gesture(context, actual)?;
        self.dispatch_camera_action_pan(context, actual, &mut feedback)?;
        self.dispatch_camera_action_orbit(context, actual, &mut feedback)?;
        self.dispatch_camera_action_view(actual, walk, &mut feedback)?;
        self.dispatch_camera_action_zoom(context, actual, &mut feedback)?;
        self.finish_camera_action_feedback(context, view, &feedback)
    }
    fn apply_camera_orientation(
        &mut self,
        view: &V3dView,
        feedback: &mut CameraActionFeedback,
    ) -> Result<(), Failure> {
        let request = &mut self.render_input.orientation;
        if request.to_set_view_orient {
            view.set_proj_orientation(request.view_orient, false)?;
            request.to_fit_all = true;
            feedback.update_cursor = feedback.has_shader_echo;
        }
        if request.to_fit_all {
            view.fit_all(0.01, false)?;
            view.invalidate();
            request.to_fit_all = false;
            feedback.update_cursor = feedback.has_shader_echo;
        }
        Ok(())
    }
    fn restore_camera_action_gesture(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
    ) -> Result<(), Failure> {
        if !self.render_input.is_new_gesture {
            return Ok(());
        }
        self.remove_camera_action_anchor(context, view, &self.camera_action_state.rotate_anchor)?;
        self.remove_camera_action_anchor(context, view, &self.camera_action_state.pan_anchor)?;
        if self.camera_action_state.had_computed_hlr {
            self.camera_action_state.had_computed_hlr = false;
            view.set_computed_mode(true);
            view.invalidate();
        }
        Ok(())
    }
    pub(super) fn suspend_camera_action_hlr(&mut self, view: &V3dView, start: bool) {
        if !start || self.camera_action_state.had_computed_hlr {
            return;
        }
        self.camera_action_state.had_computed_hlr = view.computed_mode();
        if self.camera_action_state.had_computed_hlr {
            view.set_computed_mode(false);
        }
    }
}
fn recompute_camera_action_objects(
    context: &AisInteractiveContext,
    view: &V3dView,
) -> Result<(), Failure> {
    let mut objects = List::default();
    context.displayed_objects(&mut objects)?;
    for object in &objects {
        object.recompute_transformation(&view.camera());
    }
    Ok(())
}
