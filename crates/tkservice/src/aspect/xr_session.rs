//! `Aspect_XRSession`: the abstract XR (VR headset) session, a trait over an embedded base
//! holding the generic actions, action sets, tracked poses and display parameters.
use super::{
    ASPECT_XR_GENERIC_ACTION_NB, AspectColorSpace, AspectEye, AspectFrustumLrbt,
    AspectGraphicsLibrary, AspectTrackedDevicePose, AspectXRAction, AspectXRActionSet,
    AspectXRAnalogActionData, AspectXRDigitalActionData, AspectXRGenericAction,
    AspectXRHapticActionData, AspectXRPoseActionData, AspectXRTrackedDeviceRole,
};
use crate::graphic3d::Graphic3dArrayOfTriangles;
use crate::image::ImageTexture;
use rustycad_tkernel::ncollection::{Mat4, Vec2};
use rustycad_tkernel::standard::exceptions::{out_of_range, program_error};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Transform3;
use std::ffi::c_void;

/// `Aspect_XRTrackedDeviceRole_NB`.
pub const ASPECT_XR_TRACKED_DEVICE_ROLE_NB: i32 = AspectXRTrackedDeviceRole::Other as i32 + 1;

/// `Aspect_XRSession::TrackingUniverseOrigin`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AspectXRSessionTrackingUniverseOrigin {
    /// `TrackingUniverseOrigin_Seated`: poses relative to the seated zero pose.
    Seated = 0,
    /// `TrackingUniverseOrigin_Standing`: poses relative to the configured safe bounds.
    Standing = 1,
}

/// `Aspect_XRSession::InfoString`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AspectXRSessionInfoString {
    /// `InfoString_Vendor`.
    Vendor = 0,
    /// `InfoString_Device`.
    Device = 1,
    /// `InfoString_Tracker`.
    Tracker = 2,
    /// `InfoString_SerialNumber`.
    SerialNumber = 3,
}

/// The protected fields of a session.
#[derive(Clone)]
pub struct AspectXRSessionFields {
    /// `myRoleActions`: the generic actions of each device role.
    pub role_actions: Vec<Vec<Handle<AspectXRAction>>>,
    /// `myActionSets`.
    pub action_sets: Vec<(AsciiString, Handle<AspectXRActionSet>)>,
    /// `myTrackOrigin`.
    pub track_origin: AspectXRSessionTrackingUniverseOrigin,
    /// `myTrackedPoses` (indexed from 0).
    pub tracked_poses: Vec<AspectTrackedDevicePose>,
    /// `myHeadPose`.
    pub head_pose: Transform3,
    /// `myRendSize`.
    pub rend_size: Vec2<i32>,
    /// `myFrustumL`.
    pub frustum_l: AspectFrustumLrbt<f64>,
    /// `myFrustumR`.
    pub frustum_r: AspectFrustumLrbt<f64>,
    /// `myUnitFactor`.
    pub unit_factor: f64,
    /// `myAspect`.
    pub aspect: f64,
    /// `myFieldOfView`.
    pub field_of_view: f64,
    /// `myIod`.
    pub iod: f64,
    /// `myDispFreq`.
    pub disp_freq: f32,
}

impl Default for AspectXRSessionFields {
    /// Standing origin, one (invalid) tracked pose, unit factor 1, aspect 1, 90 degrees.
    fn default() -> Self {
        let actions = vec![Handle::default(); ASPECT_XR_GENERIC_ACTION_NB as usize];
        Self {
            role_actions: vec![actions; ASPECT_XR_TRACKED_DEVICE_ROLE_NB as usize],
            action_sets: Vec::new(),
            track_origin: AspectXRSessionTrackingUniverseOrigin::Standing,
            tracked_poses: vec![AspectTrackedDevicePose::default()],
            head_pose: Transform3::default(),
            rend_size: Vec2::new(0, 0),
            frustum_l: AspectFrustumLrbt::default(),
            frustum_r: AspectFrustumLrbt::default(),
            unit_factor: 1.0,
            aspect: 1.0,
            field_of_view: 90.0,
            iod: 0.0,
            disp_freq: 0.0,
        }
    }
}

/// The embedded state of the session classes.
#[derive(Default)]
pub struct AspectXRSessionBase {
    /// The fields, shared by every handle.
    pub fields: SharedState<AspectXRSessionFields>,
}

