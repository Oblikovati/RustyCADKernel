//! Martensite camera animation follows AIS_AnimationCamera at OCCT 3d097a0.
use super::{AisAnimationBase, AisAnimationProgress, AisAnimationSource};
use crate::v3d::V3dView;
use rustycad_tkernel::{
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::AsciiString,
};
use rustycad_tkservice::graphic3d::Graphic3dCamera;

/// Native protected camera animation handles; `animation.fields.read().view`.
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
    base: AisAnimationBase,
    /// Native protected camera state shared by every handle.
    pub fields: SharedState<AisAnimationCameraFields>,
}
rustycad_tkernel::impl_transient!(AisAnimationCamera,"AIS_AnimationCamera",dyn AisAnimationSource,transient,
    interfaces:[dyn AisAnimationSource]);
impl AisAnimationCamera {
    /// Construct a stopped animation with unspecified endpoints; `AisAnimationCamera::new("orbit", &view)`.
    pub fn new(name: impl Into<AsciiString>, view: &Handle<V3dView>) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: AisAnimationBase::new(name),
            fields: SharedState::new(AisAnimationCameraFields {
                view: view.clone(),
                camera_start: Handle::default(),
                camera_end: Handle::default(),
            }),
        }
    }
    /// Target view including null; `animation.target_view()`.
    pub fn target_view(&self) -> Handle<V3dView> {
        self.fields.read().view.clone()
    }
    /// Retarget or detach the animation; `animation.set_view(&view)`.
    pub fn set_view(&self, view: &Handle<V3dView>) {
        self.fields.write().view = view.clone();
    }
    /// Retained start camera including null; `animation.camera_start()`.
    pub fn camera_start(&self) -> Handle<Graphic3dCamera> {
        self.fields.read().camera_start.clone()
    }
    /// Retain the start endpoint by identity; `animation.set_camera_start(&camera)`.
    pub fn set_camera_start(&self, camera: &Handle<Graphic3dCamera>) {
        self.fields.write().camera_start = camera.clone();
    }
    /// Retained end camera including null; `animation.camera_end()`.
    pub fn camera_end(&self) -> Handle<Graphic3dCamera> {
        self.fields.read().camera_end.clone()
    }
    /// Retain the end endpoint by identity; `animation.set_camera_end(&camera)`.
    pub fn set_camera_end(&self, camera: &Handle<Graphic3dCamera>) {
        self.fields.write().camera_end = camera.clone();
    }
}
impl AisAnimationSource for AisAnimationCamera {
    fn animation_base(&self) -> &AisAnimationBase {
        &self.base
    }
    fn update_own(&self, progress: &AisAnimationProgress) -> Result<(), Failure> {
        let (view, start, end) = camera_animation_endpoints(self);
        let (Some(view), Some(start), Some(end)) = (view.get(), start.get(), end.get()) else {
            return Ok(());
        };
        let fraction = if self.has_own_duration() {
            progress.local_normalized
        } else {
            1.
        };
        update_view_camera(view, start, end, fraction)
    }
}
use super::AisAnimationMembers;
fn update_view_camera(
    view: &V3dView,
    start: &Graphic3dCamera,
    end: &Graphic3dCamera,
    fraction: f64,
) -> Result<(), Failure> {
    let camera = view.camera();
    Graphic3dCamera::interpolate(start, end, fraction, &camera)?;
    // Native AIS_AnimationCamera temporarily suppresses redraw, then invalidates the view.
    let immediate = view.set_immediate_update(false);
    let result = view.set_camera(&camera);
    view.set_immediate_update(immediate);
    result?;
    view.invalidate();
    Ok(())
}

fn camera_animation_endpoints(
    animation: &AisAnimationCamera,
) -> (
    Handle<V3dView>,
    Handle<Graphic3dCamera>,
    Handle<Graphic3dCamera>,
) {
    let fields = animation.fields.read();
    (
        fields.view.clone(),
        fields.camera_start.clone(),
        fields.camera_end.clone(),
    )
}
