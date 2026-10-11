//! The seven native snapshots prove live handle-slot identity without retaining old targets.
use super::*;
use rustycad_tkernel::standard::Handle;
const NATIVE: &str = include_str!("../../tests/fixtures/view-cube-animation-reference.txt");
#[test]
fn native_retained_animation_aliases_observe_replacement_null_restore_and_protected_assignment() {
    let cube = AisViewCube::new().unwrap();
    let original = cube.view_animation();
    let copied = original.clone();
    let first = Handle::new(AisAnimationCamera::new("first", &Handle::default()));
    let second = Handle::new(AisAnimationCamera::new("second", &Handle::default()));
    let left = cube.view_animation_ref();
    let right = cube.view_animation_ref();
    for (step, row) in NATIVE.lines().enumerate() {
        replace_native_animation_slot(&cube, step, [&original, &first, &second]);
        let actual =
            animation_alias_channels(&cube, left, right, [&original, &first, &second], &copied);
        assert_eq!(
            actual,
            row.split_whitespace()
                .skip(1)
                .map(|v| v.parse::<i32>().unwrap())
                .collect::<Vec<_>>()
        );
    }
}
fn replace_native_animation_slot(
    cube: &AisViewCube,
    step: usize,
    owners: [&Handle<AisAnimationCamera>; 3],
) {
    match step {
        0 => {}
        1 => cube.set_view_animation(owners[1]),
        2 => cube.set_view_animation(&Handle::default()),
        3 => cube.set_view_animation(owners[0]),
        4 => cube.interaction.fields.write().animation = owners[2].clone(),
        5 => cube.interaction.fields.write().animation = Handle::default(),
        6 => cube.interaction.fields.write().animation = owners[1].clone(),
        _ => panic!("animation step={step}; expected native sequence 0..=6"),
    }
}
fn animation_alias_channels(
    cube: &AisViewCube,
    left: AisViewCubeAnimationRef<'_>,
    right: AisViewCubeAnimationRef<'_>,
    owners: [&Handle<AisAnimationCamera>; 3],
    copied: &Handle<AisAnimationCamera>,
) -> Vec<i32> {
    vec![
        left.is_same_reference(&right) as i32,
        left.is_same_reference(&cube.view_animation_ref()) as i32,
        animation_target_identity(left.object_address(), owners),
        animation_target_identity(right.object_address(), owners),
        left.is_null() as i32,
        right.is_null() as i32,
        animation_target_identity(copied.object_address(), owners),
        owners[0].ref_count(),
        owners[1].ref_count(),
        owners[2].ref_count(),
    ]
}
fn animation_target_identity(address: *const (), owners: [&Handle<AisAnimationCamera>; 3]) -> i32 {
    if address.is_null() {
        return 0;
    }
    owners
        .iter()
        .position(|owner| owner.object_address() == address)
        .map_or(-1, |i| i as i32 + 1)
}
#[test]
fn animation_alias_snapshot_is_explicit_and_callbacks_can_replace_the_live_slot() {
    let cube = AisViewCube::new().unwrap();
    let other = AisViewCube::new().unwrap();
    let alias = cube.view_animation_ref();
    assert!(!alias.is_same_reference(&other.view_animation_ref()));
    let original = alias.clone_handle();
    let next = Handle::new(AisAnimationCamera::new("next", &Handle::default()));
    alias.with_animation(|animation| {
        assert!(std::ptr::eq(animation.unwrap(), &*original));
        cube.set_view_animation(&next);
    });
    assert_eq!(alias.object_address(), next.object_address());
    assert_ne!(original.object_address(), alias.object_address());
    cube.set_view_animation(&Handle::default());
    assert!(alias.with_animation(|animation| animation.is_none()));
}
