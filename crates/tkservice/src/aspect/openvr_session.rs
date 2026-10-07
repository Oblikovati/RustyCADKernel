//! `Aspect_OpenVRSession` as the oracle build compiles it: without OpenVR (`HAVE_OPENVR`
//! off), so no headset is present, `Open` fails with the native message and the queries
//! return empty values; the action type checks still raise.
use super::{
    AspectColorSpace, AspectEye, AspectGraphicsLibrary, AspectXRAction, AspectXRActionType,
    AspectXRAnalogActionData, AspectXRDigitalActionData, AspectXRHapticActionData,
    AspectXRPoseActionData, AspectXRSession, AspectXRSessionBase, AspectXRSessionInfoString,
    AspectXRSessionTrackingUniverseOrigin, AspectXRTrackedDeviceRole,
};
use crate::graphic3d::Graphic3dArrayOfTriangles;
use crate::image::ImageTexture;
use rustycad_tkernel::message::Message;
use rustycad_tkernel::ncollection::{Mat4, Vec2};
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::ffi::c_void;

/// The OpenVR session of a build without OpenVR; `AspectOpenVRSession::default().open() == false`.
#[derive(Default)]
pub struct AspectOpenVRSession {
    transient: StandardTransient,
    base: AspectXRSessionBase,
    /// `myActionsManifest`.
    actions_manifest: SharedState<AsciiString>,
}
rustycad_tkernel::impl_transient!(AspectOpenVRSession, "Aspect_OpenVRSession", dyn AspectXRSession, transient,
    interfaces: [dyn AspectXRSession]);

/// The `Standard_ProgramError` of an action of the wrong type (or null).
fn check_action(
    action: &Handle<AspectXRAction>,
    expected: AspectXRActionType,
    member: &str,
) -> Result<(), Failure> {
    match action.get() {
        Some(a) if a.action_type() == expected => Ok(()),
        found => Err(program_error(&format!(
            "Aspect_OpenVRSession::{member}() called for wrong action; got {:?}, expected {expected:?}",
            found.map(AspectXRAction::action_type)
        ))),
    }
}

impl AspectOpenVRSession {
    /// `IsHmdPresent`: no headset without OpenVR; `AspectOpenVRSession::is_hmd_present()`.
    pub fn is_hmd_present() -> bool {
        false
    }
    /// `defaultActionsManifest`: empty without OpenVR.
    pub fn default_actions_manifest(&self) -> AsciiString {
        AsciiString::default()
    }
    /// The actions manifest path; `session.actions_manifest()`.
    pub fn actions_manifest(&self) -> AsciiString {
        self.actions_manifest.read().clone()
    }
    /// `closeVR`: nothing to close without OpenVR.
    pub fn close_vr(&self) {}
    /// `getVRSystem`: null without OpenVR.
    pub fn get_vr_system(&self) -> *mut c_void {
        std::ptr::null_mut()
    }
    /// `updateProjectionFrustums`: nothing without OpenVR.
    pub fn update_projection_frustums(&self) {}
    /// `initInput`: false without OpenVR.
    pub fn init_input(&self) -> bool {
        false
    }
    /// `onTrackedDeviceActivated`: a trace message; `session.on_tracked_device_activated(1)`.
    pub fn on_tracked_device_activated(&self, device: i32) {
        trace_device(device, "attached");
    }
    /// `onTrackedDeviceDeactivated`: a trace message.
    pub fn on_tracked_device_deactivated(&self, device: i32) {
        trace_device(device, "detached");
    }
    /// `onTrackedDeviceUpdated`: a trace message.
    pub fn on_tracked_device_updated(&self, device: i32) {
        trace_device(device, "updated");
    }
}

fn trace_device(device: i32, what: &str) {
    Message::send_trace_text(&AsciiString::from(
        format!("OpenVR, Device {device} {what}").as_str(),
    ));
}

impl AspectXRSession for AspectOpenVRSession {
    fn xr_session_base(&self) -> &AspectXRSessionBase {
        &self.base
    }
    fn is_open(&self) -> bool {
        false
    }
    fn open(&self) -> bool {
        Message::send_fail_text(&AsciiString::from(
            "Open CASCADE has been built without OpenVR support",
        ));
        false
    }
    fn close(&self) {
        self.close_vr();
    }
    fn wait_poses(&self) -> bool {
        false
    }
    fn recommended_viewport(&self) -> Vec2<i32> {
        self.base.fields.read().rend_size
    }
    fn eye_to_head_transform(&self, _eye: AspectEye) -> Mat4<f64> {
        Mat4::identity()
    }
    fn projection_matrix(&self, _eye: AspectEye, _z_near: f64, _z_far: f64) -> Mat4<f64> {
        Mat4::identity()
    }
    fn has_projection_frustums(&self) -> bool {
        true
    }
    fn process_events(&self) {}
    fn submit_eye(
        &self,
        _texture: *mut c_void,
        _library: AspectGraphicsLibrary,
        _space: AspectColorSpace,
        _eye: AspectEye,
    ) -> bool {
        false
    }
    fn named_tracked_device(&self, _role: AspectXRTrackedDeviceRole) -> i32 {
        -1
    }
    fn get_digital_action_data(
        &self,
        action: &Handle<AspectXRAction>,
    ) -> Result<AspectXRDigitalActionData, Failure> {
        check_action(
            action,
            AspectXRActionType::InputDigital,
            "GetDigitalActionData",
        )?;
        Ok(AspectXRDigitalActionData::default())
    }
    fn get_analog_action_data(
        &self,
        action: &Handle<AspectXRAction>,
    ) -> Result<AspectXRAnalogActionData, Failure> {
        check_action(
            action,
            AspectXRActionType::InputAnalog,
            "GetAnalogActionData",
        )?;
        Ok(AspectXRAnalogActionData::default())
    }
    fn get_pose_action_data_for_next_frame(
        &self,
        action: &Handle<AspectXRAction>,
    ) -> Result<AspectXRPoseActionData, Failure> {
        check_action(
            action,
            AspectXRActionType::InputPose,
            "GetPoseActionDataForNextFrame",
        )?;
        Ok(AspectXRPoseActionData::default())
    }
    fn set_tracking_origin(&self, origin: AspectXRSessionTrackingUniverseOrigin) {
        self.base.fields.write().track_origin = origin;
    }
    fn get_string(&self, _info: AspectXRSessionInfoString) -> AsciiString {
        AsciiString::default()
    }
    fn load_render_model_impl(
        &self,
        _device: i32,
        _to_apply: bool,
        _texture: &mut Handle<ImageTexture>,
    ) -> Handle<Graphic3dArrayOfTriangles> {
        Handle::default()
    }
    fn trigger_haptic_vibration_action_impl(
        &self,
        action: &Handle<AspectXRAction>,
        _params: &AspectXRHapticActionData,
    ) -> Result<(), Failure> {
        check_action(
            action,
            AspectXRActionType::OutputHaptic,
            "triggerHapticVibrationAction",
        )
    }
}
