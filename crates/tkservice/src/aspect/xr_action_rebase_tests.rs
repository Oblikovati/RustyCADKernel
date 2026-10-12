//! Native const action-slot references preserve physical identity across equal-length role-array rebasing.
use super::*;
use rustycad_tkernel::{
    standard::{Handle, catch_native_failure},
    tcollection::AsciiString,
};
const LEFT_ROLE: AspectXRTrackedDeviceRole = AspectXRTrackedDeviceRole::LeftHand;
const HAPTIC_KIND: AspectXRGenericAction = AspectXRGenericAction::OutputHaptic;
struct RebasedActionScene {
    session: AspectOpenVRSession,
    first: Handle<AspectXRAction>,
    second: Handle<AspectXRAction>,
}
fn rebased_action_scene() -> RebasedActionScene {
    let scene = RebasedActionScene {
        session: AspectOpenVRSession::default(),
        first: Handle::new(AspectXRAction::new(
            &AsciiString::from("/first"),
            AspectXRActionType::OutputHaptic,
        )),
        second: Handle::new(AspectXRAction::new(
            &AsciiString::from("/second"),
            AspectXRActionType::OutputHaptic,
        )),
    };
    scene.session.xr_session_base().fields.write().role_actions[LEFT_ROLE as usize]
        [HAPTIC_KIND as i32] = scene.first.clone();
    scene
}
fn rebase_action_array_state(
    session: &AspectOpenVRSession,
) -> (i32, i32, *const Handle<AspectXRAction>) {
    let fields = session.xr_session_base().fields.read();
    let actions = &fields.role_actions[LEFT_ROLE as usize];
    (
        actions.lower(),
        actions.upper(),
        std::ptr::from_ref(actions.at(HAPTIC_KIND as usize).unwrap()),
    )
}
fn rebase_action_row(
    scene: &RebasedActionScene,
    alias: &AspectXrActionRef,
    phase: i32,
) -> [i32; 9] {
    let (lower, upper, address) = rebase_action_array_state(&scene.session);
    [
        phase,
        lower,
        upper,
        i32::from(alias.reference_address() == address),
        i32::from(alias.object_address() == scene.first.object_address()),
        i32::from(alias.object_address() == scene.second.object_address()),
        scene.first.ref_count(),
        scene.second.ref_count(),
        i32::from(fresh_rebased_action_is_null(scene)),
    ]
}
fn fresh_rebased_action_is_null(scene: &RebasedActionScene) -> bool {
    let fresh = scene
        .session
        .generic_action_ref(LEFT_ROLE, HAPTIC_KIND)
        .unwrap();
    fresh.is_null()
}
fn native_rebased_action_rows() -> Vec<Vec<i32>> {
    include_str!("../../tests/fixtures/xr-action-rebase-native.txt")
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|word| word.parse().unwrap())
                .collect()
        })
        .collect()
}
#[test]
fn generic_action_reference_rereads_original_physical_slot_after_signed_rebase() {
    let scene = rebased_action_scene();
    let alias = scene
        .session
        .generic_action_ref(LEFT_ROLE, HAPTIC_KIND)
        .unwrap();
    let expected = native_rebased_action_rows();
    for (phase, row) in expected.iter().enumerate() {
        apply_rebased_action_phase(&scene, phase);
        assert_eq!(
            rebase_action_row(&scene, &alias, phase as i32).as_slice(),
            row.as_slice()
        );
    }
}
fn apply_rebased_action_phase(scene: &RebasedActionScene, phase: usize) {
    match phase {
        1 => rebase_role_actions(&scene.session, 9),
        2 => {
            scene.session.xr_session_base().fields.write().role_actions[LEFT_ROLE as usize]
                [HAPTIC_KIND as i32 + 9] = scene.second.clone()
        }
        3 => rebase_role_actions(&scene.session, 0),
        _ => {}
    }
}

fn rebase_role_actions(session: &AspectOpenVRSession, lower: i32) {
    let mut fields = session.xr_session_base().fields.write();
    let array = &mut fields.role_actions[LEFT_ROLE as usize];
    let upper = lower + array.len() as i32 - 1;
    array.resize_bounds(lower, upper, false).unwrap();
}
#[test]
fn reallocated_generic_action_reference_raises_typed_failure_instead_of_following_index() {
    let session = AspectOpenVRSession::default();
    let alias = session.generic_action_ref(LEFT_ROLE, HAPTIC_KIND).unwrap();
    session.xr_session_base().fields.write().role_actions[LEFT_ROLE as usize]
        .resize_bounds(0, 0, false)
        .unwrap();
    assert!(catch_native_failure(|| Ok(alias.observe())).is_err());
    assert!(catch_native_failure(|| Ok(alias.is_same_reference(&alias))).is_err());
}
