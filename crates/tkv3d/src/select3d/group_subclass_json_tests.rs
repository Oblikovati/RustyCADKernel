//! Native qualified Group diagnostics and distance retain application geometry overrides.
use super::group_hook_fake::GroupHookFake;
use super::group_json_tests::normalize_group_json_pointers;
use super::group_oracle_geometry::group_children;
use super::quadric_oracle_volume::quadric_volume;
use super::triangulation_oracle_snapshot::compare_oracle_snapshot;
use super::{Select3DSensitiveEntity, Select3DSensitiveGroup};
use crate::select_mgr::SelectMgrEntityOwner;
use rustycad_tkernel::standard::dump::DumpStream;
fn geometry_source(geometry: i32) -> GroupHookFake {
    let owner = SelectMgrEntityOwner::new(13).cast();
    let children = group_children(2, &owner);
    let core = Select3DSensitiveGroup::new(&owner, true);
    core.add_all(&children);
    let mut source = GroupHookFake::new(core, 0);
    source.geometry = geometry;
    source
}
fn replay_subclass_json(geometry: i32, depth: i32, expected: &mut std::str::Lines<'_>) {
    let source = geometry_source(geometry);
    let mut output = DumpStream::default();
    source.dump_json(&mut output, depth).unwrap();
    let actual = format!(
        "{geometry} {depth} {}",
        normalize_group_json_pointers(output.text())
    );
    assert_eq!(actual, expected.next().unwrap());
    let volume = quadric_volume(0, false, false, false);
    let distance = super::select3d_group_distance_to_cog(&source, &volume).unwrap();
    compare_oracle_snapshot(
        &[f64::from(geometry), f64::from(depth), distance],
        expected.next().unwrap(),
    );
}
#[test]
fn native_group_application_geometry_reaches_base_diagnostics_and_qualified_distance() {
    let mut expected =
        include_str!("../../tests/fixtures/sensitive-group-subclass-json.txt").lines();
    for geometry in 0..4 {
        for depth in [-1, 0, 1, 2, 3] {
            replay_subclass_json(geometry, depth, &mut expected);
        }
    }
    assert!(expected.next().is_none());
}
