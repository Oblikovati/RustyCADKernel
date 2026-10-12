//! Rust-only replay of actual native derived dispatch, live replacements and exception ordering.
use super::super::{view_cube_interaction_fixture::assert_widget_camera, *};
use super::view_cube_animation_source_fixture::*;
use crate::v3d::V3dTypeOfOrientation;
use std::sync::atomic::Ordering;
const TRANSPORT: &str = include_str!("../../tests/fixtures/view-cube-animation-source.txt");
#[test]
fn native_derived_camera_dispatch_counts_replacements_and_failure_order_replay() {
    for scenario in 0..=12 {
        let proof = CubeTransportScene::new(scenario);
        replay_transport_case(&proof);
        let expected = transport_rows(scenario, false);
        assert_eq!(*proof.trace.rows.read(), expected, "scenario={scenario}");
        let camera = transport_rows(scenario, true);
        let channels = camera[0].split_whitespace().skip(1).collect::<Vec<_>>();
        assert_widget_camera(&proof.scene.view.camera(), &channels);
    }
}
fn replay_transport_case(proof: &CubeTransportScene) {
    let result = run_transport_case(proof);
    if let Err(failure) = result {
        proof
            .trace
            .rows
            .write()
            .push(format!("failure {} {failure}", proof.trace.scenario));
    }
    proof.snapshot("final");
}
fn run_transport_case(
    proof: &CubeTransportScene,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let cube = &proof.scene.cube;
    let owner = AisViewCubeOwner::new(cube, V3dTypeOfOrientation::Xpos, 5);
    cube.start_animation(&owner)?;
    proof.snapshot("started");
    seek_transport_failure_frame(proof);
    let continuing = cube.update_animation(false)?;
    proof.trace.rows.write().push(format!(
        "return {} {}",
        proof.trace.scenario, continuing as i32
    ));
    if proof.trace.scenario == 0 {
        replay_transport_completion(proof)?;
    }
    Ok(())
}
fn replay_transport_completion(
    proof: &CubeTransportScene,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    proof.owners[0]
        .next
        .store(0.5_f64.to_bits(), Ordering::SeqCst);
    let continuing = proof.scene.cube.update_animation(false)?;
    proof
        .trace
        .rows
        .write()
        .push(format!("end-return 0 {}", continuing as i32));
    Ok(())
}
fn transport_rows(scenario: usize, camera: bool) -> Vec<String> {
    TRANSPORT
        .lines()
        .filter(|line| {
            let row = line.split_whitespace().collect::<Vec<_>>();
            row[1] == scenario.to_string() && (row[0] == "camera") == camera
        })
        .map(str::to_owned)
        .collect()
}
const CLICK_FAILURE: &str = include_str!("../../tests/fixtures/view-cube-click-failure.txt");
#[test]
fn native_click_failure_propagates_from_owner_and_context_without_selection() {
    let actual = (0..2)
        .flat_map(replay_failed_transport_click)
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        CLICK_FAILURE.lines().map(str::to_owned).collect::<Vec<_>>()
    );
}
fn replay_failed_transport_click(entry: usize) -> Vec<String> {
    let proof = CubeTransportScene::new(3);
    let owner = AisViewCubeOwner::new(&proof.scene.cube, V3dTypeOfOrientation::Xpos, 5);
    proof.scene.context.fields.write().last_picked = owner.cast();
    let failure = transport_click_result(&proof, &owner, entry).unwrap_err();
    let animation = &proof.owners[0];
    let mut rows = proof.trace.rows.read().clone();
    rows.push(format!("click-failure {entry} {failure}"));
    rows.push(format!(
        "click-state {entry} {} {} {} {} {}",
        proof.scene.context.nb_selected(),
        proof.scene.context.has_detected() as i32,
        animation.is_stopped() as i32,
        animation.target_view_ref().is_null() as i32,
        animation.ref_count()
    ));
    rows
}
fn transport_click_result(
    proof: &CubeTransportScene,
    owner: &rustycad_tkernel::standard::Handle<AisViewCubeOwner>,
    entry: usize,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    use crate::select_mgr::SelectMgrEntityOwnerSource;
    if entry == 1 {
        return proof
            .scene
            .context
            .select_detected(AisSelectionScheme::Replace)
            .map(|_| ());
    }
    rustycad_tkernel::standard::catch_native_failure(|| dispatch_transport_owner_click(owner))
}
fn dispatch_transport_owner_click(
    owner: &rustycad_tkernel::standard::Handle<AisViewCubeOwner>,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    use crate::select_mgr::SelectMgrEntityOwnerSource;
    use rustycad_tkernel::ncollection::Vec2;
    use rustycad_tkservice::aspect::{ASPECT_VKEY_FLAGS_SHIFT, ASPECT_VKEY_MOUSE_RIGHT_BUTTON};
    assert!(owner.handle_mouse_click(
        Vec2::new(13, 17),
        ASPECT_VKEY_MOUSE_RIGHT_BUTTON,
        ASPECT_VKEY_FLAGS_SHIFT,
        true
    ));
    Ok(())
}

#[test]
fn borrowed_derived_animation_access_does_not_increment_native_count_and_can_replace_slot() {
    let proof = CubeTransportScene::new(0);
    let alias = proof.scene.cube.view_animation_ref();
    alias.with_animation(|animation| {
        assert_eq!(animation.unwrap().transient_base().ref_count(), 2);
        proof.trace.replace(1);
        assert_eq!(animation.unwrap().transient_base().ref_count(), 1);
    });
    assert_eq!(alias.object_address(), proof.owners[1].object_address());
    assert_eq!(proof.owners[0].ref_count(), 1);
    assert_eq!(proof.owners[1].ref_count(), 2);
}

fn seek_transport_failure_frame(proof: &CubeTransportScene) {
    if matches!(proof.trace.scenario, 5 | 7 | 8) {
        proof.owners[0]
            .next
            .store(0.5_f64.to_bits(), Ordering::SeqCst);
    }
}
