//! The native selected Camera reference captures a field once, never a manager getter.
use rustycad_tkernel::standard::{
    Handle, HandleObserver, HandleReference, SharedState, WeakHandleFieldRef,
};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use std::sync::{Arc, OnceLock};

/// Live camera field alias without native owner retention; `let alias=volume.camera_ref()`.
/// Expiration is a safe extension outside the native valid owner lifetime, returning null.
#[derive(Clone)]
pub struct SelectMgrCameraFieldReference {
    field: Option<WeakHandleFieldRef<Handle<Graphic3dCamera>, Graphic3dCamera>>,
}
impl SelectMgrCameraFieldReference {
    pub(crate) fn capture(camera: &Arc<SharedState<Handle<Graphic3dCamera>>>) -> Self {
        Self {
            field: Some(WeakHandleFieldRef::new(camera, captured_camera_field)),
        }
    }
    pub(crate) fn empty() -> Self {
        // Native manager Camera returns one function-static const empty handle, not its active slot.
        Self { field: None }
    }
}
impl HandleReference<Graphic3dCamera> for SelectMgrCameraFieldReference {
    fn observe(&self) -> HandleObserver<Graphic3dCamera> {
        match &self.field {
            Some(field) => field.observe(),
            None => static_empty_manager_camera().observe(),
        }
    }
    fn reference_address(&self) -> *const () {
        match &self.field {
            Some(field) => field.reference_address(),
            None => std::ptr::from_ref(static_empty_manager_camera()).cast(),
        }
    }
}
fn captured_camera_field(camera: &Handle<Graphic3dCamera>) -> &Handle<Graphic3dCamera> {
    // The physical field survives camera replacement; no copied or reselected getter participates.
    camera
}
fn static_empty_manager_camera() -> &'static Handle<Graphic3dCamera> {
    // OCCT SelectMgr_SelectingVolumeManager.cxx109 returns this exact shared fallback slot.
    static EMPTY_CAMERA: OnceLock<Handle<Graphic3dCamera>> = OnceLock::new();
    EMPTY_CAMERA.get_or_init(Handle::default)
}
