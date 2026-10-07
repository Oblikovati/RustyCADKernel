//! Aspect XR actions and the OpenVR session of a build without OpenVR.
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::aspect::{
    AspectEye, AspectOpenVRSession, AspectXRAction, AspectXRActionSet, AspectXRActionType,
    AspectXRAnalogActionData, AspectXRGenericAction, AspectXRHapticActionData, AspectXRSession,
    AspectXRSessionMembers, AspectXRSessionTrackingUniverseOrigin, AspectXRTrackedDeviceRole,
};

fn action(id: &str, kind: AspectXRActionType) -> Handle<AspectXRAction> {
    Handle::new(AspectXRAction::new(&AsciiString::from(id), kind))
}

#[test]
fn actions_and_sets() {
    let menu = action("/menu", AspectXRActionType::InputDigital);
    assert!(!menu.is_valid());
    menu.set_raw_handle(5);
    assert!(menu.is_valid() && menu.raw_handle() == 5);
    let set = AspectXRActionSet::new(&AsciiString::from("/main"));
    set.add_action(menu.clone());
    set.add_action(action("/menu", AspectXRActionType::InputPose));
    assert_eq!(set.actions().len(), 1);
    assert_eq!(set.actions()[0].1, menu);
    assert!(!AspectXRAnalogActionData::default().is_changed());
    let haptic = AspectXRHapticActionData {
        delay: 0.0,
        duration: 1.0,
        frequency: 2.0,
        amplitude: 0.5,
    };
    assert!(haptic.is_valid() && !AspectXRHapticActionData::default().is_valid());
}

#[test]
fn openvr_session_without_openvr() {
    let session = AspectOpenVRSession::default();
    assert!(!AspectOpenVRSession::is_hmd_present());
    assert!(!session.open() && !session.is_open() && !session.wait_poses());
    assert_eq!(
        session.named_tracked_device(AspectXRTrackedDeviceRole::Head),
        -1
    );
    assert!(session.head_to_eye_transform(AspectEye::Left).is_identity());
    assert!(session.left_hand_pose().is_ok());
    assert!(session.has_tracked_pose(0).is_ok() && session.has_tracked_pose(1).is_err());
    session.set_tracking_origin(AspectXRSessionTrackingUniverseOrigin::Seated);
    assert_eq!(
        session.tracking_origin(),
        AspectXRSessionTrackingUniverseOrigin::Seated
    );
    assert!(
        session
            .generic_action(
                AspectXRTrackedDeviceRole::LeftHand,
                AspectXRGenericAction::OutputHaptic
            )
            .is_null()
    );
    let digital = action("/d", AspectXRActionType::InputDigital);
    assert!(session.get_digital_action_data(&digital).is_ok());
    let error = session.get_analog_action_data(&digital).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    let haptic = action("/h", AspectXRActionType::OutputHaptic);
    assert!(
        session
            .trigger_haptic_vibration_action(&haptic, &AspectXRHapticActionData::default())
            .is_err()
    );
    assert!(session.abort_haptic_vibration_action(&haptic).is_ok());
    let mut texture = Handle::default();
    assert!(session.load_render_model(0, &mut texture).is_null());
    assert_eq!(session.unit_factor(), 1.0);
    assert_eq!(session.field_of_view(), 90.0);
}
