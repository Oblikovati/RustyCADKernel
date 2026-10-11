//! Connected-copy defaults, owner propagation and independent BVH lifecycle.
use super::primitive_array_matching_tests::{
    MatchScenario, compare_matching_snapshot, matching_probe_owned,
};
use super::{
    Select3DSensitiveEntity, Select3DSensitiveEntityMembers, Select3DSensitivePrimitiveArray,
};
use crate::select_mgr::{SelectMgrEntityOwner, SelectMgrEntityOwnerMembers};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;
type DetectionMapSnapshot = (
    Handle<HPackedMapOfInteger>,
    Handle<HPackedMapOfInteger>,
    Vec<f64>,
);
use rustycad_tkservice::graphic3d::Graphic3dBufferMembers;

fn lifecycle_probe(ordinal: i32) -> Select3DSensitivePrimitiveArray {
    let matching = (ordinal / 8) * 512
        + (ordinal / 4 % 2) * 256
        + (ordinal / 2 % 2) * 128
        + (ordinal % 2) * 64
        + 56
        + 4
        + 1024;
    matching_probe_owned(
        MatchScenario::new(matching),
        &SelectMgrEntityOwner::new(13).cast(),
        7,
    )
}
fn lifecycle_snapshot(
    probe: &Select3DSensitivePrimitiveArray,
    scenario: i32,
    step: i32,
) -> Vec<f64> {
    let mut snapshot = vec![
        f64::from(scenario),
        f64::from(step),
        f64::from(probe.to_build_bvh()),
        f64::from(probe.bvh_indices.nb_elements()),
        f64::from(probe.nb_sub_elements()),
        f64::from(probe.patch_size_max()),
        f64::from(probe.patch_distance()),
        f64::from(probe.sensitivity_factor()),
    ];
    snapshot.extend(lifecycle_detection_snapshot(probe));
    snapshot.extend(lifecycle_owner_snapshot(probe));
    snapshot
}
fn lifecycle_owner_snapshot(probe: &Select3DSensitivePrimitiveArray) -> [f64; 2] {
    let owner = probe.owner_id();
    [
        f64::from(owner.is_null()),
        f64::from(owner.get().map_or(0, |value| value.priority())),
    ]
}
fn lifecycle_detection_snapshot(probe: &Select3DSensitivePrimitiveArray) -> [f64; 5] {
    [
        f64::from(probe.to_detect_elements()),
        f64::from(probe.to_detect_nodes()),
        f64::from(probe.to_detect_edges()),
        f64::from(probe.to_detect_element_map()),
        f64::from(probe.to_detect_node_map()),
    ]
}

fn connected_probe(
    probe: &Select3DSensitivePrimitiveArray,
) -> Handle<Select3DSensitivePrimitiveArray> {
    let connected = probe
        .get_connected()
        .unwrap()
        .cast::<Select3DSensitivePrimitiveArray>();
    assert_eq!(probe.read().verts, connected.read().verts);
    assert_eq!(probe.read().indices, connected.read().indices);
    assert_eq!(probe.owner_id(), connected.owner_id());
    connected
}
fn assert_subgroup_ownership(probe: &Select3DSensitivePrimitiveArray) {
    for group in probe.read().groups.unwrap_or_default() {
        assert_eq!(group.owner_id(), probe.owner_id());
    }
}
fn compare_lifecycle_pair(
    original: &Select3DSensitivePrimitiveArray,
    connected: &Select3DSensitivePrimitiveArray,
    ordinal: i32,
    step: i32,
    native: &mut std::str::Lines<'_>,
) {
    compare_matching_snapshot(
        &lifecycle_snapshot(original, ordinal, step),
        native.next().unwrap(),
    );
    compare_matching_snapshot(
        &lifecycle_snapshot(connected, ordinal, step + 1),
        native.next().unwrap(),
    );
}
#[test]
fn native_connected_copy_owner_propagation_and_bvh_dirty_lifecycle() {
    let mut native =
        include_str!("../../tests/fixtures/sensitive-primitive-lifecycle-native.txt").lines();
    for ordinal in 0..16 {
        let original = lifecycle_probe(ordinal);
        let connected = connected_probe(&original);
        compare_lifecycle_pair(&original, &connected, ordinal, 0, &mut native);
        original.set(&SelectMgrEntityOwner::new(29).cast());
        assert_subgroup_ownership(&original);
        assert_subgroup_ownership(&connected);
        compare_lifecycle_pair(&original, &connected, ordinal, 2, &mut native);
        original.bvh().unwrap();
        connected.bvh().unwrap();
        compare_lifecycle_pair(&original, &connected, ordinal, 4, &mut native);
        original.set.mark_dirty();
        compare_lifecycle_pair(&original, &connected, ordinal, 6, &mut native);
    }
    assert!(native.next().is_some());
    assert!(native.next().is_none());
}

#[test]
fn native_detection_map_reset_preserves_identity_until_disabled() {
    let original = matching_probe_owned(MatchScenario::new(1036), &Handle::default(), 2);
    let snapshot = detection_map_control_snapshot(&original);
    compare_matching_snapshot(
        &snapshot,
        include_str!("../../tests/fixtures/sensitive-primitive-lifecycle-native.txt")
            .lines()
            .nth(128)
            .unwrap(),
    );
}

fn detection_map_control_snapshot(original: &Select3DSensitivePrimitiveArray) -> Vec<f64> {
    let (elements, nodes, mut snapshot) = matched_detection_maps(original);
    reset_retained_detection_maps(original, &elements, &nodes, &mut snapshot);
    replace_detection_maps(original, &elements, &nodes, &mut snapshot);
    snapshot
}
fn matched_detection_maps(original: &Select3DSensitivePrimitiveArray) -> DetectionMapSnapshot {
    let mut pick = crate::select_basics::SelectBasicsPickResult::default();
    let volume = super::primitive_array_matching_tests::matching_volume(1, false);
    let hit = original.matches(&volume, &mut pick).unwrap();
    let (elements, nodes) = (
        original.last_detected_element_map(),
        original.last_detected_node_map(),
    );
    let snapshot = vec![
        16.0,
        0.0,
        f64::from(hit),
        f64::from(elements.map().extent()),
        f64::from(nodes.map().extent()),
    ];
    (elements, nodes, snapshot)
}
fn reset_retained_detection_maps(
    original: &Select3DSensitivePrimitiveArray,
    elements: &Handle<HPackedMapOfInteger>,
    nodes: &Handle<HPackedMapOfInteger>,
    snapshot: &mut Vec<f64>,
) {
    original.set_detect_element_map(true);
    original.set_detect_node_map(true);
    snapshot.extend([
        f64::from(*elements == original.last_detected_element_map()),
        f64::from(*nodes == original.last_detected_node_map()),
        f64::from(elements.map().is_empty()),
        f64::from(nodes.map().is_empty()),
    ]);
}
fn replace_detection_maps(
    original: &Select3DSensitivePrimitiveArray,
    elements: &Handle<HPackedMapOfInteger>,
    nodes: &Handle<HPackedMapOfInteger>,
    snapshot: &mut Vec<f64>,
) {
    original.set_detect_element_map(false);
    original.set_detect_node_map(false);
    snapshot.extend([
        f64::from(original.last_detected_element_map().is_null()),
        f64::from(original.last_detected_node_map().is_null()),
    ]);
    original.set_detect_element_map(true);
    original.set_detect_node_map(true);
    snapshot.extend([
        f64::from(*elements != original.last_detected_element_map()),
        f64::from(*nodes != original.last_detected_node_map()),
    ]);
}
