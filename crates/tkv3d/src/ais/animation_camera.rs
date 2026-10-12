//! Martensite camera animation follows AIS_AnimationCamera at OCCT 3d097a0.
use super::{AisAnimationBase, AisAnimationProgress, AisAnimationSource};
use crate::v3d::V3dView;
use rustycad_tkernel::{
    standard::{Failure, Handle, StandardTransient},
    tcollection::AsciiString,
};
use rustycad_tkservice::graphic3d::Graphic3dCamera;

/// Native protected camera handles; `animation.camera_animation_base().fields.read().view`.
pub struct AisAnimationCameraFields {
    /// Target view, which may be detached.
    pub view: Handle<V3dView>,
    /// Starting camera, retained by identity.
    pub camera_start: Handle<Graphic3dCamera>,
    /// Ending camera, retained by identity.
    pub camera_end: Handle<Graphic3dCamera>,
}
/// Interpolate the live view camera without scheduling immediate redraws; `AisAnimationCamera::new("orbit", &view)`.
pub struct AisAnimationCamera {
    transient: StandardTransient,
    base: super::AisAnimationCameraBase,
}
rustycad_tkernel::impl_transient!(AisAnimationCamera,"AIS_AnimationCamera",dyn AisAnimationSource,transient,
    interfaces:[dyn super::AisAnimationCameraSource,dyn AisAnimationSource]);
impl AisAnimationCamera {
    /// Construct a stopped animation with unspecified endpoints; `AisAnimationCamera::new("orbit", &view)`.
    pub fn new(name: impl Into<AsciiString>, view: &Handle<V3dView>) -> Self {
        let base = super::AisAnimationCameraBase::new(name, view);
        Self {
            transient: StandardTransient::default(),
            base,
        }
    }

    /// Explicit copied target; native View parity uses AisAnimationCameraMembers::target_view_ref.
    /// Target view including null; `animation.target_view()`.
    pub fn target_view(&self) -> Handle<V3dView> {
        let snapshot = super::AisAnimationCameraMembers::target_view(self);
        snapshot
    }
    /// Retarget or detach the animation; `animation.set_view(&view)`.
    pub fn set_view(&self, view: &Handle<V3dView>) {
        // Concrete and derived handles mutate the same retained base field.
        super::AisAnimationCameraMembers::set_view(self, view);
    }
    /// Explicit copied start; native CameraStart parity uses AisAnimationCameraMembers::camera_start_ref.
    /// Retained start camera including null; `animation.camera_start()`.
    pub fn camera_start(&self) -> Handle<Graphic3dCamera> {
        let snapshot = super::AisAnimationCameraMembers::camera_start(self);
        snapshot
    }
    /// Retain the start endpoint by identity; `animation.set_camera_start(&camera)`.
    pub fn set_camera_start(&self, camera: &Handle<Graphic3dCamera>) {
        // Concrete and derived handles mutate the same retained base field.
        super::AisAnimationCameraMembers::set_camera_start(self, camera);
    }
    /// Explicit copied end; native CameraEnd parity uses AisAnimationCameraMembers::camera_end_ref.
    /// Retained end camera including null; `animation.camera_end()`.
    pub fn camera_end(&self) -> Handle<Graphic3dCamera> {
        let snapshot = super::AisAnimationCameraMembers::camera_end(self);
        snapshot
    }
    /// Retain the end endpoint by identity; `animation.set_camera_end(&camera)`.
    pub fn set_camera_end(&self, camera: &Handle<Graphic3dCamera>) {
        // Concrete and derived handles mutate the same retained base field.
        super::AisAnimationCameraMembers::set_camera_end(self, camera);
    }
}
impl AisAnimationSource for AisAnimationCamera {
    fn animation_base(&self) -> &AisAnimationBase {
        // Inherited playback sees the actual shared camera timeline.
        &self.base.animation
    }
    fn update_own(&self, progress: &AisAnimationProgress) -> Result<(), Failure> {
        // Concrete and derived atoms execute the same source-pinned interpolation body.
        super::animation_camera_source::ais_animation_camera_update(self, progress)
    }
}
impl super::AisAnimationCameraSource for AisAnimationCamera {
    fn camera_animation_base(&self) -> &super::AisAnimationCameraBase {
        // A subclass borrows the same owned animation and camera fields.
        &self.base
    }
}
