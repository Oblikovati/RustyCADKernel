//! Native retained map/node references remain valid through other-node removal and map rehashing.
use super::group_oracle_geometry::group_children;
use super::triangulation_oracle_snapshot::compare_oracle_snapshot;
use super::{
    Select3DSensitiveEntity, Select3DSensitiveEntityMembers, Select3DSensitiveGroup,
    Select3DSensitivePoint,
};
use crate::select_mgr::{SelectMgrEntityOwner, SelectMgrEntityOwnerMembers};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
struct GroupReferenceScenario {
    group: Select3DSensitiveGroup,
    original: Vec<Handle<dyn Select3DSensitiveEntity>>,
}
impl GroupReferenceScenario {
    fn new() -> Self {
        let owner = SelectMgrEntityOwner::new(13).cast();
        let group = Select3DSensitiveGroup::new(&owner, false);
        let original = group_children(2, &owner);
        Self { group, original }
    }
    fn mutate(&self, step: i32) {
        match step {
            0 => {
                for child in &self.original {
                    self.group.add(child);
                }
            }
            1 => self.rehash_members(),
            2 => self.group.remove(&self.original[0]),
            3 => self.group.set(&SelectMgrEntityOwner::new(29).cast()),
            4 => self.remove_other_nodes(),
            6 => self.group.clear(),
            _ => unreachable!(),
        }
    }
    fn rehash_members(&self) {
        for index in 0..300 {
            let child = Handle::new(Select3DSensitivePoint::new(
                &self.group.owner_id(),
                Point3::new(f64::from(index), 0., 0.),
            ))
            .cast();
            self.group.add(&child);
        }
    }
    fn remove_other_nodes(&self) {
        for child in self.group.entities() {
            if child != self.original[2] {
                self.group.remove(&child);
            }
        }
    }
}
fn replay_group_references(scenario: &GroupReferenceScenario, expected: &mut std::str::Lines<'_>) {
    let first = scenario.group.entities_ref();
    let second = scenario.group.entities_ref();
    assert!(first.is_empty());
    scenario.mutate(0);
    let node_a = first.sub_entity(3).unwrap();
    let node_b = scenario.group.sub_entity_ref(3).unwrap();
    for step in 0..5 {
        if step != 0 {
            scenario.mutate(step);
        }
        let columns = group_reference_columns(scenario, step, &first, &second, &node_a, &node_b);
        compare_oracle_snapshot(&columns, expected.next().unwrap());
    }
}
#[test]
fn native_group_map_and_surviving_node_references_are_live_and_checked() {
    let scenario = GroupReferenceScenario::new();
    let mut expected = include_str!("../../tests/fixtures/sensitive-group-reference.txt").lines();
    replay_group_references(&scenario, &mut expected);
    for index in [-1, 0, 2] {
        let failure = scenario.group.sub_entity_ref(index).err().unwrap();
        assert_eq!(failure.exception_type(), "Standard_OutOfRange");
        assert!(failure.message().contains(&index.to_string()));
        compare_oracle_snapshot(&[5., f64::from(index), 1.], expected.next().unwrap());
    }
    let retained = scenario.group.entities_ref();
    scenario.mutate(6);
    compare_oracle_snapshot(
        &[6., retained.len() as f64, retained.len() as f64],
        expected.next().unwrap(),
    );
    assert!(retained.is_empty());
    assert!(expected.next().is_none());
}

fn group_reference_columns(
    scenario: &GroupReferenceScenario,
    step: i32,
    first: &super::Select3DGroupEntitiesRef<'_>,
    second: &super::Select3DGroupEntitiesRef<'_>,
    node_a: &super::Select3DGroupSubEntityRef<'_>,
    node_b: &super::Select3DGroupSubEntityRef<'_>,
) -> Vec<f64> {
    let mut columns = vec![f64::from(step), first.len() as f64, second.len() as f64];
    append_reference_nodes(&mut columns, scenario, node_a, node_b);
    columns
}
fn append_reference_nodes(
    columns: &mut Vec<f64>,
    scenario: &GroupReferenceScenario,
    node_a: &super::Select3DGroupSubEntityRef<'_>,
    node_b: &super::Select3DGroupSubEntityRef<'_>,
) {
    let observed = node_a.entity();
    columns.extend([
        f64::from(observed == scenario.original[2]),
        f64::from(node_b.entity() == scenario.original[2]),
    ]);
    columns.extend([
        f64::from(observed.owner_id().get().unwrap().priority()),
        current_member_index(&scenario.group, &observed) as f64,
    ]);
}
fn current_member_index(
    group: &Select3DSensitiveGroup,
    member: &Handle<dyn Select3DSensitiveEntity>,
) -> usize {
    // A surviving native node keeps its key after another key's removal relocates its numeric index.
    group
        .entities()
        .iter()
        .position(|value| value == member)
        .unwrap()
        + 1
}
