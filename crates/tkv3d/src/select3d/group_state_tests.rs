//! Native defaults, every visible accessor and RTTI identity are checked through core and application views.
use super::group_hook_fake::GroupHookFake;
use super::group_oracle_geometry::group_children;
use super::triangulation_oracle_snapshot::compare_oracle_snapshot;
use super::{
    Select3DSensitiveEntity, Select3DSensitiveEntityMembers, Select3DSensitiveGroup,
    Select3DSensitiveGroupSource, Select3DSensitiveSet,
};
use crate::select_mgr::SelectMgrEntityOwner;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
enum GroupStateProbe {
    Core(Handle<Select3DSensitiveGroup>),
    Application(Handle<GroupHookFake>),
}
impl GroupStateProbe {
    fn core(&self) -> &Select3DSensitiveGroup {
        match self {
            Self::Core(group) => group.get().unwrap(),
            Self::Application(group) => &group.get().unwrap().core,
        }
    }
    fn entity(&self) -> Handle<dyn Select3DSensitiveEntity> {
        match self {
            Self::Core(group) => group.clone().cast(),
            Self::Application(group) => group.clone().cast(),
        }
    }
    fn source(&self) -> Handle<dyn Select3DSensitiveGroupSource> {
        match self {
            Self::Core(group) => group.clone().cast(),
            Self::Application(group) => group.clone().cast(),
        }
    }
}
fn group_state_probe(kind: i32) -> (GroupStateProbe, Vec<Handle<dyn Select3DSensitiveEntity>>) {
    let owner = if kind == 0 {
        Handle::default()
    } else {
        SelectMgrEntityOwner::new(13).cast()
    };
    let children = group_children(if kind <= 1 { 0 } else { 5 }, &owner);
    let core = construct_state_group(kind, &owner, &children);
    let probe = if kind == 3 {
        GroupStateProbe::Application(Handle::new(GroupHookFake::new(core, 0)))
    } else {
        GroupStateProbe::Core(Handle::new(core))
    };
    (probe, children)
}
fn construct_state_group(
    kind: i32,
    owner: &Handle<dyn crate::select_mgr::SelectMgrEntityOwnerSource>,
    children: &[Handle<dyn Select3DSensitiveEntity>],
) -> Select3DSensitiveGroup {
    if kind <= 1 {
        Select3DSensitiveGroup::with_default_match_type(owner)
    } else if kind == 2 {
        Select3DSensitiveGroup::with_entities_default_match_type(owner, children)
    } else {
        let core = Select3DSensitiveGroup::new(owner, true);
        core.add_all(children);
        core
    }
}
fn group_rtti_row(probe: &GroupStateProbe, kind: i32) -> String {
    let descriptor = Select3DSensitiveGroup::static_type();
    let entity = probe.entity();
    verify_group_interface_identity(probe, &entity);
    let flags = group_rtti_flags(&entity, descriptor);
    let text = flags
        .into_iter()
        .map(|value| format!(" {}", i32::from(value)))
        .collect::<String>();
    format!(
        "{kind} 0 {} {} {}{text}",
        Select3DSensitiveGroup::TYPE_NAME,
        descriptor.name(),
        descriptor.parent().unwrap().name()
    )
}
fn verify_group_interface_identity(
    probe: &GroupStateProbe,
    entity: &Handle<dyn Select3DSensitiveEntity>,
) {
    let source = super::select3d_group_downcast(entity);
    assert_eq!(source.object_address(), probe.source().object_address());
    assert_eq!(entity.object_address(), source.object_address());
    assert_eq!(
        source
            .cast::<super::Select3DSensitiveGroupBase>()
            .object_address(),
        entity.object_address()
    );
}
fn group_rtti_flags(
    entity: &Handle<dyn Select3DSensitiveEntity>,
    descriptor: &rustycad_tkernel::standard::RuntimeType,
) -> [bool; 8] {
    [
        std::ptr::eq(descriptor, Select3DSensitiveGroup::static_type()),
        std::ptr::eq(entity.dynamic_type().unwrap(), descriptor),
        entity.is_instance(descriptor),
        entity.is_kind(descriptor),
        entity.is_kind(<dyn Select3DSensitiveSet>::static_type()),
        entity.is_kind(<dyn Select3DSensitiveEntity>::static_type()),
        entity
            .clone()
            .cast::<Select3DSensitiveGroup>()
            .object_address()
            == entity.object_address(),
        native_group_alias_is_parent(descriptor),
    ]
}
fn native_group_alias_is_parent(descriptor: &rustycad_tkernel::standard::RuntimeType) -> bool {
    // The native fixture's is_same assertion also binds the public alias to the actual parent.
    std::ptr::eq(
        descriptor.parent().unwrap(),
        <super::Select3DSensitiveGroupBase>::static_type(),
    )
}
fn group_state_columns(
    probe: &GroupStateProbe,
    children: &[Handle<dyn Select3DSensitiveEntity>],
    kind: i32,
    step: i32,
) -> Vec<f64> {
    let group = probe.core();
    let entity = probe.entity();
    let mut columns = group_initial_state_columns(group, &entity, kind, step);
    columns.extend([
        f64::from(entity.has_init_location()),
        f64::from(entity.transform_persistence().is_null()),
        f64::from(entity.flipper().is_null()),
        f64::from(group.last_detected_entity_index()),
    ]);
    append_group_state_geometry(&mut columns, group, &entity, children);
    columns
}
fn append_group_state_geometry(
    columns: &mut Vec<f64>,
    group: &Select3DSensitiveGroup,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    children: &[Handle<dyn Select3DSensitiveEntity>],
) {
    let center = entity.center_of_geometry();
    columns.extend([
        center.x(),
        center.y(),
        center.z(),
        f64::from(group.is_in(&Handle::default())),
    ]);
    columns.extend(children.iter().map(|child| f64::from(group.is_in(child))));
}
fn group_initial_state_columns(
    group: &Select3DSensitiveGroup,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    kind: i32,
    step: i32,
) -> Vec<f64> {
    vec![
        f64::from(kind),
        f64::from(step),
        f64::from(group.must_match_all()),
        f64::from(group.to_check_overlap_all()),
        f64::from(entity.nb_sub_elements()),
        f64::from(group.size()),
        f64::from(entity.to_build_bvh()),
        f64::from(entity.sensitivity_factor()),
        f64::from(entity.owner_id().is_null()),
    ]
}
fn replay_group_state(kind: i32, expected: &mut std::str::Lines<'_>) {
    let (probe, children) = group_state_probe(kind);
    assert_eq!(group_rtti_row(&probe, kind), expected.next().unwrap());
    for step in 1..=4 {
        match step {
            2 => {
                probe.core().set_match_type(false);
                probe.core().set_check_overlap_all(true);
            }
            3 if !children.is_empty() => probe.core().remove(&children[0]),
            4 => probe.entity().clear(),
            _ => (),
        }
        compare_oracle_snapshot(
            &group_state_columns(&probe, &children, kind, step),
            expected.next().unwrap(),
        );
    }
}
#[test]
fn native_group_remaining_accessors_default_arguments_and_inherited_rtti_are_complete() {
    let mut expected = include_str!("../../tests/fixtures/sensitive-group-state.txt").lines();
    for kind in 0..4 {
        replay_group_state(kind, &mut expected);
    }
    assert!(expected.next().is_none());
}

