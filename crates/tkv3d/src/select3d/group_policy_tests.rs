//! Native changing-type callbacks and nearest-pick tie/NaN behavior remain ordered.
use super::group_fault_child_fake::{GroupFaultContext, fault_children, fault_trace_columns};
use super::group_oracle_snapshot::group_snapshot;
use super::group_policy_volume_fake::GroupPolicyVolumeFake;
use super::quadric_oracle_snapshot::quadric_seed;
use super::triangulation_oracle_snapshot::compare_oracle_snapshot;
use super::{Select3DSensitiveEntity, Select3DSensitiveGroup};
use crate::select_mgr::SelectMgrEntityOwner;
use rustycad_tkernel::standard::Handle;
use std::sync::Arc;
struct GroupPolicyScenario {
    group: Select3DSensitiveGroup,
    children: Vec<Handle<dyn Select3DSensitiveEntity>>,
    context: Arc<GroupFaultContext>,
    ordinal: i32,
}
impl GroupPolicyScenario {
    fn new(ordinal: i32) -> Self {
        let owner = SelectMgrEntityOwner::new(13).cast();
        let mut context = GroupFaultContext::new(2);
        context.miss = ordinal / 4 % 4;
        context.depth = ordinal % 4;
        let context = Arc::new(context);
        let policy = ordinal / 16 % 4;
        let children = fault_children(&owner, &context);
        let group = Select3DSensitiveGroup::with_entities(&owner, &children, policy / 2 != 0);
        group.set_check_overlap_all(policy % 2 != 0);
        Self {
            group,
            children,
            context,
            ordinal,
        }
    }
    fn replay(&self, expected: &mut std::str::Lines<'_>) {
        let ordinal = self.ordinal;
        let group = &self.group;
        let context = &self.context;
        let children = &self.children;
        let volume = GroupPolicyVolumeFake::new(group, context.clone(), ordinal / 64);
        context.trace.lock().unwrap().clear();
        let mut pick = quadric_seed();
        let hit = group.matches(&volume, &mut pick).unwrap();
        compare_oracle_snapshot(
            &fault_trace_columns(context, ordinal, 0),
            expected.next().unwrap(),
        );
        let columns = group_snapshot(group, children, ordinal, 0, false, i32::from(hit), &pick);
        compare_oracle_snapshot(&columns, expected.next().unwrap());
    }
}

#[test]
fn native_group_policy_reads_follow_each_virtual_call_and_preserve_min_nan_ties() {
    let mut expected = super::group_policy_fixtures::GROUP_POLICY_NATIVE.lines();
    for ordinal in 0..320 {
        GroupPolicyScenario::new(ordinal).replay(&mut expected);
    }
    assert!(expected.next().is_none());
}

#[test]
fn group_checked_failure_bridge_resumes_unrelated_rust_panics() {
    let owner = SelectMgrEntityOwner::new(13).cast();
    let group = Select3DSensitiveGroup::new(&owner, false);
    let volume = GroupPolicyVolumeFake::new(&group, Arc::new(GroupFaultContext::new(2)), 99);
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        group.matches(&volume, &mut quadric_seed())
    }))
    .unwrap_err();
    assert_eq!(*panic.downcast::<i32>().unwrap(), 73);
    assert!(group.entities_ref().is_empty());
}
