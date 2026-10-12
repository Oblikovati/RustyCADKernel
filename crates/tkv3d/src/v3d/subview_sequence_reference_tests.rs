//! Real view overrides mutate the actual native subview sequence while node aliases stay live.
use super::*;
use rustycad_tkernel::standard::{
    Handle, HandleReference, SharedState, WeakHandle, catch_native_failure,
};
use rustycad_tkservice::null_driver::NullGraphicDriver;
use std::sync::Arc;
struct SubviewMutationCommands {
    policy: i32,
    parent: WeakHandle<dyn V3dViewSource>,
    removed: WeakHandle<dyn V3dViewSource>,
    appended: WeakHandle<dyn V3dViewSource>,
}
struct SubviewMutationProbe {
    base: V3dView,
    identity: i32,
    commands: Arc<SharedState<Option<SubviewMutationCommands>>>,
}
rustycad_tkernel::impl_transient!(SubviewMutationProbe,"Martensite_SubviewMutationProbe",V3dView,base,interfaces:[dyn V3dViewSource]);
impl V3dViewSource for SubviewMutationProbe {
    fn view_base(&self) -> &V3dView {
        &self.base
    }
    fn redraw(&self) {
        v3d_view_redraw(&self.base);
        if self.identity != 1 {
            return;
        }
        let commands = self.commands.read();
        apply_subview_mutation(commands.as_ref().unwrap());
    }
}
fn apply_subview_mutation(commands: &SubviewMutationCommands) {
    let parent = commands.parent.observe();
    let removed = commands.removed.observe();
    match commands.policy {
        1 => {
            parent.remove_subview(removed.view_base());
            parent.add_subview(&commands.appended.observe().clone());
        }
        2 => *parent.state.write().subviews.first_mut().unwrap() = removed.clone(),
        _ => unreachable!(),
    }
}
fn subview_probe(
    viewer: &Handle<V3dViewer>,
    identity: i32,
    commands: &Arc<SharedState<Option<SubviewMutationCommands>>>,
) -> Handle<dyn V3dViewSource> {
    let view = Handle::new(SubviewMutationProbe {
        base: V3dView::new_embedded(viewer, V3dTypeOfView::Orthographic).unwrap(),
        identity,
        commands: commands.clone(),
    })
    .cast();
    V3dView::bind_source(&view).unwrap();
    view
}
fn subview_probe_identity(view: &Handle<dyn V3dViewSource>) -> i32 {
    let projected = view.view::<SubviewMutationProbe>();
    projected.get().unwrap().identity
}
fn capture_subview_node_rows(policy: i32) -> Vec<String> {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let parent = V3dView::new(&viewer, V3dTypeOfView::Orthographic).unwrap();
    let commands = Arc::new(SharedState::new(None));
    let children: Vec<Handle<dyn V3dViewSource>> = (1..=4)
        .map(|id| subview_probe(&viewer, id, &commands))
        .collect();
    for child in &children[..3] {
        parent.add_subview(child);
    }
    set_subview_mutation_commands(policy, &parent, &children, &commands);
    capture_subview_sequence_nodes(policy, &parent)
}
fn set_subview_mutation_commands(
    policy: i32,
    parent: &Handle<dyn V3dViewSource>,
    children: &[Handle<dyn V3dViewSource>],
    commands: &Arc<SharedState<Option<SubviewMutationCommands>>>,
) {
    *commands.write() = Some(SubviewMutationCommands {
        policy,
        parent: parent.downgrade(),
        removed: children[1].downgrade(),
        appended: children[3].downgrade(),
    });
}
fn capture_subview_sequence_nodes(policy: i32, parent: &Handle<dyn V3dViewSource>) -> Vec<String> {
    let mut cursor = parent.subview_iterator();
    let mut rows = Vec::new();
    while cursor.more() {
        let slot = cursor.value_ref();
        let address = slot.reference_address();
        let before = subview_probe_identity(&slot.observe());
        slot.observe().redraw();
        let after = slot.observe();
        rows.push(subview_node_snapshot(
            policy, before, address, &slot, &after, parent,
        ));
        cursor.next();
    }
    rows
}
fn subview_node_snapshot(
    policy: i32,
    before: i32,
    address: *const (),
    slot: &V3dSubviewNodeRef,
    after: &Handle<dyn V3dViewSource>,
    parent: &Handle<dyn V3dViewSource>,
) -> String {
    format!(
        "{policy} {before} {} {} {} {}",
        subview_probe_identity(after),
        i32::from(slot.reference_address() == address),
        after.ref_count(),
        parent.state.read().subviews.len()
    )
}

#[test]
fn native_redraw_mutations_preserve_live_subview_node_aliases_and_exact_counts() {
    let rows: Vec<String> = [1, 2]
        .into_iter()
        .flat_map(capture_subview_node_rows)
        .collect();
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/v3d-subview-nodes-native.txt")
    );
}
#[test]
fn removing_the_retained_subview_node_reports_a_typed_failure() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let parent = viewer.create_view().unwrap();
    let child = viewer.create_view().unwrap();
    parent.add_subview(&child);
    let slot = parent.subview_iterator().value_ref();
    assert!(parent.remove_subview(child.view_base()));
    assert!(!parent.remove_subview(child.view_base()));
    assert!(catch_native_failure(|| Ok(slot.observe())).is_err());
    assert!(catch_native_failure(|| Ok(slot.reference_address())).is_err());
}