/// An XR session; a backend implements the virtual members.
pub trait AspectXRSession: Transient {
    /// The embedded base; `session.xr_session_base()`.
    fn xr_session_base(&self) -> &AspectXRSessionBase;
    /// Whether the session is open; `session.is_open()`.
    fn is_open(&self) -> bool;
    /// Opens the session; `session.open()`.
    fn open(&self) -> bool;
    /// Closes the session; `session.close()`.
    fn close(&self);
    /// Waits for the next poses; `session.wait_poses()`.
    fn wait_poses(&self) -> bool;
    /// The recommended viewport size; `session.recommended_viewport()`.
    fn recommended_viewport(&self) -> Vec2<i32>;
    /// The eye to head transformation; `session.eye_to_head_transform(AspectEye::Left)`.
    fn eye_to_head_transform(&self, eye: AspectEye) -> Mat4<f64>;
    /// The projection matrix of an eye; `session.projection_matrix(eye, 0.1, 100.0)`.
    fn projection_matrix(&self, eye: AspectEye, z_near: f64, z_far: f64) -> Mat4<f64>;
    /// Whether projection frustums are given; `session.has_projection_frustums()`.
    fn has_projection_frustums(&self) -> bool;
    /// Processes the backend events; `session.process_events()`.
    fn process_events(&self);
    /// Submits an eye texture; `session.submit_eye(texture, library, space, eye)`.
    fn submit_eye(
        &self,
        texture: *mut c_void,
        library: AspectGraphicsLibrary,
        color_space: AspectColorSpace,
        eye: AspectEye,
    ) -> bool;
    /// The tracked device of a role (-1 when none); `session.named_tracked_device(role)`.
    fn named_tracked_device(&self, role: AspectXRTrackedDeviceRole) -> i32;
    /// The state of a digital action; `session.get_digital_action_data(&action)?`.
    fn get_digital_action_data(
        &self,
        action: &Handle<AspectXRAction>,
    ) -> Result<AspectXRDigitalActionData, Failure>;
    /// The state of an analog action; `session.get_analog_action_data(&action)?`.
    fn get_analog_action_data(
        &self,
        action: &Handle<AspectXRAction>,
    ) -> Result<AspectXRAnalogActionData, Failure>;
    /// The pose of an action for the next frame; `session.get_pose_action_data_for_next_frame(&action)?`.
    fn get_pose_action_data_for_next_frame(
        &self,
        action: &Handle<AspectXRAction>,
    ) -> Result<AspectXRPoseActionData, Failure>;
    /// Sets the tracking origin; `session.set_tracking_origin(origin)`.
    fn set_tracking_origin(&self, origin: AspectXRSessionTrackingUniverseOrigin) {
        self.xr_session_base().fields.write().track_origin = origin;
    }
    /// A device information string; `session.get_string(AspectXRSessionInfoString::Vendor)`.
    fn get_string(&self, info: AspectXRSessionInfoString) -> AsciiString;
    /// `loadRenderModel`: the render model of a device; `session.load_render_model_impl(0, true, &mut texture)`.
    fn load_render_model_impl(
        &self,
        device: i32,
        to_apply_unit_factor: bool,
        texture: &mut Handle<ImageTexture>,
    ) -> Handle<Graphic3dArrayOfTriangles>;
    /// `triggerHapticVibrationAction`; `session.trigger_haptic_vibration_action_impl(&action, &params)?`.
    fn trigger_haptic_vibration_action_impl(
        &self,
        action: &Handle<AspectXRAction>,
        params: &AspectXRHapticActionData,
    ) -> Result<(), Failure>;
}
rustycad_tkernel::impl_transient_interface!(
    dyn AspectXRSession,
    "Aspect_XRSession",
    StandardTransient
);

