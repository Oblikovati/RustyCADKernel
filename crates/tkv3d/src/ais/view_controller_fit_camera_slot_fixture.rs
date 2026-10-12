//! Named neutral-window I/O replaces real camera fields before typed failures or unrelated panic retry.
use crate::v3d::{V3dViewSource,V3dViewMembers};
use rustycad_tkernel::standard::{Failure,Handle,SharedState,WeakHandle,raise_native_failure};
use rustycad_tkservice::{aspect::*,graphic3d::Graphic3dCamera};
use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
pub(super) struct FitCameraSlotControl {
    pub armed:AtomicBool,
    mode:i32,
    view:WeakHandle<dyn V3dViewSource>,
    replacement:WeakHandle<Graphic3dCamera>,
    pub records:SharedState<[i32;3]>,
}
pub(super) struct FitCameraSlotWindow {
    backend:AspectNeutralWindow,
    control:Arc<FitCameraSlotControl>,
}
rustycad_tkernel::impl_transient!(FitCameraSlotWindow,"Martensite_FitCameraSlotWindow",AspectNeutralWindow,backend,interfaces:[dyn AspectWindow]);
impl AspectWindow for FitCameraSlotWindow {
    forward_neutral_window_surface!(backend);
    fn size(&self)->(i32,i32) {
        let dimensions=self.backend.size();
        if self.control.armed.swap(false,Ordering::Relaxed) { self.control.replace_camera(); }
        dimensions
    }
}
impl FitCameraSlotControl {
    fn replace_camera(&self) {
        let view=self.view.observe();
        let before=view.camera_ref().observe().ref_count();
        view.set_camera(&*self.replacement.observe()).unwrap();
        *self.records.write()=[1,before,view.camera_ref().observe().ref_count()];
        match self.mode {
            1=>raise_native_failure(Err(Failure::new("fit Size fault=1; expected success"))),
            2=>std::panic::panic_any(73_u32),
            _=>{}
        }
    }
}
pub(super) fn fit_camera_slot_control(mode:i32,view:&Handle<dyn V3dViewSource>,replacement:&Handle<Graphic3dCamera>)->Arc<FitCameraSlotControl> {
    Arc::new(FitCameraSlotControl {armed:AtomicBool::new(false),mode,view:view.downgrade(),replacement:replacement.downgrade(),records:SharedState::new([0;3])})
}
pub(super) fn attach_fit_camera_slot_window(view:&Handle<dyn V3dViewSource>,control:&Arc<FitCameraSlotControl>)->Handle<FitCameraSlotWindow> {
    let backend=AspectNeutralWindow::default();backend.set_size(640,480);
    let window=Handle::new(FitCameraSlotWindow {backend,control:control.clone()});
    view.set_window(&window.cast(),0).unwrap();window
}
pub(super) fn fit_camera_slot_callback_failure(result:Result<Result<(),Failure>,Box<dyn std::any::Any+Send>>)->i32 {
    match result {
        Ok(Ok(()))=>0,
        Ok(Err(failure))=>{assert!(failure.message().contains("fit Size fault=1"));1},
        Err(payload)=>{assert_eq!(payload.downcast_ref::<u32>(),Some(&73));2}
    }
}
