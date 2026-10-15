//! `AIS_ViewController` XR input: trackpad turns and teleports, trigger picking with haptic
//! feedback, and the tracked-device presentations with their lasers.
use super::view_controller_lazy_move::camera_action_view;
use super::*;
use crate::prs_mgr::{
    PrsMgrDisplayStatus, PrsMgrPresentableObject, PrsMgrPresentableObjectMembers,
};
use crate::select_mgr::*;
use crate::v3d::V3dViewSource;
use rustycad_tkernel::precision::{infinite, is_infinite};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{self, Axis1, Point3, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::aspect::{
    AspectXRGenericAction, AspectXRSession, AspectXRSessionMembers, AspectXRTrackedDeviceRole,
    AspectXrActionRef,
};
use rustycad_tkservice::graphic3d::{Graphic3dCView, Graphic3dCViewMembers, Graphic3dCViewXr};
use rustycad_tkservice::image::ImageTexture;

const HANDS: [AspectXRTrackedDeviceRole; 2] = [
    AspectXRTrackedDeviceRole::RightHand,
    AspectXRTrackedDeviceRole::LeftHand,
];

fn null_session() -> Failure {
    Failure::with_message("Standard_NullObject", "Graphic3d_CView: null XR session")
}

impl AisViewController {
    /// `handleXRInput`: process the XR input of the view, then (when XR is active) trackpad
    /// turns, teleports and picking; `controller.handle_xr_input(&context, &view, &walk)?`.
    pub fn handle_xr_input(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
        walk: &AisWalkDelta,
    ) -> Result<(), Failure> {
        let _ = walk;
        let actual = camera_action_view(view)?;
        actual.view().process_xr_input()?;
        if !actual.view().is_active_xr() {
            return Ok(());
        }
        self.handle_xr_turn_pad(context, view)?;
        self.handle_xr_teleport(context, view)?;
        self.handle_xr_picking(context, view)
    }

    /// `handleXRTurnPad`: a sideways trackpad click turns the XR camera by the turn angle
    /// around its up direction; `controller.handle_xr_turn_pad(&context, &view)?`.
    pub fn handle_xr_turn_pad(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let _ = context;
        let actual = camera_action_view(view)?;
        let cview = actual.view();
        if self.xr.turn_angle <= 0.0 || !cview.is_active_xr() {
            return Ok(());
        }
        let session_handle = cview.xr_session();
        let session = session_handle.get().ok_or_else(null_session)?;
        for role in HANDS {
            let click = session.generic_action(role, AspectXRGenericAction::InputTrackPadClick);
            let position =
                session.generic_action(role, AspectXRGenericAction::InputTrackPadPosition);
            if click.is_null() || position.is_null() {
                continue;
            }
            let click = session.get_digital_action_data(&AspectXrActionRef::borrowed(&click))?;
            let position =
                session.get_analog_action_data(&AspectXrActionRef::borrowed(&position))?;
            if click.is_active
                && click.is_pressed
                && click.is_changed
                && position.is_active
                && position.vec_xyz.y().abs() < 0.5
                && position.vec_xyz.x().abs() > 0.7
            {
                let up = cview.base_xr_camera().get().ok_or_else(null_session)?.up();
                let mut turn = Transform3::default();
                let angle = if position.vec_xyz.x() < 0.0 {
                    self.xr.turn_angle
                } else {
                    -self.xr.turn_angle
                };
                turn.set_rotation(Axis1::new(gp::origin(), up), angle)?;
                cview.turn_view_xr_camera(&turn)?;
                break;
            }
        }
        Ok(())
    }

    /// `handleXRTeleport`: a forward trackpad press aims a teleport laser (picking its
    /// destination depth); releasing it moves the eye there (onto a floor or against a
    /// wall), with haptic feedback on hand changes.
    /// Example: `controller.handle_xr_teleport(&context, &view)?`.
    pub fn handle_xr_teleport(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let actual = camera_action_view(view)?;
        let cview = actual.view();
        if !cview.is_active_xr() {
            return Ok(());
        }
        let session_handle = cview.xr_session();
        let session = session_handle.get().ok_or_else(null_session)?;
        let tele_old = self.xr.last_teleport_hand;
        self.xr.last_teleport_hand = AspectXRTrackedDeviceRole::Other;
        for role in HANDS {
            let device = session.named_tracked_device(role);
            if device == -1 {
                continue;
            }
            let click = session.generic_action(role, AspectXRGenericAction::InputTrackPadClick);
            let position =
                session.generic_action(role, AspectXRGenericAction::InputTrackPadPosition);
            if click.is_null() || position.is_null() {
                continue;
            }
            let click = session.get_digital_action_data(&AspectXrActionRef::borrowed(&click))?;
            let position =
                session.get_analog_action_data(&AspectXrActionRef::borrowed(&position))?;
            let is_pressed = click.is_pressed;
            let is_clicked = !click.is_pressed && click.is_changed;
            if !(click.is_active
                && (is_pressed || is_clicked)
                && position.is_active
                && position.vec_xyz.y() > 0.6
                && position.vec_xyz.x().abs() < 0.5)
            {
                continue;
            }
            let pose = tracked_pose(session, device)?;
            if !pose.is_valid_pose {
                continue;
            }
            self.xr.last_teleport_hand = role;
            *self.pick_depth_mut(role) = infinite();
            let mut pick_normal = [0.0f32; 3];
            let hand_base = cview.pose_xr_to_world(&pose.orientation)?;
            let head_height = session.head_pose().translation_part().y();
            let picked = self.handle_xr_move_to(context, view, &pose.orientation, false)?;
            if picked >= 1 {
                let selector = context.main_selector();
                let data = selector.picked_data(picked)?;
                pick_normal = [data.normal.x(), data.normal.y(), data.normal.z()];
                let square: f32 = pick_normal.iter().map(|v| v * v).sum();
                if square > f32::EPSILON {
                    *self.pick_depth_mut(role) = data
                        .point
                        .distance(&Point3::from(hand_base.translation_part()));
                }
            }
            if is_clicked {
                self.xr.last_teleport_hand = AspectXRTrackedDeviceRole::Other;
                let depth = *self.pick_depth_mut(role);
                if !is_infinite(depth) {
                    teleport_eye(
                        &*cview,
                        session,
                        &hand_base,
                        pick_normal,
                        depth,
                        head_height,
                    )?;
                }
            }
            break;
        }
        if self.xr.last_teleport_hand != tele_old {
            if tele_old != AspectXRTrackedDeviceRole::Other {
                let haptic = session.generic_action(tele_old, AspectXRGenericAction::OutputHaptic);
                if !haptic.is_null() {
                    session.abort_haptic_vibration_action(&AspectXrActionRef::borrowed(&haptic))?;
                }
            }
            let hand = self.xr.last_teleport_hand;
            if hand != AspectXRTrackedDeviceRole::Other {
                let haptic = session.generic_action(hand, AspectXRGenericAction::OutputHaptic);
                if !haptic.is_null() {
                    session.trigger_haptic_vibration_action(
                        &AspectXrActionRef::borrowed(&haptic),
                        &self.xr.teleport_haptic,
                    )?;
                }
            }
        }
        Ok(())
    }

    /// `handleXRPicking`: a pulled trigger highlights along the hand laser; its click selects
    /// the detected object (with haptic feedback); detection is cleared on hand changes.
    /// Example: `controller.handle_xr_picking(&context, &view)?`.
    pub fn handle_xr_picking(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let actual = camera_action_view(view)?;
        let cview = actual.view();
        if !cview.is_active_xr() {
            return Ok(());
        }
        let session_handle = cview.xr_session();
        let session = session_handle.get().ok_or_else(null_session)?;
        let pick_old = self.xr.last_picking_hand;
        self.xr.last_picking_hand = AspectXRTrackedDeviceRole::Other;
        for role in HANDS {
            let click = session.generic_action(role, AspectXRGenericAction::InputTriggerClick);
            let pull = session.generic_action(role, AspectXRGenericAction::InputTriggerPull);
            if click.is_null() || pull.is_null() {
                continue;
            }
            let click = session.get_digital_action_data(&AspectXrActionRef::borrowed(&click))?;
            let pull = session.get_analog_action_data(&AspectXrActionRef::borrowed(&pull))?;
            if !(pull.is_active && pull.vec_xyz.x().abs() > 0.1) {
                continue;
            }
            self.xr.last_picking_hand = role;
            self.handle_xr_highlight(context, view)?;
            if click.is_active && click.is_pressed && click.is_changed {
                context.select_detected(AisSelectionScheme::Replace)?;
                self.on_selection_changed(context, view)?;
                let haptic = session.generic_action(role, AspectXRGenericAction::OutputHaptic);
                if !haptic.is_null() {
                    session.trigger_haptic_vibration_action(
                        &AspectXrActionRef::borrowed(&haptic),
                        &self.xr.select_haptic,
                    )?;
                }
            }
            break;
        }
        if self.xr.last_picking_hand != pick_old {
            context.clear_detected(false);
        }
        Ok(())
    }

    /// `handleXRHighlight`: highlight along the laser of the picking hand (haptic feedback on
    /// a new detection) and keep the picked depth as the laser length.
    /// Example: `controller.handle_xr_highlight(&context, &view)?`.
    pub fn handle_xr_highlight(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let hand = self.xr.last_picking_hand;
        if hand != AspectXRTrackedDeviceRole::LeftHand
            && hand != AspectXRTrackedDeviceRole::RightHand
        {
            return Ok(());
        }
        let actual = camera_action_view(view)?;
        let cview = actual.view();
        let session_handle = cview.xr_session();
        let session = session_handle.get().ok_or_else(null_session)?;
        let device = session.named_tracked_device(hand);
        if device == -1 {
            return Ok(());
        }
        let pose = tracked_pose(session, device)?;
        if !pose.is_valid_pose {
            return Ok(());
        }
        let detected_old = context.detected_owner().clone();
        self.handle_xr_move_to(context, view, &pose.orientation, true)?;
        let detected = context.detected_owner().clone();
        if !detected.is_null() && detected != detected_old {
            let haptic = session.generic_action(hand, AspectXRGenericAction::OutputHaptic);
            if !haptic.is_null() {
                session.trigger_haptic_vibration_action(
                    &AspectXrActionRef::borrowed(&haptic),
                    &self.xr.picking_haptic,
                )?;
            }
        }
        *self.pick_depth_mut(hand) = infinite();
        let selector = context.main_selector();
        if selector.nb_picked() > 0 {
            let hand_base = cview.pose_xr_to_world(&pose.orientation)?;
            let picked = selector.picked_data(1)?;
            *self.pick_depth_mut(hand) = picked
                .point
                .distance(&Point3::from(hand_base.translation_part()));
        }
        Ok(())
    }

    /// `handleXRMoveTo`: pick (or highlight) along the view axis of an XR pose; 1 when
    /// something is picked (detected), else 0.
    /// Example: `controller.handle_xr_move_to(&context, &view, &pose, true)?`.
    pub fn handle_xr_move_to(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
        pose: &Transform3,
        to_highlight: bool,
    ) -> Result<i32, Failure> {
        let actual = camera_action_view(view)?;
        let axis = actual.view().view_axis_in_world(pose)?;
        if to_highlight {
            context.move_to_axis(axis, view, false)?;
            return Ok(i32::from(!context.detected_owner().is_null()));
        }
        let selector = context.main_selector();
        selector.pick_axis(axis, actual)?;
        Ok(i32::from(selector.nb_picked() >= 1))
    }

    /// `handleXRPresentations`: display a tracked-device presentation per valid pose (hands
    /// and auxiliary devices as configured) placed in the world, with the picking or
    /// teleport laser; everything is removed when XR is inactive.
    /// Example: `controller.handle_xr_presentations(&context, &view)?`.
    pub fn handle_xr_presentations(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let actual = camera_action_view(view)?;
        let cview = actual.view();
        let (show_aux, show_hands) = (
            self.parameters.to_display_xr_aux_devices,
            self.parameters.to_display_xr_hands,
        );
        if !cview.is_active_xr() || (!show_aux && !show_hands) {
            for device in &mut self.xr.prs_devices {
                if !device.is_null() && device.has_interactive_context() {
                    context.remove(&device.clone().into_cast(), false)?;
                }
                *device = Handle::default();
            }
            return Ok(());
        }
        let session_handle = cview.xr_session();
        let session = session_handle.get().ok_or_else(null_session)?;
        let poses = session.tracked_poses();
        if self.xr.prs_devices.len() != poses.len() {
            for device in &self.xr.prs_devices {
                if !device.is_null() {
                    context.remove(&device.clone().into_cast(), false)?;
                }
            }
            // NCollection_Array1::Resize without copying: every slot is reset.
            self.xr.prs_devices_lower = 0;
            self.xr.prs_devices = vec![Handle::default(); poses.len()];
        }
        let head = session.named_tracked_device(AspectXRTrackedDeviceRole::Head);
        let left = session.named_tracked_device(AspectXRTrackedDeviceRole::LeftHand);
        let right = session.named_tracked_device(AspectXRTrackedDeviceRole::RightHand);
        for (index, pose) in poses.iter().enumerate() {
            let device_index = index as i32 + self.xr.prs_devices_lower;
            if !pose.is_valid_pose {
                continue;
            }
            let is_hand = device_index == left || device_index == right;
            if (!show_hands && is_hand) || (!show_aux && !is_hand) {
                let device = &self.xr.prs_devices[index];
                if !device.is_null() && device.has_interactive_context() {
                    context.remove(&device.clone().into_cast(), false)?;
                }
                continue;
            }
            let role = if device_index == left {
                AspectXRTrackedDeviceRole::LeftHand
            } else if device_index == right {
                AspectXRTrackedDeviceRole::RightHand
            } else {
                AspectXRTrackedDeviceRole::Other
            };
            let unit_factor = cview.unit_factor() as f32;
            let slot = &mut self.xr.prs_devices[index];
            if let Some(device) = slot.get()
                && device.unit_factor() != unit_factor
            {
                context.remove(&slot.clone().into_cast(), false)?;
                *slot = Handle::default();
            }
            if slot.is_null() {
                let mut texture: Handle<ImageTexture> = Handle::default();
                let triangles = if device_index != head {
                    session.load_render_model(device_index, &mut texture)
                } else {
                    Handle::default()
                };
                let device = if triangles.is_null() {
                    AisXrTrackedDevice::new()?
                } else {
                    AisXrTrackedDevice::with_render_model(&triangles, &texture.into_cast())?
                };
                device.set_unit_factor(unit_factor);
                device.set_mutable(true);
                device.set_infinite_state(true);
                *slot = device;
            }
            let device = slot.clone();
            device.set_role(role);
            let object: Handle<dyn AisInteractiveObject> = device.clone().into_cast();
            if !device.has_interactive_context() {
                context.display_with_modes(&object, 0, -1, false, PrsMgrDisplayStatus::None)?;
            }
            let mut pose_local = pose.orientation;
            if device_index == head {
                let t = pose_local.translation_part();
                pose_local.set_translation_part(Vector3::new(t.x(), 0.0, t.z()));
            }
            let pose_world = cview.pose_xr_to_world(&pose_local)?;
            context.set_location(&object, &Location::new(pose_world))?;
            let mut laser_length = 0.0;
            if is_hand && device.role() == self.xr.last_picking_hand {
                laser_length = self.laser_length(&*cview, self.xr.last_picking_hand)?;
                device.set_laser_color(self.xr.laser_pick_color)?;
            } else if is_hand && device.role() == self.xr.last_teleport_hand {
                laser_length = self.laser_length(&*cview, self.xr.last_teleport_hand)?;
                device.set_laser_color(self.xr.laser_tele_color)?;
            }
            device.set_laser_length(laser_length as f32)?;
        }
        Ok(())
    }

    /// `myXRTurnAngle`: the discrete trackpad turn angle; `controller.xr_turn_angle()`.
    pub fn xr_turn_angle(&self) -> f64 {
        self.xr.turn_angle
    }
    /// Set `myXRTurnAngle` (non-positive disables turning); `controller.set_xr_turn_angle(0.5)`.
    pub fn set_xr_turn_angle(&mut self, angle: f64) {
        self.xr.turn_angle = angle;
    }
    /// `myXRLastPickingHand`; `controller.xr_last_picking_hand()`.
    pub fn xr_last_picking_hand(&self) -> AspectXRTrackedDeviceRole {
        self.xr.last_picking_hand
    }
    /// `myXRLastTeleportHand`; `controller.xr_last_teleport_hand()`.
    pub fn xr_last_teleport_hand(&self) -> AspectXRTrackedDeviceRole {
        self.xr.last_teleport_hand
    }
    /// `myXRLastPickDepthLeft` / `myXRLastPickDepthRight`: the last picked laser length of a
    /// hand (infinite when nothing is picked); `controller.xr_last_pick_depth(role)`.
    pub fn xr_last_pick_depth(&self, hand: AspectXRTrackedDeviceRole) -> f64 {
        if hand == AspectXRTrackedDeviceRole::LeftHand {
            self.xr.last_pick_depth_left
        } else {
            self.xr.last_pick_depth_right
        }
    }
    /// `myXRPrsDevices`: the tracked-device presentations by device index (null slots for
    /// devices without one); `controller.xr_prs_devices()`.
    pub fn xr_prs_devices(&self) -> &[Handle<AisXrTrackedDevice>] {
        &self.xr.prs_devices
    }
    /// `myXRLaserTeleColor` / `myXRLaserPickColor`: the teleport and picking laser colors;
    /// `controller.xr_laser_colors()`.
    pub fn xr_laser_colors(
        &self,
    ) -> (
        rustycad_tkernel::quantity::Color,
        rustycad_tkernel::quantity::Color,
    ) {
        (self.xr.laser_tele_color, self.xr.laser_pick_color)
    }
    /// `myXRTeleportHaptic`, `myXRPickingHaptic` and `myXRSelectHaptic`: the vibrations on
    /// teleport aiming, highlighting and selection; `controller.xr_haptics()`.
    pub fn xr_haptics(&self) -> [rustycad_tkservice::aspect::AspectXRHapticActionData; 3] {
        [
            self.xr.teleport_haptic,
            self.xr.picking_haptic,
            self.xr.select_haptic,
        ]
    }

    fn pick_depth_mut(&mut self, hand: AspectXRTrackedDeviceRole) -> &mut f64 {
        if hand == AspectXRTrackedDeviceRole::LeftHand {
            &mut self.xr.last_pick_depth_left
        } else {
            &mut self.xr.last_pick_depth_right
        }
    }

    /// The laser length of a hand: its pick depth, else the view extent (100 when empty).
    fn laser_length(
        &mut self,
        cview: &dyn Graphic3dCView,
        hand: AspectXRTrackedDeviceRole,
    ) -> Result<f64, Failure> {
        let depth = *self.pick_depth_mut(hand);
        if !is_infinite(depth) {
            return Ok(depth);
        }
        let bounds = cview.min_max_values(true)?;
        Ok(if bounds.is_void() {
            100.0
        } else {
            bounds.square_extent().sqrt()
        })
    }
}

fn tracked_pose(
    session: &dyn AspectXRSession,
    device: i32,
) -> Result<rustycad_tkservice::aspect::AspectTrackedDevicePose, Failure> {
    session
        .tracked_poses()
        .get(device as usize)
        .copied()
        .ok_or_else(|| Failure::out_of_range("Aspect_XRSession::TrackedPoses: device index"))
}

/// The eye moved by a teleport: onto a horizontal surface at the head height, else against
/// the picked wall at a quarter of the head height.
fn teleport_eye(
    cview: &dyn Graphic3dCView,
    session: &dyn AspectXRSession,
    hand_base: &Transform3,
    pick_normal: [f32; 3],
    depth: f64,
    head_height: f64,
) -> Result<(), Failure> {
    let tele_dir = gp::dz().transformed(*hand_base).reversed();
    let up = cview.base_xr_camera().get().ok_or_else(null_session)?.up();
    let mut normal = rustycad_tkmath::gp::Direction3::new(
        f64::from(pick_normal[0]),
        f64::from(pick_normal[1]),
        f64::from(pick_normal[2]),
    )?;
    let quarter = std::f64::consts::FRAC_PI_4;
    let is_horizontal = up.is_equal(&normal, quarter) || up.is_equal(&normal.reversed(), quarter);
    let origin = hand_base.translation_part();
    let eye = if is_horizontal {
        origin + tele_dir.xyz() * depth + up.xyz() * head_height
    } else {
        if normal.dot(tele_dir) < 0.0 {
            normal.reverse();
        }
        origin + tele_dir.xyz() * depth - normal.xyz() * head_height / 4.0
    };
    let posed = cview.posed_xr_camera();
    posed
        .get()
        .ok_or_else(null_session)?
        .move_eye_to(&Point3::from(eye));
    cview.compute_xr_base_camera_from_posed(&posed, &session.head_pose())
}
