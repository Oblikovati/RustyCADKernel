//! Removing a live active node must inspect its handle before taking the same viewer's write guard.
use rustycad_tkernel::standard::{Handle, HandleReference};
use rustycad_tkservice::{
    aspect::AspectNeutralWindow, graphic3d::Graphic3dCViewMembers as _,
    null_driver::NullGraphicDriver,
};
use rustycad_tkv3d::v3d::*;
struct DeactivationAliasView {
    base: V3dView,
}
rustycad_tkernel::impl_transient!(DeactivationAliasView,"Martensite_DeactivationAliasView",V3dView,base,
    interfaces:[dyn V3dViewSource]);
impl V3dViewSource for DeactivationAliasView {
    fn view_base(&self) -> &V3dView {
        &self.base
    }
}
#[test]
fn active_node_deactivation_completes_and_replays_native_alias_owners() {
    let rows: String = (0..3).map(bounded_deactivation_case).collect();
    assert_eq!(
        rows,
        include_str!("fixtures/viewer-deactivation-alias-native.txt")
    );
}
fn bounded_deactivation_case(policy: usize) -> String {
    let (sender, receiver) = std::sync::mpsc::channel();
    let worker =
        std::thread::spawn(move || sender.send(deactivation_alias_replay(policy)).unwrap());
    let rows = receiver.recv_timeout(std::time::Duration::from_secs(3)).expect(
        "Active view-node deactivation completion=timeout; expected unlocked completion within 3 seconds",
    );
    worker.join().unwrap();
    rows
}
fn deactivation_alias_replay(policy: usize) -> String {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = mapped_deactivation_alias_view(&viewer);
    let defined = viewer.defined_view_iterator().value_ref();
    let before = deactivation_alias_row("before", &view, &defined, &viewer);
    deactivate_alias_case(policy, &viewer);
    let after = deactivation_alias_row("after", &view, &defined, &viewer);
    format!("case {policy}\n{before}\n{after}\n")
}
fn mapped_deactivation_alias_view(viewer: &Handle<V3dViewer>) -> Handle<dyn V3dViewSource> {
    let view: Handle<dyn V3dViewSource> = Handle::new(DeactivationAliasView {
        base: V3dView::new_embedded(viewer, V3dTypeOfView::Orthographic).unwrap(),
    })
    .cast();
    V3dView::bind_source(&view).unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(640, 480);
    view.set_window(&window.cast(), 0).unwrap();
    view
}
fn deactivation_alias_row(
    phase: &str,
    view: &Handle<dyn V3dViewSource>,
    alias: &dyn HandleReference<dyn V3dViewSource>,
    viewer: &V3dViewer,
) -> String {
    let same_slot = defined_alias_slot_identity(alias, viewer) as u8;
    let implementation = view.view_ref().observe();
    format!(
        "{phase} {same_slot} {} {} {}",
        view.ref_count(),
        implementation.ref_count(),
        implementation.is_active() as u8
    )
}

fn deactivate_alias_case(policy: usize, viewer: &V3dViewer) {
    if policy == 2 {
        viewer.set_view_off();
        return;
    }
    let cursor = if policy == 0 {
        viewer.active_view_iterator()
    } else {
        viewer.defined_view_iterator()
    };
    viewer.set_view_off_one(&cursor.value_ref());
}

fn defined_alias_slot_identity(
    alias: &dyn HandleReference<dyn V3dViewSource>,
    viewer: &V3dViewer,
) -> bool {
    let fresh = viewer.defined_view_iterator().value_ref();
    alias.reference_address() == fresh.reference_address()
}
