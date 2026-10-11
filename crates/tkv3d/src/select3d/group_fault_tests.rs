//! Defined native virtual-child failures retain partial state and repair through explicit reconstruction.
use super::group_fault_child_fake::{GroupFaultContext, fault_children, fault_trace_columns};
use super::group_oracle_snapshot::group_snapshot;
use super::quadric_oracle_snapshot::quadric_seed;
use super::quadric_oracle_volume::quadric_volume;
use super::triangulation_failure::catch_triangulation_failure;
use super::triangulation_oracle_snapshot::compare_oracle_snapshot;
use super::{Select3DSensitiveEntity, Select3DSensitiveGroup, Select3DSensitiveSet};
use crate::select_basics::SelectBasicsPickResult;
use crate::select_mgr::SelectMgrEntityOwner;
use rustycad_tkernel::standard::{Failure, Handle};
use std::sync::{Arc, atomic::Ordering};
struct GroupFaultScenario {
    group: Select3DSensitiveGroup,
    children: Vec<Handle<dyn Select3DSensitiveEntity>>,
    context: Arc<GroupFaultContext>,
    ordinal: i32,
    operation: i32,
    stage: i32,
}
impl GroupFaultScenario {
    fn new(ordinal: i32, operation: i32, stage: i32) -> Self {
        let owner = SelectMgrEntityOwner::new(13).cast();
        let context = Arc::new(GroupFaultContext::new(if operation == 7 { 1 } else { 2 }));
        let children = fault_children(&owner, &context);
        let group = Select3DSensitiveGroup::new(&owner, true);
        group.add(&children[0]);
        if operation >= 2 {
            group.add(&children[1]);
            group.add(&children[2]);
        }
        Self {
            group,
            children,
            context,
            ordinal,
            operation,
            stage,
        }
    }
    fn operate(&self, pick: &mut SelectBasicsPickResult) -> Result<(), Failure> {
        if self.operation <= 3 {
            self.mutate_membership();
            return Ok(());
        }
        self.select_or_build(pick)
    }
    fn select_or_build(&self, pick: &mut SelectBasicsPickResult) -> Result<(), Failure> {
        match self.operation {
            4 => {
                self.group.get_connected()?;
            }
            5 => {
                self.group
                    .matches(&quadric_volume(8, false, false, false), pick)?;
            }
            6 => {
                self.group.set_base().mark_dirty();
                self.group.bvh()?;
            }
            _ => {
                self.group.element_box(0);
            }
        }
        Ok(())
    }
    fn mutate_membership(&self) {
        match self.operation {
            0 => self.group.add(&self.children[1]),
            1 => self.group.add_all(&self.children[1..]),
            2 => self.group.remove(&self.children[2]),
            _ => self.group.set(&SelectMgrEntityOwner::new(29).cast()),
        }
    }
    fn repair(&self) -> Result<(), Failure> {
        let members = self.group.entities();
        self.group.clear();
        self.group.add_all(&members);
        self.group.set_base().mark_dirty();
        self.group.bvh()
    }
    fn prepare_step(&self, step: i32) {
        self.context
            .stage
            .store(if step == 0 { self.stage } else { 0 }, Ordering::Relaxed);
        self.context.trace.lock().unwrap().clear();
    }
    fn replay_step(&self, step: i32, expected: &mut std::str::Lines<'_>) {
        self.prepare_step(step);
        let mut pick = quadric_seed();
        let outcome = catch_triangulation_failure(|| {
            if step == 1 {
                self.repair()
            } else {
                self.operate(&mut pick)
            }
        });
        assert_group_checked_failure(outcome.as_ref().err());
        compare_oracle_snapshot(
            &fault_trace_columns(&self.context, self.ordinal, step),
            expected.next().unwrap(),
        );
        self.context.stage.store(0, Ordering::Relaxed);
        self.compare_fault_snapshot(step, outcome.is_err(), &pick, expected.next().unwrap());
    }
    fn compare_fault_snapshot(
        &self,
        step: i32,
        failure: bool,
        pick: &SelectBasicsPickResult,
        native: &str,
    ) {
        let columns = group_snapshot(
            &self.group,
            &self.children,
            self.ordinal,
            step,
            failure,
            -1,
            pick,
        );
        compare_oracle_snapshot(&columns, native);
    }
}
#[test]
fn native_group_virtual_failures_preserve_partial_writes_then_repair_and_retest() {
    let cases = group_fault_cases();
    let mut expected = include_str!("../../tests/fixtures/sensitive-group-faults.txt").lines();
    for (ordinal, (operation, stage)) in cases.into_iter().enumerate() {
        let scenario = GroupFaultScenario::new(ordinal as i32, operation, stage);
        for step in 0..3 {
            scenario.replay_step(step, &mut expected);
        }
    }
    assert!(expected.next().is_none());
}
fn group_fault_cases() -> [(i32, i32); 12] {
    [
        (0, 1),
        (0, 2),
        (1, 1),
        (1, 2),
        (2, 1),
        (2, 2),
        (3, 4),
        (4, 5),
        (5, 3),
        (6, 1),
        (6, 2),
        (7, 1),
    ]
}

fn assert_group_checked_failure(failure: Option<&Failure>) {
    if let Some(failure) = failure {
        assert_eq!(failure.exception_type(), "Standard_Failure");
        assert!(failure.message().contains("expected successful"));
    }
}