fn protected_state_probe() -> (
    Handle<GroupHookFake>,
    Vec<Handle<dyn Select3DSensitiveEntity>>,
) {
    let owner = SelectMgrEntityOwner::new(13).cast();
    let children = group_children(2, &owner);
    let core = Select3DSensitiveGroup::new(&owner, false);
    core.add_all(&children);
    core.swap(0, 2);
    core.bvh().unwrap();
    (Handle::new(GroupHookFake::new(core, 0)), children)
}
fn protected_owner_columns(
    group: &Select3DSensitiveGroup,
    children: &[Handle<dyn Select3DSensitiveEntity>],
    step: i32,
) -> Vec<f64> {
    let mut columns = vec![
        4.,
        f64::from(step),
        f64::from(group.owner_id().get().unwrap().priority()),
        super::group_oracle_snapshot::member_identity(&group.last_detected_entity(), children),
    ];
    columns.extend(
        children
            .iter()
            .map(|child| f64::from(child.owner_id().get().unwrap().priority())),
    );
    columns
}
#[test]
fn native_inherited_protected_group_state_mutation_preserves_owner_and_bvh_subobjects() {
    let (source, children) = protected_state_probe();
    let probe = GroupStateProbe::Application(source.clone());
    let mut expected =
        include_str!("../../tests/fixtures/sensitive-group-protected-state.txt").lines();
    for step in 0..3 {
        mutate_inherited_group_state(source.get().unwrap(), step);
        compare_oracle_snapshot(
            &group_state_columns(&probe, &children, 4, step),
            expected.next().unwrap(),
        );
        compare_oracle_snapshot(
            &protected_owner_columns(&source.core, &children, step),
            expected.next().unwrap(),
        );
    }
    assert!(expected.next().is_none());
}
#[test]
fn group_application_downcast_rejects_null_and_unrelated_native_types() {
    assert!(super::select3d_group_downcast(&Handle::default()).is_null());
    let point = Handle::new(super::Select3DSensitivePoint::new(
        &Handle::default(),
        rustycad_tkmath::gp::Point3::default(),
    ))
    .cast();
    assert!(super::select3d_group_downcast(&point).is_null());
}
use crate::select_mgr::SelectMgrEntityOwnerMembers;

fn mutate_inherited_group_state(source: &GroupHookFake, step: i32) {
    let mut fields = source.sensitive_base().fields.write();
    fields.owner_id = SelectMgrEntityOwner::new(29).cast();
    fields.sensitivity_factor = 7;
    drop(fields);
    source
        .set_base()
        .set_detected_idx(if step == 0 { -1 } else { step - 1 });
    source.set_base().mark_dirty();
}
