//! Native composite inputs include identity duplicates and multi-leaf primitive permutations.
use super::quadric_oracle_geometry::quadric_entity;
use super::{
    Select3DSensitiveEntity, Select3DSensitiveGroup, Select3DSensitiveSegment,
    Select3DSensitiveSphere,
};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
pub(super) fn group_children(
    kind: i32,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Vec<Handle<dyn Select3DSensitiveEntity>> {
    match kind {
        0 => Vec::new(),
        1 => vec![quadric_entity(19, owner)],
        2 => mixed_group_children(owner),
        _ => sphere_group_children(kind, owner),
    }
}
fn mixed_group_children(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Vec<Handle<dyn Select3DSensitiveEntity>> {
    vec![
        Handle::new(Select3DSensitiveSegment::new(
            owner,
            Point3::new(-2., 0., 0.),
            Point3::new(2., 0., 0.),
        ))
        .cast(),
        quadric_entity(0, owner),
        Handle::new(Select3DSensitiveSphere::new(
            owner,
            Point3::new(3., 0., 0.5),
            0.5,
        ))
        .cast(),
    ]
}
fn sphere_group_children(
    kind: i32,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Vec<Handle<dyn Select3DSensitiveEntity>> {
    let count = match kind {
        3 => 9,
        4 => 17,
        _ => 3,
    };
    let mut children = (0..count)
        .map(|index| group_sphere(index, count, owner))
        .collect::<Vec<_>>();
    if kind == 5 {
        children.push(children[0].clone());
        children
            .push(Handle::new(Select3DSensitiveSphere::new(owner, Point3::default(), 0.25)).cast());
    }
    children
}
fn group_sphere(
    index: i32,
    count: i32,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Handle<dyn Select3DSensitiveEntity> {
    let center = Point3::new(
        f64::from((index * 7) % count - count / 2) * 0.3,
        0.,
        f64::from(index % 3 - 1) * 0.4,
    );
    // Deliberately scrambled centers force BVH sorting to differ from membership order.
    Handle::new(Select3DSensitiveSphere::new(owner, center, 0.25)).cast()
}
pub(super) fn construct_group(
    ordinal: i32,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> (
    Handle<Select3DSensitiveGroup>,
    Vec<Handle<dyn Select3DSensitiveEntity>>,
) {
    let children = group_children(ordinal / 288, owner);
    let policy = ordinal / 4 % 4;
    let group = if ordinal / 144 % 2 == 0 {
        individually_added_group(&children, owner, policy / 2 != 0)
    } else {
        Select3DSensitiveGroup::with_entities(owner, &children, policy / 2 != 0)
    };
    group.set_check_overlap_all(policy % 2 != 0);
    group.set_sensitivity_factor(9).unwrap();
    (Handle::new(group), children)
}
fn individually_added_group(
    children: &[Handle<dyn Select3DSensitiveEntity>],
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    must: bool,
) -> Select3DSensitiveGroup {
    let group = Select3DSensitiveGroup::new(owner, must);
    for child in children {
        group.add(child);
    }
    group
}
use super::Select3DSensitiveEntityMembers;
