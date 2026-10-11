//! Gesture identities and walking predicates replay pinned native behavior.
#[path = "support/view_navigation_fixture.rs"]
mod fixture;
use fixture::*;
use rustycad_tkv3d::ais::*;
#[test]
fn walk_component_defaults_and_empty_boundary_match_native() {
    assert_native(&part_values(&AisWalkPart::new()), "part-default");
    assert_native(&part_values(&AisWalkPart::default()), "part-default");
    for (index, value) in boundary_values().into_iter().enumerate() {
        let part = AisWalkPart {
            value,
            pressure: 0.25,
            duration: 7.,
        };
        assert_native(&part_values(&part), &format!("part-{index}"));
    }
}
fn boundary_values() -> Vec<f64> {
    let small = f64::MIN_POSITIVE;
    let paired = [
        f64::from_bits(small.to_bits() - 1),
        small,
        f64::from_bits(small.to_bits() + 1),
        1e-300,
        f64::INFINITY,
    ];
    [0., -0., f64::from_bits(1), -f64::from_bits(1)]
        .into_iter()
        .chain(paired.into_iter().flat_map(|value| [value, -value]))
        .chain([f64::NAN])
        .collect()
}
#[test]
fn held_keys_and_stance_flags_are_independent_from_motion() {
    assert_native(&delta_values(&AisWalkDelta::default()), "flags-0");
    for mask in 0..16 {
        let mut delta = AisWalkDelta::new();
        delta.set_defined(mask & 1 != 0);
        delta.set_jumping(mask & 2 != 0);
        delta.set_crouching(mask & 4 != 0);
        delta.set_running(mask & 8 != 0);
        assert_native(&delta_values(&delta), &format!("flags-{mask}"));
        delta.set_defined(false);
        delta.set_jumping(false);
        delta.set_crouching(false);
        delta.set_running(false);
        assert_native(&delta_values(&delta), "flags-0");
    }
}
fn modify_axis(delta: &mut AisWalkDelta, index: usize, value: f64) {
    let part = if index < 3 {
        delta.translation_mut(TRANSLATION_AXES[index])
    } else {
        delta.rotation_mut(ROTATION_AXES[index - 3])
    };
    part.value = value;
    part.pressure = 0.25 + index as f64;
    part.duration = 0.5 + index as f64;
}
#[test]
fn every_axis_has_mutable_and_shared_access_with_independent_copies() {
    for index in 0..6 {
        check_axis_snapshot(index);
    }
}
fn check_axis_snapshot(index: usize) {
    let mut delta = AisWalkDelta::new();
    let value = 0.125 * (index + 1) as f64 * if index < 3 { 1. } else { -1. };
    modify_axis(&mut delta, index, value);
    assert_native(&delta_values(&delta), &format!("axis-{index}"));
    let copy = delta;
    if index < 3 {
        delta[TRANSLATION_AXES[index]].value = 0.;
    } else {
        delta[ROTATION_AXES[index - 3]].value = 0.;
    }
    assert_native(&delta_values(&delta), &format!("axis-cleared-{index}"));
    assert_native(&delta_values(&copy), &format!("axis-copy-{index}"));
}
#[test]
fn explicit_shared_accessors_match_indexed_components() {
    let mut delta = AisWalkDelta::new();
    for (index, axis) in TRANSLATION_AXES.into_iter().enumerate() {
        delta[axis].value = index as f64;
        assert_eq!(delta.translation(axis), &delta[axis]);
    }
    for (index, axis) in ROTATION_AXES.into_iter().enumerate() {
        delta[axis].value = -(index as f64);
        assert_eq!(delta.rotation(axis), &delta[axis]);
    }
}
#[test]
fn nonfinite_motion_preserves_native_nonempty_queries() {
    for (index, value) in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY]
        .into_iter()
        .enumerate()
    {
        let mut delta = AisWalkDelta::new();
        delta[AisWalkTranslation::Forward].value = value;
        assert_native(&delta_values(&delta), &format!("nonfinite-{index}"));
    }
}
#[test]
fn navigation_policy_and_bounds_match_native() {
    let navigation = [
        AisNavigationMode::Orbit as i32,
        AisNavigationMode::FirstPersonFlight as i32,
        AisNavigationMode::FirstPersonWalk as i32,
        AisNavigationModeBounds::Lower as i32,
        AisNavigationModeBounds::Upper as i32,
    ];
    assert_native(&navigation.map(f64::from), "navigation");
}
#[test]
fn rotation_policy_and_bounds_match_native() {
    let rotation = [
        AisRotationMode::BndBoxActive as i32,
        AisRotationMode::PickLast as i32,
        AisRotationMode::PickCenter as i32,
        AisRotationMode::CameraAt as i32,
        AisRotationMode::BndBoxScene as i32,
        AisRotationModeBounds::Lower as i32,
        AisRotationModeBounds::Upper as i32,
    ];
    assert_native(&rotation.map(f64::from), "rotation");
}
#[test]
fn mouse_gesture_values_match_native() {
    let mouse = [
        AisMouseGesture::None,
        AisMouseGesture::SelectRectangle,
        AisMouseGesture::SelectLasso,
        AisMouseGesture::Zoom,
        AisMouseGesture::ZoomVertical,
        AisMouseGesture::ZoomWindow,
        AisMouseGesture::Pan,
        AisMouseGesture::RotateOrbit,
        AisMouseGesture::RotateView,
        AisMouseGesture::Drag,
    ];
    assert_native(&mouse.map(|gesture| f64::from(gesture as i32)), "mouse");
}
#[test]
fn walk_axis_values_match_native() {
    let axes = [
        TRANSLATION_AXES[0] as i32,
        TRANSLATION_AXES[1] as i32,
        TRANSLATION_AXES[2] as i32,
        ROTATION_AXES[0] as i32,
        ROTATION_AXES[1] as i32,
        ROTATION_AXES[2] as i32,
    ];
    assert_native(&axes.map(f64::from), "axes");
}
