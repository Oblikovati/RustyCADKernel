//! Composite snapshots preserve outputs, cached geometry, detection and child sharing.
use super::quadric_oracle_snapshot::{append_quadric_pick, append_quadric_point};
use super::{
    Select3DSensitiveEntity, Select3DSensitiveEntityMembers, Select3DSensitiveGroup,
    Select3DSensitiveSet, Select3DSensitiveSphere,
};
use crate::select_basics::SelectBasicsPickResult;
use crate::select_mgr::SelectMgrEntityOwnerMembers;
use rustycad_tkernel::standard::{Handle, real::real_last};
pub(super) fn member_identity(
    member: &Handle<dyn Select3DSensitiveEntity>,
    original: &[Handle<dyn Select3DSensitiveEntity>],
) -> f64 {
    // Native map keys distinguish equal geometry and preserve the first duplicate identity.
    original
        .iter()
        .position(|value| value == member)
        .map_or(0., |index| (index + 1) as f64)
}
pub(super) fn group_snapshot(
    group: &Select3DSensitiveGroup,
    original: &[Handle<dyn Select3DSensitiveEntity>],
    ordinal: i32,
    step: i32,
    failure: bool,
    hit: i32,
    pick: &SelectBasicsPickResult,
) -> Vec<f64> {
    let mut columns = group_outcome_columns(ordinal, step, failure, hit);
    append_quadric_pick(&mut columns, pick);
    append_group_properties(&mut columns, group, original);
    for member in &group.entities() {
        append_group_member(&mut columns, member, original);
    }
    columns
}
fn append_group_properties(
    columns: &mut Vec<f64>,
    group: &Select3DSensitiveGroup,
    original: &[Handle<dyn Select3DSensitiveEntity>],
) {
    columns.extend([
        f64::from(group.size()),
        f64::from(group.nb_sub_elements()),
        f64::from(group.to_build_bvh()),
        f64::from(group.last_detected_entity_index()),
        member_identity(&group.last_detected_entity(), original),
    ]);
    append_group_policies(columns, group);
    append_group_geometry(columns, group);
}
fn append_group_policies(columns: &mut Vec<f64>, group: &Select3DSensitiveGroup) {
    columns.extend([
        f64::from(group.owner_id().get().unwrap().priority()),
        f64::from(group.sensitivity_factor()),
        f64::from(group.must_match_all()),
        f64::from(group.to_check_overlap_all()),
    ]);
}
fn append_group_geometry(columns: &mut Vec<f64>, group: &Select3DSensitiveGroup) {
    let bounds = group.bounding_box();
    let (min, max) = (bounds.corner_min(), bounds.corner_max());
    columns.extend([
        f64::from(bounds.is_valid()),
        min.x,
        min.y,
        min.z,
        max.x,
        max.y,
        max.z,
    ]);
    append_quadric_point(columns, &group.center_of_geometry());
}
fn append_group_member(
    columns: &mut Vec<f64>,
    member: &Handle<dyn Select3DSensitiveEntity>,
    original: &[Handle<dyn Select3DSensitiveEntity>],
) {
    let entity = member.get().unwrap();
    columns.extend([
        member_identity(member, original),
        f64::from(entity.owner_id().get().unwrap().priority()),
        f64::from(entity.sensitivity_factor()),
    ]);
    let sphere = member.clone().cast::<Select3DSensitiveSphere>();
    let point = sphere
        .get()
        .map(Select3DSensitiveSphere::last_detected_point);
    let detected = point.is_some_and(|point| point.x() != real_last());
    columns.push(f64::from(detected));
    append_quadric_point(columns, &point.filter(|_| detected).unwrap_or_default());
}

fn group_outcome_columns(ordinal: i32, step: i32, failure: bool, hit: i32) -> Vec<f64> {
    // Outcomes remain distinct from untouched or partially written pick fields.
    vec![
        f64::from(ordinal),
        f64::from(step),
        f64::from(failure),
        f64::from(hit),
    ]
}

pub(super) fn snapshot_group_match(
    group: &Select3DSensitiveGroup,
    original: &[Handle<dyn Select3DSensitiveEntity>],
    ordinal: i32,
    step: i32,
    volume: &dyn crate::select_basics::SelectBasicsSelectingVolumeManager,
) -> Vec<f64> {
    let mut pick = super::quadric_oracle_snapshot::quadric_seed();
    let outcome = group.matches(volume, &mut pick);
    group_snapshot(
        group,
        original,
        ordinal,
        step,
        outcome.is_err(),
        outcome.map(i32::from).unwrap_or(-1),
        &pick,
    )
}
