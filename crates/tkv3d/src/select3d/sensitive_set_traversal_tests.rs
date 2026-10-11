//! Traversal extraction regressions cover callback ordering and unordered depths.
use super::*;
use crate::select3d::primitive_array_matching_tests::{MatchScenario, matching_probe_owned};
use rustycad_tkernel::standard::Handle;
#[path = "sensitive_set_traversal_volume_fake.rs"]
mod volume_fake;
use volume_fake::ChangingPolicyVolume;

#[path = "sensitive_set_traversal_set_fake.rs"]
mod set_fake;
use set_fake::CallbackSensitiveSet;

fn regression_primitive() -> CallbackSensitiveSet {
    let probe = matching_probe_owned(MatchScenario::new(1032), &Handle::default(), 2);
    CallbackSensitiveSet::new(probe)
}
fn regression_range() -> ElementRange {
    ElementRange {
        first: 0,
        last: 2,
        is_full_inside: false,
    }
}
fn run_regression_range(
    probe: &CallbackSensitiveSet,
    volume: &ChangingPolicyVolume,
) -> (SelectBasicsPickResult, i32) {
    let mut pick = SelectBasicsPickResult::default();
    let mut matches = -1;
    let range = regression_range();
    let hit = process_elements(probe, volume, &range, false, &mut pick, &mut matches).unwrap();
    assert!(hit);
    (pick, matches)
}
#[test]
fn callbacks_recheck_overlap_policy_for_every_element() {
    let probe = regression_primitive();
    let volume = ChangingPolicyVolume::new(true);
    let (pick, matches) = run_regression_range(&probe, &volume);
    assert_eq!(
        *volume.calls.borrow(),
        ["policy", "overlap", "policy", "inside", "policy", "inside"]
    );
    assert_eq!(matches, 2);
    assert_eq!(pick.depth(), 5.0);
}
#[test]
fn overlap_callbacks_reuse_one_scratch_pick_per_range() {
    let probe = regression_primitive();
    let volume = ChangingPolicyVolume::new(false);
    let (_, matches) = run_regression_range(&probe, &volume);
    assert_eq!(*volume.incoming_depths.borrow(), [f64::MAX, 5.0, 5.0]);
    assert_eq!(probe.set_base().detected_idx(), 0);
    assert_eq!(matches, 2);
}
#[test]
fn unordered_nearest_depths_preserve_pick_and_detected_index() {
    for (nearest, candidate) in [(1.0, f64::NAN), (f64::NAN, 1.0)] {
        let probe = regression_primitive();
        let mut pick = SelectBasicsPickResult::default();
        pick.set_depth(nearest);
        let mut matches = -1;
        let mut state = ElementMatchState::new(&mut pick, &mut matches);
        state.scratch.set_depth(candidate);
        state.retain_nearest(&probe, 17);
        assert_eq!(probe.set_base().detected_idx(), -1);
        assert!(pick.depth().is_nan() && nearest.is_nan() || pick.depth() == nearest);
    }
}
#[test]
fn failed_tree_callback_restores_the_taken_tree() {
    let probe = regression_primitive();
    let mut first_nodes = 0;
    let failed: Result<(), Failure> = with_built_tree(&probe, |tree, _| {
        first_nodes = tree.node_info_buffer().len();
        Err(Failure::with_message(
            "Standard_ProgramError",
            "test callback failed; expected restored tree",
        ))
    });
    assert!(failed.is_err());
    assert!(!probe.set_base().is_dirty());
    let restored = with_built_tree(&probe, |tree, _| Ok(tree.node_info_buffer().len())).unwrap();
    assert_eq!(first_nodes, restored);
    assert!(restored > 0);
}
