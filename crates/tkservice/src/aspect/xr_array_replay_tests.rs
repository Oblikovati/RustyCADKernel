//! Full signed-bound native array-object/head snapshots replay without OCCT.
use super::*;
use rustycad_tkmath::gp::Vector3;
const ARRAY_REFERENCE_BOUNDS: [(i32, i32); 4] = [(-3, 2), (5, 5), (9, 8), (10, 12)];
#[test]
fn actual_head_and_array_object_aliases_match_all_native_signed_bounds_rows() {
    let fixture = include_str!("../../tests/fixtures/xr-reference-arrays-native.txt");
    for (scenario, bounds) in ARRAY_REFERENCE_BOUNDS.into_iter().enumerate() {
        replay_native_array_scenario(fixture, scenario as i32, bounds);
    }
    replay_native_action_bounds(fixture);
}
fn replay_native_array_scenario(fixture: &str, scenario: i32, bounds: (i32, i32)) {
    let session = AspectOpenVRSession::default();
    let array = session.tracked_poses_ref();
    let head = session.head_pose_ref();
    for frame in 0..3 {
        if frame > 0 {
            update_native_array_frame(&session, scenario, frame, bounds);
        }
        let row = xr_array_reference_snapshot(&session, &array, &head, scenario, frame);
        assert_eq!(row, native_array_row(fixture, scenario, frame));
    }
}
fn update_native_array_frame(
    session: &AspectOpenVRSession,
    scenario: i32,
    frame: i32,
    bounds: (i32, i32),
) {
    let mut fields = session.xr_session_base().fields.write();
    if frame == 1 {
        fields
            .tracked_poses
            .resize_bounds(bounds.0, bounds.1, false)
            .unwrap();
    }
    update_array_first_pose(&mut fields, scenario, frame, bounds.0);
    fields.head_pose.set_translation(Vector3::new(
        f64::from(11 + scenario + (frame - 1) * 10),
        5.,
        6.,
    ));
}
fn update_array_first_pose(
    fields: &mut AspectXRSessionFields,
    scenario: i32,
    frame: i32,
    lower: i32,
) {
    if !fields.tracked_poses.is_empty() {
        let pose = fields.tracked_poses.change_value(lower).unwrap();
        pose.is_valid_pose = true;
        pose.orientation.set_translation(Vector3::new(
            f64::from(7 + scenario + (frame - 1) * 10),
            2.,
            3.,
        ));
    }
}
fn xr_array_reference_snapshot(
    session: &AspectOpenVRSession,
    array: &AspectXrTrackedPosesRef,
    head: &AspectXrTransformRef,
    scenario: i32,
    frame: i32,
) -> Vec<i32> {
    let pose = first_array_pose_snapshot(array);
    let mut row = vec![scenario, frame];
    row.extend(array_reference_bound_columns(array));
    row.extend([
        i32::from(pose.is_valid_pose),
        pose.orientation.translation_part().x() as i32,
    ]);
    append_array_alias_identity(&mut row, session, array, head);
    row
}
fn array_reference_bound_columns(array: &AspectXrTrackedPosesRef) -> [i32; 4] {
    [
        array.lower(),
        array.upper(),
        array.len() as i32,
        i32::from(array.is_empty()),
    ]
}
fn first_array_pose_snapshot(array: &AspectXrTrackedPosesRef) -> AspectTrackedDevicePose {
    if array.is_empty() {
        return AspectTrackedDevicePose::default();
    }
    array.pose_snapshot(array.lower()).unwrap()
}
fn append_array_alias_identity(
    row: &mut Vec<i32>,
    session: &AspectOpenVRSession,
    array: &AspectXrTrackedPosesRef,
    head: &AspectXrTransformRef,
) {
    row.extend([
        head.snapshot().unwrap().translation_part().x() as i32,
        i32::from(array.is_same_reference(&session.tracked_poses_ref())),
        i32::from(head.is_same_reference(&session.head_pose_ref()).unwrap()),
    ]);
}
fn native_array_row(fixture: &str, scenario: i32, frame: i32) -> Vec<i32> {
    fixture
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|value| value.parse::<i32>().unwrap())
                .collect::<Vec<i32>>()
        })
        .find(|row| row[0] == scenario && row[1] == frame)
        .unwrap()
}
fn replay_native_action_bounds(fixture: &str) {
    let session = AspectOpenVRSession::default();
    let role = AspectXRTrackedDeviceRole::LeftHand;
    session.xr_session_base().fields.write().role_actions[role as usize]
        .resize_bounds(8, 24, false)
        .unwrap();
    let failed = session
        .generic_action_ref(role, AspectXRGenericAction::InputTriggerClick)
        .is_err();
    let valid = session
        .generic_action_ref(role, AspectXRGenericAction::OutputHaptic)
        .unwrap();
    assert_eq!(
        vec![4, 0, i32::from(failed), i32::from(valid.is_null())],
        native_array_row(fixture, 4, 0)
    );
}
