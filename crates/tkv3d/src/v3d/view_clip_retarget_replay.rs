//! Exact pinned protected-field retarget records, including old Camera and live View aliases.
use super::view_clip_retarget_fixture::{ClipRetargetActors, clip_retarget_actors};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkservice::graphic3d::{Graphic3dCView, Graphic3dCamera};
fn record_final(actors: &ClipRetargetActors) -> String {
    let old = actors.old.clip_probe_slot.as_ref().unwrap();
    let next = actors.next.clip_probe_slot.as_ref().unwrap();
    format!(
        "final {} {} {} {} {} {} {}",
        i32::from(actors.owner.view_ref().object_address() == actors.next.object_address()),
        i32::from(old.object_address() == next.object_address()),
        old.observe().size(),
        actors.old.ref_count(),
        actors.next.ref_count(),
        old.observe().ref_count(),
        actors.plane.ref_count()
    )
}
fn record_aliases(
    actors: &ClipRetargetActors,
    view: &dyn HandleReference<dyn Graphic3dCView>,
    camera: &dyn HandleReference<Graphic3dCamera>,
) -> String {
    let fresh = actors.owner.camera_ref();
    let old = actors.old.cview_camera_ref();
    format!(
        "aliases {} {} {} {} {} {}",
        i32::from(view.reference_address() == actors.owner.view_ref().reference_address()),
        i32::from(view.object_address() == actors.owner.view_ref().object_address()),
        i32::from(camera.reference_address() != fresh.reference_address()),
        i32::from(camera.reference_address() == old.reference_address()),
        camera.observe().ref_count(),
        fresh.observe().ref_count()
    )
}
fn capture_clip_operation(actors: &ClipRetargetActors, append: bool) -> Vec<String> {
    *actors.probe.replace_once.write() = true;
    actors.probe.rows.write().clear();
    let view = actors.owner.view_ref();
    let camera = actors.owner.camera_ref();
    let mut rows = vec![if append { "add" } else { "remove" }.to_string()];
    if append {
        actors.owner.add_clip_plane(&actors.plane);
    } else {
        actors.owner.remove_clip_plane(&actors.plane);
    }
    rows.extend(actors.probe.rows.read().iter().cloned());
    rows.push(record_final(actors));
    rows.push(record_aliases(actors, view, camera.as_ref()));
    rows
}
#[test]
fn actual_virtual_clipping_retains_old_returned_field_and_rereads_protected_driver_slot() {
    let actors = clip_retarget_actors().unwrap();
    let mut rows = capture_clip_operation(&actors, true);
    actors
        .owner
        .driver_view_slot
        .assign(&actors.old.cast::<dyn Graphic3dCView>())
        .unwrap();
    rows.extend(capture_clip_operation(&actors, false));
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/v3d-clip-view-retarget-native.txt")
    );
}
