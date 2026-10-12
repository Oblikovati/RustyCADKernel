//! Native failure leaves prior callback edits in place; valid anchors exclude dangling accesses.
use super::view_clip_retarget_fixture::{ClipRetargetActors, clip_retarget_actors};
use rustycad_tkernel::standard::{Failure, catch_native_failure};

#[test]
fn inherited_clipping_failures_preserve_native_partial_mutations_and_counts() {
    let rows: Vec<String> = [true, false]
        .into_iter()
        .flat_map(|append| (1..=3).flat_map(move |stage| failure_rows(append, stage)))
        .collect();
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/v3d-clip-callback-failure-native.txt")
    );
}
fn failure_rows(append: bool, stage: i32) -> Vec<String> {
    let actors = clip_retarget_actors().unwrap();
    if !append {
        actors.owner.add_clip_plane(&actors.plane);
    }
    actors.probe.rows.write().clear();
    *actors.probe.replace_once.write() = true;
    *actors.probe.failure_stage.write() = stage;
    let failure = invoke_failure(&actors, append).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    let mut rows = vec![format!(
        "case {} {stage}",
        if append { "add" } else { "remove" }
    )];
    rows.push(format!("failure {}", failure.message()));
    rows.extend(actors.probe.rows.read().iter().cloned());
    rows.push(failure_state(&actors));
    rows
}
fn invoke_failure(actors: &ClipRetargetActors, append: bool) -> Result<(), Failure> {
    catch_native_failure(|| {
        if append {
            actors.owner.add_clip_plane(&actors.plane);
        } else {
            actors.owner.remove_clip_plane(&actors.plane);
        }
        Ok(())
    })
}
fn failure_state(actors: &ClipRetargetActors) -> String {
    let old = actors.old.clip_probe_slot.as_ref().unwrap();
    let next = actors.next.clip_probe_slot.as_ref().unwrap();
    format!(
        "state {} {} {} {} {} {} {} {} {}",
        i32::from(actors.owner.view_ref().object_address() == actors.next.object_address()),
        i32::from(old.object_address() == next.object_address()),
        old.observe().size(),
        next.observe().size(),
        actors.old.ref_count(),
        actors.next.ref_count(),
        old.observe().ref_count(),
        next.observe().ref_count(),
        actors.plane.ref_count()
    )
}
