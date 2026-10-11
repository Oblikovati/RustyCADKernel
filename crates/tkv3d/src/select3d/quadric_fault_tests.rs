//! Native callback faults prove mutation order before exceptions and subsequent repair.
use super::Select3DSensitiveEntityMembers;
use super::quadric_fault_volume_fake::QuadricFaultVolumeFake;
use super::quadric_oracle_geometry::quadric_entity;
use super::quadric_oracle_snapshot::{quadric_seed, quadric_snapshot};
use super::triangulation_oracle_snapshot::compare_oracle_snapshot;
use crate::select_mgr::SelectMgrEntityOwner;

struct QuadricFaultScenario {
    ordinal: i32,
    owner: rustycad_tkernel::standard::Handle<dyn crate::select_mgr::SelectMgrEntityOwnerSource>,
    probe: rustycad_tkernel::standard::Handle<dyn super::Select3DSensitiveEntity>,
    volume: QuadricFaultVolumeFake,
}
impl QuadricFaultScenario {
    fn new(ordinal: i32) -> Self {
        let family = (ordinal - 900) / 5;
        let owner = SelectMgrEntityOwner::new(13).cast();
        let kind = match family {
            0 => 0,
            1 => 11,
            _ => 19,
        };
        let probe = quadric_entity(kind, &owner);
        probe.get().unwrap().set_sensitivity_factor(9).unwrap();
        Self {
            ordinal,
            owner,
            probe,
            volume: QuadricFaultVolumeFake::new((ordinal - 900) % 5 + 1),
        }
    }
    fn replay_step(&self, step: i32, native: &mut std::str::Lines<'_>) {
        let snapshot = self.step_snapshot(step);
        compare_oracle_snapshot(&snapshot, native.next().unwrap());
        assert_quadric_callback_trace(self.volume.trace.get(), native.next().unwrap());
        self.volume.stage.set(0);
        self.volume.trace.set(0);
    }
    fn step_snapshot(&self, step: i32) -> Vec<f64> {
        let mut pick = quadric_seed();
        let result = self.probe.get().unwrap().matches(&self.volume, &mut pick);
        assert_quadric_callback_failure(result.as_ref().err(), self.volume.stage.get());
        quadric_snapshot(
            &self.probe,
            &self.owner,
            self.ordinal,
            step,
            result.is_err(),
            result.map_or(-1, i32::from),
            &pick,
        )
    }
}
fn assert_quadric_callback_failure(
    failure: Option<&rustycad_tkernel::standard::Failure>,
    stage: i32,
) {
    let Some(failure) = failure else {
        assert!(!matches!(stage, 1..=3));
        return;
    };
    assert_eq!(failure.exception_type(), "Standard_Failure");
    assert!(
        failure
            .message()
            .contains(&format!("stage {stage}: expected successful"))
    );
}
fn assert_quadric_callback_trace(actual: i32, native: &str) {
    let expected = native
        .strip_prefix("trace ")
        .unwrap()
        .parse::<i32>()
        .unwrap();
    assert_eq!(actual, expected);
}
fn fault_quadric_scenario(ordinal: i32, native: &mut std::str::Lines<'_>) {
    let scenario = QuadricFaultScenario::new(ordinal);
    scenario.replay_step(0, native);
    scenario.replay_step(1, native);
}
#[test]
fn native_quadric_callback_failures_preserve_pick_and_sphere_assignment_order() {
    let mut native = include_str!("../../tests/fixtures/sensitive-quadrics-faults.txt").lines();
    for ordinal in 900..915 {
        fault_quadric_scenario(ordinal, &mut native);
    }
    assert!(native.next().is_none());
}
