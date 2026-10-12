//! A real owned view subclass records native callback owners; completion is bounded against alias lock recursion.
use rustycad_tkernel::standard::{Handle, HandleReference, SharedState, WeakHandle};
use rustycad_tkservice::{
    aspect::AspectNeutralWindow, graphic3d::Graphic3dCViewMembers as _,
    null_driver::NullGraphicDriver,
};
use rustycad_tkv3d::v3d::*;
use std::sync::{Arc, mpsc};
use std::time::Duration;
struct ActivationAliasTrace {
    subject: SharedState<WeakHandle<dyn V3dViewSource>>,
    armed: SharedState<bool>,
    rows: SharedState<Vec<String>>,
}
struct ActivationAliasView {
    base: V3dView,
    trace: Arc<ActivationAliasTrace>,
}
rustycad_tkernel::impl_transient!(ActivationAliasView,"Martensite_ActivationAliasView",V3dView,base,
    interfaces:[dyn V3dViewSource]);
impl V3dViewSource for ActivationAliasView {
    fn view_base(&self) -> &V3dView {
        &self.base
    }
    fn redraw(&self) {
        if *self.trace.armed.read() {
            let owners = self.trace.subject.read().observe().ref_count();
            let implementation = self.base.view_ref().observe().ref_count();
            self.trace
                .rows
                .write()
                .push(format!("redraw {owners} {implementation}"));
        }
        v3d_view_redraw(&self.base);
    }
}
#[test]
fn one_defined_view_zero_lights_activates_without_lock_recursion_and_replays_native_owners() {
    let (sender, receiver) = mpsc::channel();
    let worker = std::thread::spawn(move || sender.send(activation_alias_replay()).unwrap());
    let rows = receiver.recv_timeout(Duration::from_secs(3)).expect(
        "Defined-view alias activation completion=timeout; expected completion within 3 seconds",
    );
    worker.join().unwrap();
    assert_eq!(
        rows,
        include_str!("fixtures/viewer-activation-alias-native.txt")
    );
}
fn activation_alias_replay() -> String {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let trace = Arc::new(ActivationAliasTrace {
        subject: Default::default(),
        armed: Default::default(),
        rows: Default::default(),
    });
    let view = make_activation_alias_view(&viewer, &trace);
    assert!(viewer.active_lights().is_empty());
    let cursor = viewer.defined_view_iterator();
    let alias = cursor.value_ref();
    let before = activation_alias_row("before", &view, &alias, &viewer);
    viewer.set_view_on().unwrap();
    let after = activation_alias_row("after", &view, &alias, &viewer);
    let callbacks = trace.rows.read().join("\n");
    format!("{before}\n{callbacks}\n{after}\n")
}
fn make_activation_alias_view(
    viewer: &Handle<V3dViewer>,
    trace: &Arc<ActivationAliasTrace>,
) -> Handle<dyn V3dViewSource> {
    let view: Handle<dyn V3dViewSource> = Handle::new(ActivationAliasView {
        base: V3dView::new_embedded(viewer, V3dTypeOfView::Orthographic).unwrap(),
        trace: trace.clone(),
    })
    .cast();
    V3dView::bind_source(&view).unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(640, 480);
    view.set_window(&window.cast(), 0).unwrap();
    viewer.set_view_off_one(&view);
    view.set_immediate_update(true);
    *trace.subject.write() = view.downgrade();
    *trace.armed.write() = true;
    view
}
fn activation_alias_row(
    phase: &str,
    view: &Handle<dyn V3dViewSource>,
    alias: &dyn HandleReference<dyn V3dViewSource>,
    viewer: &V3dViewer,
) -> String {
    let fresh = viewer.defined_view_iterator().value_ref();
    let same_slot = (alias.reference_address() == fresh.reference_address()) as u8;
    let implementation = view.view_ref().observe();
    format!(
        "{phase} {same_slot} {} {} {}",
        view.ref_count(),
        implementation.ref_count(),
        implementation.is_active() as u8
    )
}
