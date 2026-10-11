//! Named headset I/O supplies a deterministic open session; camera synchronization remains real.
use rustycad_tkernel::{
    ncollection::{Mat4, Vec2},
    standard::{Failure, Handle, StandardTransient},
    tcollection::AsciiString,
};
use rustycad_tkservice::{aspect::*, graphic3d::Graphic3dArrayOfTriangles, image::ImageTexture};
use std::ffi::c_void;
#[derive(Default)]
pub(super) struct NavigationOpenSession {
    transient: StandardTransient,
    backend: AspectOpenVRSession,
}
rustycad_tkernel::impl_transient!(NavigationOpenSession,"NavigationOpenSession",dyn AspectXRSession,transient,interfaces:[dyn AspectXRSession]);
impl AspectXRSession for NavigationOpenSession {
    fn xr_session_base(&self) -> &AspectXRSessionBase {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.xr_session_base()
    }
    fn is_open(&self) -> bool {
        // Simulated headset I/O exercises active XR without replacing cameras.
        true
    }
    fn open(&self) -> bool {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.open()
    }
    fn close(&self) {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.close()
    }
    fn wait_poses(&self) -> bool {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.wait_poses()
    }
    fn recommended_viewport(&self) -> Vec2<i32> {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.recommended_viewport()
    }
    fn eye_to_head_transform(&self, _eye: AspectEye) -> Mat4<f64> {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.eye_to_head_transform(_eye)
    }
    fn projection_matrix(&self, _eye: AspectEye, _z_near: f64, _z_far: f64) -> Mat4<f64> {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.projection_matrix(_eye, _z_near, _z_far)
    }
    fn has_projection_frustums(&self) -> bool {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.has_projection_frustums()
    }
    fn process_events(&self) {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.process_events()
    }
    fn submit_eye(
        &self,
        _texture: *mut c_void,
        _library: AspectGraphicsLibrary,
        _space: AspectColorSpace,
        _eye: AspectEye,
    ) -> bool {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.submit_eye(_texture, _library, _space, _eye)
    }
    fn named_tracked_device(&self, _role: AspectXRTrackedDeviceRole) -> i32 {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.named_tracked_device(_role)
    }
    fn get_digital_action_data(
        &self,
        action: &Handle<AspectXRAction>,
    ) -> Result<AspectXRDigitalActionData, Failure> {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.get_digital_action_data(action)
    }
    fn get_analog_action_data(
        &self,
        action: &Handle<AspectXRAction>,
    ) -> Result<AspectXRAnalogActionData, Failure> {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.get_analog_action_data(action)
    }
    fn get_pose_action_data_for_next_frame(
        &self,
        action: &Handle<AspectXRAction>,
    ) -> Result<AspectXRPoseActionData, Failure> {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.get_pose_action_data_for_next_frame(action)
    }
    fn set_tracking_origin(&self, origin: AspectXRSessionTrackingUniverseOrigin) {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.set_tracking_origin(origin)
    }
    fn get_string(&self, _info: AspectXRSessionInfoString) -> AsciiString {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend.get_string(_info)
    }
    fn load_render_model_impl(
        &self,
        _device: i32,
        _to_apply: bool,
        _texture: &mut Handle<ImageTexture>,
    ) -> Handle<Graphic3dArrayOfTriangles> {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend
            .load_render_model_impl(_device, _to_apply, _texture)
    }
    fn trigger_haptic_vibration_action_impl(
        &self,
        action: &Handle<AspectXRAction>,
        _params: &AspectXRHapticActionData,
    ) -> Result<(), Failure> {
        // Delegate non-headset behavior to the existing Martensite backend.
        self.backend
            .trigger_haptic_vibration_action_impl(action, _params)
    }
}
#[test]
fn deterministic_headset_preserves_backend_lifecycle_and_pose_queries() {
    let s = NavigationOpenSession::default();
    assert!(s.is_open());
    assert!(!s.open());
    s.close();
    assert!(!s.wait_poses());
    assert_eq!(s.recommended_viewport(), s.backend.recommended_viewport());
    assert_eq!(s.eye_to_head_transform(AspectEye::Left), Mat4::identity());
    assert_eq!(
        s.projection_matrix(AspectEye::Right, 0.1, 100.0),
        Mat4::identity()
    );
    assert!(s.has_projection_frustums());
    s.process_events();
}
#[test]
fn deterministic_headset_delegates_texture_and_action_validation() {
    let s = NavigationOpenSession::default();
    assert!(!s.submit_eye(
        std::ptr::null_mut(),
        AspectGraphicsLibrary::OpenGL,
        AspectColorSpace::Linear,
        AspectEye::Left
    ));
    assert!(
        s.load_render_model_impl(0, true, &mut Handle::default())
            .is_null()
    );
}

#[test]
fn deterministic_headset_delegates_metadata_and_tracking() {
    let s = NavigationOpenSession::default();
    assert_eq!(s.named_tracked_device(AspectXRTrackedDeviceRole::Head), -1);
    assert!(s.get_string(AspectXRSessionInfoString::Vendor).is_empty());
    s.set_tracking_origin(AspectXRSessionTrackingUniverseOrigin::Seated);
    assert_eq!(
        s.xr_session_base().fields.read().track_origin,
        AspectXRSessionTrackingUniverseOrigin::Seated
    );
}

#[test]
fn deterministic_headset_preserves_null_action_failures() {
    let s = NavigationOpenSession::default();
    assert!(s.get_digital_action_data(&Handle::default()).is_err());
    assert!(s.get_analog_action_data(&Handle::default()).is_err());
    assert!(
        s.get_pose_action_data_for_next_frame(&Handle::default())
            .is_err()
    );
    assert!(
        s.trigger_haptic_vibration_action_impl(
            &Handle::default(),
            &AspectXRHapticActionData::default()
        )
        .is_err()
    );
}
