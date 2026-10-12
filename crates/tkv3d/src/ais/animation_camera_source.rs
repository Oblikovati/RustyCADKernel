//! Owned native camera-animation base preserves real virtual timeline subclass dispatch.
use super::{AisAnimationBase, AisAnimationCameraFields, AisAnimationProgress, AisAnimationSource};
use crate::v3d::V3dViewSource;
use rustycad_tkernel::{
    standard::{Failure, Handle, HandleFieldRef, SharedState},
    tcollection::AsciiString,
};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
/// Embed real timeline/camera state; `AisAnimationCameraBase::new("camera",&view)`.
pub struct AisAnimationCameraBase {
    /// Native inherited timeline state, shared by derived overrides.
    pub animation: AisAnimationBase,
    /// Native retained target and camera endpoint fields.
    pub fields: std::sync::Arc<SharedState<AisAnimationCameraFields>>,
}
impl AisAnimationCameraBase {
    /// Own the native stopped timeline and null endpoints; `AisAnimationCameraBase::new("camera",&view)`.
    pub fn new(name: impl Into<AsciiString>, view: &Handle<dyn V3dViewSource>) -> Self {
        Self {
            animation: AisAnimationBase::new(name),
            fields: std::sync::Arc::new(SharedState::new(AisAnimationCameraFields {
                view: view.clone(),
                camera_start: Handle::default(),
                camera_end: Handle::default(),
            })),
        }
    }
}
/// Native subclass extension with real inherited storage; `animation.camera_animation_base()`.
pub trait AisAnimationCameraSource: AisAnimationSource {
    /// Borrow the embedded native base; `animation.camera_animation_base().animation.fields.read()`.
    fn camera_animation_base(&self) -> &AisAnimationCameraBase;
}
/// Nonvirtual native members share retained storage and cannot be replaced by subclass virtual overrides.
pub trait AisAnimationCameraMembers: AisAnimationCameraSource {
    /// Borrow the actual target handle field; `let alias=animation.target_view_ref()`.
    fn target_view_ref(&self) -> HandleFieldRef<'_, AisAnimationCameraFields, dyn V3dViewSource> {
        let fields = &self.camera_animation_base().fields;
        HandleFieldRef::new(fields, camera_target_field)
    }
    /// Borrow the actual start handle field; `let alias=animation.camera_start_ref()`.
    fn camera_start_ref(&self) -> HandleFieldRef<'_, AisAnimationCameraFields, Graphic3dCamera> {
        let fields = &self.camera_animation_base().fields;
        HandleFieldRef::new(fields, camera_start_field)
    }
    /// Borrow the actual end handle field; `let alias=animation.camera_end_ref()`.
    fn camera_end_ref(&self) -> HandleFieldRef<'_, AisAnimationCameraFields, Graphic3dCamera> {
        let fields = &self.camera_animation_base().fields;
        HandleFieldRef::new(fields, camera_end_field)
    }
    /// Explicitly retain a target snapshot; native View parity uses target_view_ref; `animation.target_view()`.
    fn target_view(&self) -> Handle<dyn V3dViewSource> {
        let alias = self.target_view_ref();
        alias.clone_handle()
    }
    /// Explicitly retain a start snapshot; native CameraStart parity uses camera_start_ref; `animation.camera_start()`.
    fn camera_start(&self) -> Handle<Graphic3dCamera> {
        let alias = self.camera_start_ref();
        alias.clone_handle()
    }
    /// Explicitly retain an end snapshot; native CameraEnd parity uses camera_end_ref; `animation.camera_end()`.
    fn camera_end(&self) -> Handle<Graphic3dCamera> {
        let alias = self.camera_end_ref();
        alias.clone_handle()
    }
    /// Replace the native target by identity; `animation.set_view(&view)`.
    fn set_view(&self, view: &Handle<dyn V3dViewSource>) {
        let fields = &self.camera_animation_base().fields;
        fields.write().view = view.clone();
    }
    /// Replace the native start endpoint; `animation.set_camera_start(&camera)`.
    fn set_camera_start(&self, camera: &Handle<Graphic3dCamera>) {
        let fields = &self.camera_animation_base().fields;
        fields.write().camera_start = camera.clone();
    }
    /// Replace the native end endpoint; `animation.set_camera_end(&camera)`.
    fn set_camera_end(&self, camera: &Handle<Graphic3dCamera>) {
        let fields = &self.camera_animation_base().fields;
        fields.write().camera_end = camera.clone();
    }
}
impl<A: AisAnimationCameraSource + ?Sized> AisAnimationCameraMembers for A {}
impl rustycad_tkernel::standard::RuntimeTyped for dyn AisAnimationCameraSource {
    const TYPE_NAME: &'static str = "AIS_AnimationCamera";
    fn static_type() -> &'static rustycad_tkernel::standard::RuntimeType {
        // Source and concrete handles name one native class descriptor, not lookalike types.
        <super::AisAnimationCamera as rustycad_tkernel::standard::RuntimeTyped>::static_type()
    }
    fn project_from(object: &dyn rustycad_tkernel::standard::Transient) -> Option<&Self> {
        // Actual registered interface casters preserve arbitrary native subclass dispatch.
        rustycad_tkernel::standard::project_interface::<Self>(object)
    }
}
fn camera_target_field(fields: &AisAnimationCameraFields) -> &Handle<dyn V3dViewSource> {
    // Return storage directly so an alias observes future setter changes.
    &fields.view
}
fn camera_start_field(fields: &AisAnimationCameraFields) -> &Handle<Graphic3dCamera> {
    // CameraStart is a native reference; retaining a handle is explicit.
    &fields.camera_start
}
fn camera_end_field(fields: &AisAnimationCameraFields) -> &Handle<Graphic3dCamera> {
    // CameraEnd keeps the same live-field identity across endpoint replacements.
    &fields.camera_end
}
/// Execute the complete base camera interpolation; `ais_animation_camera_update(&animation,&progress)?`.
pub fn ais_animation_camera_update<A: AisAnimationCameraSource + ?Sized>(
    animation: &A,
    progress: &AisAnimationProgress,
) -> Result<(), Failure> {
    rustycad_tkernel::standard::catch_native_failure(|| {
        dispatch_camera_animation_atom(animation, progress)
    })
}
fn dispatch_camera_animation_atom<A: AisAnimationCameraSource + ?Sized>(
    animation: &A,
    progress: &AisAnimationProgress,
) -> Result<(), Failure> {
    use super::AisAnimationMembers;
    let fields = animation.camera_animation_base().fields.read();
    if fields.view.is_null() || fields.camera_start.is_null() || fields.camera_end.is_null() {
        return Ok(());
    }
    // NCollection_Lerp owns both endpoints, while myView remains a live field reference.
    let (start, end) = (fields.camera_start.clone(), fields.camera_end.clone());
    drop(fields);
    let fraction = if animation.has_own_duration() {
        progress.local_normalized
    } else {
        1.0
    };
    super::animation_camera_update::update_view_camera(animation, &start, &end, fraction)
}
