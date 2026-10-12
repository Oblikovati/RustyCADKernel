//! Named native-style derived view executes real inherited bodies and records callback ownership.
use super::*;
use rustycad_tkernel::standard::{Handle, HandleReference, SharedState, WeakHandle};
use rustycad_tkservice::aspect::{AspectNeutralWindow, AspectWindowMembers as _};
use rustycad_tkservice::graphic3d::Graphic3dClipPlane;
use std::sync::Arc;

pub(super) struct ViewCallbackTrace {
    pub rows: SharedState<Vec<String>>,
    pub plane_slot: SharedState<Handle<Graphic3dClipPlane>>,
    pub replacement: Handle<Graphic3dClipPlane>,
    pub replace_on_add: SharedState<bool>,
    pub list_mutation: SharedState<ProbeListMutation>,
    pub failing_operation: SharedState<Option<&'static str>>,
}
#[derive(Default)]
pub(super) struct ProbeListMutation {
    pub policy: i32,
    pub removed: WeakHandle<dyn V3dViewSource>,
    pub appended: WeakHandle<dyn V3dViewSource>,
    pub replacement: WeakHandle<dyn V3dViewSource>,
}
pub(super) struct NativeStyleViewProbe {
    pub base: V3dView,
    pub identity: i32,
    pub trace: Arc<ViewCallbackTrace>,
}
rustycad_tkernel::impl_transient!(NativeStyleViewProbe,"Martensite_ViewSourceProbe",V3dView,base,
    interfaces:[dyn V3dViewSource]);
impl V3dViewSource for NativeStyleViewProbe {
    fn view_base(&self) -> &V3dView {
        &self.base
    }
    fn redraw(&self) {
        self.record("redraw");
        self.fail_if_requested("redraw");
        v3d_view_redraw(&self.base);
        if self.identity == 1 {
            super::view_source_list_fixture::apply_probe_list_mutation(self);
        }
    }
    fn redraw_immediate(&self) {
        self.record("immediate");
        self.fail_if_requested("immediate");
        v3d_view_redraw_immediate(&self.base);
    }
    fn add_clip_plane(&self, plane: &dyn HandleReference<Graphic3dClipPlane>) {
        self.record("add");
        self.fail_if_requested("add");
        if *self.trace.replace_on_add.read() {
            *self.trace.plane_slot.write() = self.trace.replacement.clone();
        }
        v3d_view_add_clip_plane(&self.base, plane);
    }
    fn remove_clip_plane(&self, plane: &dyn HandleReference<Graphic3dClipPlane>) {
        self.record("remove");
        self.fail_if_requested("remove");
        v3d_view_remove_clip_plane(&self.base, plane);
    }
}
impl NativeStyleViewProbe {
    fn fail_if_requested(&self, operation: &str) {
        if *self.trace.failing_operation.read() == Some(operation) {
            rustycad_tkernel::standard::raise_native_failure::<()>(Err(
                rustycad_tkernel::standard::Failure::new(&format!(
                    "view callback operation={operation}; expected success"
                )),
            ));
        }
    }
    fn record(&self, operation: &str) {
        let count = self.base.this.read().observe().ref_count();
        self.trace
            .rows
            .write()
            .push(format!("{} {operation} {count}", self.identity));
    }
}
pub(super) fn mapped_probe_view(
    viewer: &Handle<V3dViewer>,
    identity: i32,
    trace: &Arc<ViewCallbackTrace>,
) -> Handle<dyn V3dViewSource> {
    let base = V3dView::new_embedded(viewer, V3dTypeOfView::Orthographic).unwrap();
    let view = Handle::new(NativeStyleViewProbe {
        base,
        identity,
        trace: trace.clone(),
    })
    .cast();
    V3dView::bind_source(&view).unwrap();
    map_probe_window(&view);
    view
}
fn map_probe_window(view: &Handle<dyn V3dViewSource>) {
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_virtual(true);
    window.set_size(640, 480);
    view.set_window(&window.cast(), 0).unwrap();
    view.set_immediate_update(false);
    view.set_auto_z_fit_mode(false, 1.0).unwrap();
}
pub(super) fn fresh_callback_trace() -> Arc<ViewCallbackTrace> {
    Arc::new(ViewCallbackTrace {
        rows: SharedState::new(Vec::new()),
        plane_slot: SharedState::new(Handle::default()),
        replacement: Handle::new(Graphic3dClipPlane::default()),
        replace_on_add: SharedState::new(false),
        list_mutation: SharedState::new(ProbeListMutation::default()),
        failing_operation: SharedState::new(None),
    })
}
