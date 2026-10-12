//! Native reference adapters preserve retained slot identity across valid in-place edits.
use super::*;
use rustycad_tkernel::{standard::Handle, tcollection::AsciiString};
use rustycad_tkmath::gp::{Transform3, Vector3};
#[test]
fn generic_action_alias_observes_in_place_replacement_without_ownership() {
    let session = AspectOpenVRSession::default();
    let role = AspectXRTrackedDeviceRole::LeftHand;
    let kind = AspectXRGenericAction::InputTriggerClick;
    let alias = session.generic_action_ref(role, kind).unwrap();
    let duplicate = AspectXrActionRef::generic(&session, role, kind).unwrap();
    assert!(alias.is_null());
    assert!(alias.is_same_reference(&duplicate));
    let action = Handle::new(AspectXRAction::new(
        &AsciiString::from("/alias"),
        AspectXRActionType::InputDigital,
    ));
    assert_action_replacement(&session, &alias, &action, role, kind);
}
fn assert_action_replacement(
    session: &AspectOpenVRSession,
    alias: &AspectXrActionRef,
    action: &Handle<AspectXRAction>,
    role: AspectXRTrackedDeviceRole,
    kind: AspectXRGenericAction,
) {
    session.xr_session_base().fields.write().role_actions[role as usize][kind as i32] =
        action.clone();
    assert_eq!(alias.object_address(), action.object_address());
    let observer = alias.observe();
    assert_eq!(observer.object_address(), action.object_address());
    session.xr_session_base().fields.write().role_actions[role as usize][kind as i32] =
        Handle::default();
    assert!(alias.is_null());
    assert!(!observer.is_null());
    let borrowed = AspectXrActionRef::borrowed(action);
    assert!(!borrowed.is_same_reference(alias));
    assert_eq!(borrowed.object_address(), action.object_address());
}
#[test]
fn tracked_and_head_transform_aliases_observe_in_place_edits() {
    let session = AspectOpenVRSession::default();
    let head = session.head_pose_ref();
    let tracked = AspectXrTransformRef::tracked(&session, 0).unwrap();
    assert!(
        head.is_same_reference(&AspectXrTransformRef::head(&session))
            .unwrap()
    );
    assert!(
        tracked
            .is_same_reference(&AspectXrTransformRef::tracked(&session, 0).unwrap())
            .unwrap()
    );
    assert_moved_transform_fields(&session, &head, &tracked);
}
fn assert_moved_transform_fields(
    session: &AspectOpenVRSession,
    head: &AspectXrTransformRef,
    tracked: &AspectXrTransformRef,
) {
    let mut moved = Transform3::default();
    moved.set_translation(Vector3::new(3., 4., 5.));
    let head_address = head.object_address().unwrap();
    let tracked_address = tracked.object_address().unwrap();
    session.xr_session_base().fields.write().head_pose = moved;
    session.xr_session_base().fields.write().tracked_poses[0].orientation = moved;
    assert_eq!(head.snapshot().unwrap(), moved);
    assert_eq!(tracked.snapshot().unwrap(), moved);
    assert_eq!(head.object_address().unwrap(), head_address);
    assert_eq!(tracked.object_address().unwrap(), tracked_address);
    assert_borrowed_transform(session, head, &moved);
}
fn assert_borrowed_transform(
    session: &AspectOpenVRSession,
    head: &AspectXrTransformRef,
    moved: &Transform3,
) {
    let borrowed = AspectXrTransformRef::borrowed(moved);
    assert_eq!(borrowed.snapshot().unwrap(), *moved);
    assert!(!borrowed.is_same_reference(head).unwrap());
    assert!(AspectXrTransformRef::tracked(session, -1).is_err());
    assert!(AspectXrTransformRef::tracked(session, 1).is_err());
}

#[test]
fn tracked_array_object_alias_observes_signed_bounds_and_resize_without_changing_identity() {
    let session = AspectOpenVRSession::default();
    let reference = session.tracked_poses_ref();
    let duplicate = session.tracked_poses_ref();
    let address = reference.object_address();
    assert!(reference.is_same_reference(&duplicate));
    assert_eq!(
        (reference.lower(), reference.upper(), reference.len()),
        (0, 0, 1)
    );
    resize_reference_poses(&session, -3, 2);
    assert_resized_reference_array(&session, &reference, address);
}
fn resize_reference_poses(session: &AspectOpenVRSession, lower: i32, upper: i32) {
    session
        .xr_session_base()
        .fields
        .write()
        .tracked_poses
        .resize_bounds(lower, upper, false)
        .unwrap();
}
fn assert_resized_reference_array(
    session: &AspectOpenVRSession,
    reference: &AspectXrTrackedPosesRef,
    address: *const rustycad_tkernel::ncollection::Array1<AspectTrackedDevicePose>,
) {
    assert_eq!(reference.object_address(), address);
    assert_eq!(
        (reference.lower(), reference.upper(), reference.len()),
        (-3, 2, 6)
    );
    assert!(reference.pose_snapshot(-3).is_ok());
    assert!(reference.pose_snapshot(-4).is_err());
    assert_eq!(reference.snapshot().lower(), -3);
    assert!(!reference.is_empty());
    let transform = AspectXrTransformRef::tracked(session, -3).unwrap();
    assert!(transform.snapshot().unwrap().is_identity());
    assert_empty_reference_array(session, reference, address);
}
fn assert_empty_reference_array(
    session: &AspectOpenVRSession,
    reference: &AspectXrTrackedPosesRef,
    address: *const rustycad_tkernel::ncollection::Array1<AspectTrackedDevicePose>,
) {
    resize_reference_poses(session, 9, 8);
    assert!(reference.is_empty());
    assert_eq!(reference.object_address(), address);
}
#[test]
fn generic_action_alias_creation_preserves_native_nonzero_bounds_failures() {
    let session = AspectOpenVRSession::default();
    let role = AspectXRTrackedDeviceRole::LeftHand;
    let kind = AspectXRGenericAction::InputTriggerClick;
    session.xr_session_base().fields.write().role_actions[role as usize]
        .resize_bounds(8, 24, false)
        .unwrap();
    let failure = match session.generic_action_ref(role, kind) {
        Err(failure) => failure,
        Ok(_) => panic!("action4 unexpectedly in bounds8..24"),
    };
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(
        session
            .generic_action_ref(role, AspectXRGenericAction::OutputHaptic)
            .unwrap()
            .is_null()
    );
}

fn caller_action_slot(slot: &Handle<AspectXRAction>) -> &Handle<AspectXRAction> {
    // A selected original field is the native handle reference, not a cloned facade.
    slot
}
#[test]
fn borrowed_action_protocol_accepts_actual_live_caller_fields_without_ownership() {
    use rustycad_tkernel::standard::{HandleFieldRef, SharedState};
    let owned = Handle::new(AspectXRAction::new(
        &AsciiString::from("/caller"),
        AspectXRActionType::InputDigital,
    ));
    let fields = SharedState::new(owned.clone());
    let reference = HandleFieldRef::new(&fields, caller_action_slot);
    let action = AspectXrActionRef::borrowed(&reference);
    let address = action.reference_address();
    assert_eq!(action.object_address(), owned.object_address());
    assert_eq!(owned.ref_count(), 2);
    *fields.write() = Handle::default();
    assert!(action.is_null());
    assert_eq!(action.reference_address(), address);
    assert_eq!(owned.ref_count(), 1);
}