/// The non-virtual members of `Aspect_XRSession`, for every implementer.
pub trait AspectXRSessionMembers: AspectXRSession {
    /// The inverse of the eye to head transformation (the identity when singular).
    /// Example: `session.head_to_eye_transform(AspectEye::Right)`.
    fn head_to_eye_transform(&self, eye: AspectEye) -> Mat4<f64> {
        let mut inverse = Mat4::identity();
        let _ = self.eye_to_head_transform(eye).inverted_to(&mut inverse);
        inverse
    }
    /// The unit scale factor (for meters); `session.unit_factor()`.
    fn unit_factor(&self) -> f64 {
        self.xr_session_base().fields.read().unit_factor
    }
    /// Sets the unit scale factor; `session.set_unit_factor(1000.0)`.
    fn set_unit_factor(&self, factor: f64) {
        self.xr_session_base().fields.write().unit_factor = factor;
    }
    /// The aspect ratio; `session.aspect()`.
    fn aspect(&self) -> f64 {
        self.xr_session_base().fields.read().aspect
    }
    /// The field of view; `session.field_of_view()`.
    fn field_of_view(&self) -> f64 {
        self.xr_session_base().fields.read().field_of_view
    }
    /// The intra-ocular distance in meters; `session.iod()`.
    fn iod(&self) -> f64 {
        self.xr_session_base().fields.read().iod
    }
    /// The display frequency; `session.display_frequency()`.
    fn display_frequency(&self) -> f32 {
        self.xr_session_base().fields.read().disp_freq
    }
    /// The projection frustum of an eye; `session.projection_frustum(AspectEye::Left)`.
    fn projection_frustum(&self, eye: AspectEye) -> AspectFrustumLrbt<f64> {
        let fields = self.xr_session_base().fields.read();
        if eye == AspectEye::Right {
            fields.frustum_r
        } else {
            fields.frustum_l
        }
    }
    /// The head pose; `session.head_pose()`.
    fn head_pose(&self) -> Transform3 {
        self.xr_session_base().fields.read().head_pose
    }
    /// The left hand pose (identity without a left hand); `session.left_hand_pose()?`.
    fn left_hand_pose(&self) -> Result<Transform3, Failure> {
        role_pose(self, AspectXRTrackedDeviceRole::LeftHand)
    }
    /// The right hand pose (identity without a right hand); `session.right_hand_pose()?`.
    fn right_hand_pose(&self) -> Result<Transform3, Failure> {
        role_pose(self, AspectXRTrackedDeviceRole::RightHand)
    }
    /// The tracked poses; `session.tracked_poses()`.
    fn tracked_poses(&self) -> Vec<AspectTrackedDevicePose> {
        self.xr_session_base().fields.read().tracked_poses.clone()
    }
    /// Whether a device pose is valid; a device out of range raises `Standard_OutOfRange`.
    /// Example: `session.has_tracked_pose(0)?`.
    fn has_tracked_pose(&self, device: i32) -> Result<bool, Failure> {
        Ok(tracked_pose(&self.xr_session_base().fields.read(), device)?.is_valid_pose)
    }
    /// `LoadRenderModel(device, texture)`: with the unit factor applied.
    /// Example: `session.load_render_model(0, &mut texture)`.
    fn load_render_model(
        &self,
        device: i32,
        texture: &mut Handle<ImageTexture>,
    ) -> Handle<Graphic3dArrayOfTriangles> {
        self.load_render_model_impl(device, true, texture)
    }
    /// `LoadRenderModel(device, toApplyUnitFactor, texture)`.
    /// Example: `session.load_render_model_scaled(0, false, &mut texture)`.
    fn load_render_model_scaled(
        &self,
        device: i32,
        to_apply_unit_factor: bool,
        texture: &mut Handle<ImageTexture>,
    ) -> Handle<Graphic3dArrayOfTriangles> {
        self.load_render_model_impl(device, to_apply_unit_factor, texture)
    }
    /// `TriggerHapticVibrationAction`: invalid parameters raise `Standard_ProgramError`.
    /// Example: `session.trigger_haptic_vibration_action(&action, &params)?`.
    fn trigger_haptic_vibration_action(
        &self,
        action: &Handle<AspectXRAction>,
        params: &AspectXRHapticActionData,
    ) -> Result<(), Failure> {
        if !params.is_valid() {
            return Err(program_error(&format!(
                "Aspect_OpenVRSession::TriggerHapticVibrationAction() called for wrong action; invalid parameters {params:?}"
            )));
        }
        self.trigger_haptic_vibration_action_impl(action, params)
    }
    /// `AbortHapticVibrationAction`: triggers the default (stop) parameters.
    /// Example: `session.abort_haptic_vibration_action(&action)?`.
    fn abort_haptic_vibration_action(
        &self,
        action: &Handle<AspectXRAction>,
    ) -> Result<(), Failure> {
        self.trigger_haptic_vibration_action_impl(action, &AspectXRHapticActionData::default())
    }
    /// The tracking origin; `session.tracking_origin()`.
    fn tracking_origin(&self) -> AspectXRSessionTrackingUniverseOrigin {
        self.xr_session_base().fields.read().track_origin
    }
    /// The generic action of a device role; `session.generic_action(role, AspectXRGenericAction::InputPoseBase)`.
    fn generic_action(
        &self,
        role: AspectXRTrackedDeviceRole,
        action: AspectXRGenericAction,
    ) -> Handle<AspectXRAction> {
        self.xr_session_base().fields.read().role_actions[role as usize][action as usize].clone()
    }
}
impl<T: AspectXRSession + ?Sized> AspectXRSessionMembers for T {}

/// `myTrackedPoses[device]` with the native range check.
fn tracked_pose(
    fields: &AspectXRSessionFields,
    device: i32,
) -> Result<AspectTrackedDevicePose, Failure> {
    usize::try_from(device)
        .ok()
        .and_then(|d| fields.tracked_poses.get(d))
        .copied()
        .ok_or_else(|| {
            out_of_range(&format!(
                "NCollection_Array1::Value - device {device}; expected 0..{}",
                fields.tracked_poses.len()
            ))
        })
}

/// The pose of a role's device, the identity without one.
fn role_pose<S: AspectXRSession + ?Sized>(
    session: &S,
    role: AspectXRTrackedDeviceRole,
) -> Result<Transform3, Failure> {
    let device = session.named_tracked_device(role);
    if device == -1 {
        return Ok(Transform3::default());
    }
    Ok(tracked_pose(&session.xr_session_base().fields.read(), device)?.orientation)
}
