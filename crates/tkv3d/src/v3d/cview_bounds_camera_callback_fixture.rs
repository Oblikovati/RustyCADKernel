//! Actual derived CView Camera slot selection changes inside named neutral-window Size callbacks.
use super::{V3dViewSource, view_camera_alias_fixture::CameraAliasControl};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, WeakHandle, raise_native_failure};
use rustycad_tkservice::{aspect::*, graphic3d::*};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
pub(super) struct BoundsCameraWindowControl {
    pub armed: AtomicBool,
    pub mode: i32,
    pub view: WeakHandle<dyn V3dViewSource>,
    pub replacement: WeakHandle<Graphic3dCamera>,
    pub selection: Arc<CameraAliasControl>,
    pub records: SharedState<[i32; 3]>,
}
pub(super) struct BoundsCameraCallbackWindow {
    backend: AspectNeutralWindow,
    control: Arc<BoundsCameraWindowControl>,
}
rustycad_tkernel::impl_transient!(BoundsCameraCallbackWindow,"Martensite_BoundsCameraCallbackWindow",AspectNeutralWindow,backend,interfaces:[dyn AspectWindow]);
impl AspectWindow for BoundsCameraCallbackWindow {
    forward_neutral_window_surface!(backend);
    fn size(&self) -> (i32, i32) {
        let dimensions = self.backend.size();
        if self.control.armed.swap(false, Ordering::Relaxed) {
            self.control.replace_selected_camera();
        }
        dimensions
    }
}
impl BoundsCameraWindowControl {
    fn replace_selected_camera(&self) {
        let view = self.view.observe();
        let before = view.camera_ref().observe().ref_count();
        self.replace_actual_slot();
        *self.records.write() = [1, before, view.camera_ref().observe().ref_count()];
        *self.selection.alternate_selected.write() = false;
        self.raise_configured_callback_failure();
    }
    fn replace_actual_slot(&self) {
        if *self.selection.alternate_selected.read() {
            *self.selection.alternate.write() = self.replacement.observe().clone();
            return;
        }
        self.view
            .observe()
            .set_camera(&*self.replacement.observe())
            .unwrap();
    }
    fn raise_configured_callback_failure(&self) {
        match self.mode {
            1 => raise_native_failure(Err(Failure::new("bounds Size fault=1; expected success"))),
            2 => std::panic::panic_any(73_u32),
            _ => {}
        }
    }
}
pub(super) fn bounds_camera_callback_window(
    control: &Arc<BoundsCameraWindowControl>,
) -> Handle<BoundsCameraCallbackWindow> {
    let backend = AspectNeutralWindow::default();
    backend.set_size(640, 480);
    Handle::new(BoundsCameraCallbackWindow {
        backend,
        control: control.clone(),
    })
}
