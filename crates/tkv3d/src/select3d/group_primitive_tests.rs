//! Explicit primitive swaps and BVH sorting retain native boxes/axes and empty removal NaNs.
use super::group_oracle_geometry::group_children;
use super::triangulation_oracle_snapshot::compare_oracle_snapshot;
use super::{Select3DSensitiveEntity, Select3DSensitiveGroup, Select3DSensitiveSet};
use crate::select_mgr::SelectMgrEntityOwner;
fn primitive_snapshot(group: &Select3DSensitiveGroup, kind: i32, step: i32) -> Vec<f64> {
    let mut columns = vec![
        f64::from(kind),
        f64::from(step),
        f64::from(group.size()),
        f64::from(group.nb_sub_elements()),
        f64::from(group.to_build_bvh()),
    ];
    let center = group.center_of_geometry();
    columns.extend([center.x(), center.y(), center.z()]);
    for index in 0..group.size() {
        append_primitive_columns(&mut columns, group, index);
    }
    columns
}
fn append_primitive_columns(columns: &mut Vec<f64>, group: &Select3DSensitiveGroup, index: i32) {
    let bounds = group.element_box(index);
    let (min, max) = (bounds.corner_min(), bounds.corner_max());
    columns.extend([min.x, min.y, min.z, max.x, max.y, max.z]);
    for axis in [0, 1, 2, -1, 4] {
        columns.push(group.center(index, axis).unwrap());
    }
}
fn primitive_scenario(kind: i32, expected: &mut std::str::Lines<'_>) {
    let owner = SelectMgrEntityOwner::new(13).cast();
    let children = group_children(kind, &owner);
    let group = Select3DSensitiveGroup::with_entities(&owner, &children, false);
    for step in 0..7 {
        mutate_primitive_scenario(&group, &children, step);
        compare_oracle_snapshot(
            &primitive_snapshot(&group, kind, step),
            expected.next().unwrap(),
        );
    }
}
#[test]
fn native_group_primitive_boxes_axes_swaps_and_empty_removal_are_preserved() {
    let mut expected = include_str!("../../tests/fixtures/sensitive-group-primitives.txt").lines();
    for kind in 1..=4 {
        primitive_scenario(kind, &mut expected);
    }
    assert!(expected.next().is_none());
}

fn mutate_primitive_scenario(
    group: &Select3DSensitiveGroup,
    children: &[rustycad_tkernel::standard::Handle<dyn Select3DSensitiveEntity>],
    step: i32,
) {
    match step {
        1 | 3 => group.swap(0, group.size() - 1),
        2 => group.bvh().unwrap(),
        4 => {
            for child in children {
                group.remove(child);
            }
        }
        5 => group.clear(),
        6 => group.add_all(&[]),
        _ => (),
    }
}
