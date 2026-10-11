//! Rotation orchestration owns HLR transitions and actual rotation-center presentations.
use super::{
    AisInteractiveContext, AisInteractiveObjectMembers, AisNavigationMode, AisViewController,
    AisWalkDelta, AisWalkRotation, view_controller_camera_feedback::CameraActionFeedback,
};
use crate::v3d::V3dView;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use std::f64::consts::PI;
impl AisViewController {
    pub(super) fn dispatch_camera_action_orbit(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
        feedback: &mut CameraActionFeedback,
    ) -> Result<(), Failure> {
        if !self.wants_camera_action_orbit() {
            return Ok(());
        }
        self.suspend_camera_action_hlr(view, self.render_input.orbit_rotation.to_start);
        let pivot = self.start_camera_action_orbit(context, view)?;
        self.show_camera_action_orbit(context)?;
        let lock = self.to_lock_orbit_z_up() || self.navigation_mode() != AisNavigationMode::Orbit;
        self.handle_orbit_rotation(view, &pivot, lock)?;
        self.retain_camera_rotation_feedback(feedback, self.render_input.orbit_rotation.to_rotate);
        Ok(())
    }
    fn start_camera_action_orbit(
        &mut self,
        context: &AisInteractiveContext,
        view: &V3dView,
    ) -> Result<Point3, Failure> {
        if !self.render_input.orbit_rotation.to_start {
            return Ok(Point3::default());
        }
        let pivot = self.gravity_point(context, view)?;
        if self.to_show_rotate_center() {
            self.locate_camera_action_anchor(
                context,
                &self.camera_action_state.rotate_anchor,
                pivot,
            )?;
            self.locate_camera_action_anchor(context, &self.camera_action_state.pan_anchor, pivot)?;
        }
        Ok(pivot)
    }
    fn show_camera_action_orbit(&self, context: &AisInteractiveContext) -> Result<(), Failure> {
        let anchor = &self.camera_action_state.rotate_anchor;
        let show = self.to_show_rotate_center()
            && self.render_input.orbit_rotation.to_rotate
            && !self.render_input.is_new_gesture;
        if show && !anchor.has_interactive_context() {
            self.display_camera_action_anchor(context, anchor)?;
            self.display_camera_action_anchor(context, &self.camera_action_state.pan_anchor)?;
        }
        Ok(())
    }
    pub(super) fn dispatch_camera_action_view(
        &mut self,
        view: &V3dView,
        walk: &AisWalkDelta,
        feedback: &mut CameraActionFeedback,
    ) -> Result<(), Failure> {
        let request = self.render_input.view_rotation;
        let active = self.navigation_mode() != AisNavigationMode::Orbit
            || request.to_start
            || request.to_rotate;
        if !active || !self.to_allow_rotation() {
            return Ok(());
        }
        self.suspend_camera_action_hlr(view, request.to_start);
        self.apply_camera_action_walk_rotation(view, walk)?;
        self.retain_camera_rotation_feedback(feedback, self.render_input.view_rotation.to_rotate);
        Ok(())
    }
}
fn camera_action_roll(walk: &AisWalkDelta, locked: bool) -> f64 {
    let part = walk[AisWalkRotation::Roll];
    if part.is_empty() || locked {
        return 0.0;
    }
    let duration = 1000.0 * part.duration;
    let duration = if 100.0 < duration { 100.0 } else { duration };
    let roll = (PI / 12.0) * part.pressure * (duration / 100.0);
    if part.value < 0.0 { -roll } else { roll }
}

impl AisViewController {
    fn retain_camera_rotation_feedback(&self, feedback: &mut CameraActionFeedback, rotating: bool) {
        if feedback.has_shader_echo && rotating && self.to_allow_rotation() {
            feedback.hide_echo = feedback.mouse_rotation;
        }
    }
    fn wants_camera_action_orbit(&self) -> bool {
        let request = self.render_input.orbit_rotation;
        (self.navigation_mode() == AisNavigationMode::Orbit
            || request.to_start
            || request.to_rotate)
            && self.to_allow_rotation()
    }
    fn apply_camera_action_walk_rotation(
        &mut self,
        view: &V3dView,
        walk: &AisWalkDelta,
    ) -> Result<(), Failure> {
        let roll = camera_action_roll(walk, self.to_lock_orbit_z_up());
        let flight = self.navigation_mode() == AisNavigationMode::FirstPersonFlight;
        self.handle_view_rotation(
            view,
            walk[AisWalkRotation::Yaw].value,
            walk[AisWalkRotation::Pitch].value,
            roll,
            flight,
        )
    }
}
