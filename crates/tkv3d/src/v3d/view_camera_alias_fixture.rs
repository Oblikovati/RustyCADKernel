//! Genuine selected-slot camera overrides retain aliases independently of later getter selection.
use super::V3dViewSource;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkservice::graphic3d::{
    Graphic3dCamera, Graphic3dGraphicDriver, Graphic3dRetainedView,
};
use std::sync::Arc;
pub(super) struct CameraAliasControl {
    pub bounds_getter_calls: std::sync::atomic::AtomicI32,
    pub light_probe:
        SharedState<Option<Arc<super::viewer_light_probe_fixture::ViewerLightProbeControl>>>,
    pub light_identity: std::sync::atomic::AtomicUsize,
    pub alternate_selected: SharedState<bool>,
    pub alternate: Arc<SharedState<Handle<Graphic3dCamera>>>,
    pub caller: SharedState<Handle<Graphic3dCamera>>,
    pub replacement: SharedState<WeakHandle<Graphic3dCamera>>,
    pub replace_camera_argument: SharedState<bool>,
    pub view_caller: SharedState<Handle<dyn V3dViewSource>>,
    pub view_replacement: SharedState<WeakHandle<dyn V3dViewSource>>,
    pub replace_activation_argument: SharedState<bool>,
}
pub(super) struct BorrowedCameraView {
    pub transient: StandardTransient,
    pub identity: usize,
    pub retained: Graphic3dRetainedView,
    pub control: Arc<CameraAliasControl>,
}
rustycad_tkernel::impl_transient!(BorrowedCameraView,"Martensite_BorrowedCameraView",StandardTransient,transient,
    interfaces:[dyn rustycad_tkservice::graphic3d::Graphic3dCView,dyn rustycad_tkservice::graphic3d::Graphic3dDataStructureManager]);
pub(super) struct BorrowedCameraDriver {
    pub transient: StandardTransient,
    pub delegate: Handle<dyn Graphic3dGraphicDriver>,
    pub control: Arc<CameraAliasControl>,
}
rustycad_tkernel::impl_transient!(BorrowedCameraDriver,"Martensite_BorrowedCameraDriver",StandardTransient,transient,
    interfaces:[dyn Graphic3dGraphicDriver]);
pub(super) fn camera_alias_driver(
    delegate: &Handle<dyn Graphic3dGraphicDriver>,
    control: &Arc<CameraAliasControl>,
) -> Handle<dyn Graphic3dGraphicDriver> {
    Handle::new(BorrowedCameraDriver {
        transient: StandardTransient::default(),
        delegate: delegate.clone(),
        control: control.clone(),
    })
    .cast()
}
pub(super) fn fresh_camera_alias_control() -> Arc<CameraAliasControl> {
    Arc::new(CameraAliasControl {
        bounds_getter_calls: std::sync::atomic::AtomicI32::new(0),
        light_probe: Default::default(),
        light_identity: std::sync::atomic::AtomicUsize::new(0),
        alternate_selected: SharedState::new(false),
        alternate: Arc::new(SharedState::new(Handle::new(Graphic3dCamera::default()))),
        caller: SharedState::new(Handle::default()),
        replacement: SharedState::new(WeakHandle::default()),
        replace_camera_argument: SharedState::new(false),
        view_caller: SharedState::new(Handle::default()),
        view_replacement: SharedState::new(WeakHandle::default()),
        replace_activation_argument: SharedState::new(false),
    })
}
