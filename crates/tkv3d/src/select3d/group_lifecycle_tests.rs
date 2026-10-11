//! Native BVH mutation staleness is deliberate; safe full-inside paths and explicit repairs expose it.
use super::group_oracle_geometry::group_children;
use super::group_oracle_snapshot::snapshot_group_match;
use super::quadric_oracle_volume::quadric_volume;
use super::triangulation_oracle_snapshot::compare_oracle_snapshot;
use super::{
    Select3DSensitiveEntity, Select3DSensitiveEntityMembers, Select3DSensitiveGroup,
    Select3DSensitiveSphere,
};
use crate::select_mgr::SelectMgrEntityOwner;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
struct GroupLifecycleScenario {
    group: Handle<Select3DSensitiveGroup>,
    children: Vec<Handle<dyn Select3DSensitiveEntity>>,
}
impl GroupLifecycleScenario {
    fn new() -> Self {
        let owner = SelectMgrEntityOwner::new(13).cast();
        let children = group_children(3, &owner);
        let group = Handle::new(Select3DSensitiveGroup::new(&owner, false));
        group.add_all(&children);
        Self { group, children }
    }
    fn mutate(&mut self, step: i32) {
        match step {
            1 => self.group.bvh().unwrap(),
            2 => self.append_far_member(),
            4 | 6 | 13 => self.group.set_base().mark_dirty(),
            5 => self.group.remove(&self.children[0]),
            7 => {
                self.group.set_match_type(true);
                self.group.set_check_overlap_all(true);
            }
            8 => self.group.set(&SelectMgrEntityOwner::new(29).cast()),
            9 | 12 => self.group.add_all(&self.children),
            11 => self.group.clear(),
            _ => (),
        }
    }
    fn append_far_member(&mut self) {
        let extra = Handle::new(Select3DSensitiveSphere::new(
            &self.group.owner_id(),
            Point3::new(5., 0., 0.),
            0.25,
        ))
        .cast();
        self.children.push(extra);
        self.group.add(self.children.last().unwrap());
    }
    fn probe_for_step(&self, step: i32) -> Handle<Select3DSensitiveGroup> {
        if step == 10 {
            return self.group.get_connected().unwrap().cast();
        }
        // Connected state is observed separately; other steps mutate the original group.
        self.group.clone()
    }
    fn replay_step(&mut self, step: i32, native: &str) {
        self.mutate(step);
        let probe = self.probe_for_step(step);
        let volume_kind = match step {
            0 | 2 | 5 | 8 | 9 | 12 => 8,
            7 => 5,
            _ => 0,
        };
        let volume = quadric_volume(volume_kind, false, false, false);
        let columns = snapshot_group_match(probe.get().unwrap(), &self.children, 0, step, &volume);
        compare_oracle_snapshot(&columns, native);
    }
}
#[test]
fn native_group_mutation_preserves_dirty_detection_owner_and_connected_defaults() {
    let mut scenario = GroupLifecycleScenario::new();
    let expected = include_str!("../../tests/fixtures/sensitive-group-lifecycle.txt");
    for (step, native) in expected.lines().enumerate() {
        scenario.replay_step(step as i32, native);
    }
    assert_eq!(expected.lines().count(), 14);
}
use super::Select3DSensitiveSet;
