//! Shared OCCT 3d097a0 HID routing; translation intentionally bypasses virtual callbacks.
use super::{AspectVKey, AspectVKeyBasic as V, MartensiteSpaceMouseInput};
use crate::wnt::WntHidSpaceMouse;
use rustycad_tkernel::standard::Failure;
type SpaceRotationAxis = (bool, bool, f64, V, V);
/// Update translation keys directly; `dispatch_space_mouse_translation(&mut target, &report)?`.
pub fn dispatch_space_mouse_translation<T: MartensiteSpaceMouseInput + ?Sized>(
    target: &mut T,
    event: &WntHidSpaceMouse<'_>,
) -> Result<bool, Failure> {
    if !event.is_translation() {
        return Ok(false);
    }
    let base = target.space_mouse_state();
    let time = base.event_timer.elapsed_time();
    let (translation, _) = event.translation(base.mouse3d_is_quadric);
    let translation = translation * f64::from(base.mouse3d_accel_trans);
    apply_space_translation(base, time, translation)?;
    Ok(true)
}
fn apply_space_translation(
    base: &super::AspectWindowInputListenerBase,
    time: f64,
    translation: rustycad_tkernel::ncollection::Vec3<f64>,
) -> Result<(), Failure> {
    for (negative, positive, pressure) in [
        (V::NavSlideLeft, V::NavSlideRight, translation.x),
        (V::NavForward, V::NavBackward, translation.y),
        (V::NavSlideUp, V::NavSlideDown, translation.z),
    ] {
        base.keys.key_from_axis(
            negative as AspectVKey,
            positive as AspectVKey,
            time,
            pressure,
        )?;
    }
    Ok(())
}

/// Route allowed rotation axes through callbacks; `dispatch_space_mouse_rotation(&mut target, &report)?`.
pub fn dispatch_space_mouse_rotation<T: MartensiteSpaceMouseInput + ?Sized>(
    target: &mut T,
    event: &WntHidSpaceMouse<'_>,
) -> Result<bool, Failure> {
    if !event.is_rotation() {
        return Ok(false);
    }
    let time = target.space_mouse_state().event_timer.elapsed_time();
    let axes = space_rotation_axes(target, event);
    let mut changed = false;
    for axis in axes {
        changed = dispatch_space_rotation_axis(target, axis, time)? || changed;
    }
    Ok(changed)
}
fn dispatch_space_rotation_axis<T: MartensiteSpaceMouseInput + ?Sized>(
    target: &mut T,
    axis: SpaceRotationAxis,
    time: f64,
) -> Result<bool, Failure> {
    let (skip, reverse, pressure, negative, positive) = axis;
    if skip {
        return Ok(false);
    }
    target.space_mouse_axis(
        negative as AspectVKey,
        positive as AspectVKey,
        time,
        if reverse { -pressure } else { pressure },
    )?;
    Ok(true)
}
fn space_rotation_axes<T: MartensiteSpaceMouseInput + ?Sized>(
    target: &T,
    event: &WntHidSpaceMouse<'_>,
) -> [SpaceRotationAxis; 3] {
    let base = target.space_mouse_state();
    let (rotation, _) = event.rotation(base.mouse3d_is_quadric);
    let rotation = rotation * f64::from(base.mouse3d_accel_rotate);
    let (skip, reverse) = (base.mouse3d_no_rotate, base.mouse3d_to_reverse);
    [
        (skip.x, reverse.x, rotation.x, V::NavLookUp, V::NavLookDown),
        (skip.y, reverse.y, rotation.y, V::NavRollCW, V::NavRollCCW),
        (
            skip.z,
            reverse.z,
            rotation.z,
            V::NavLookLeft,
            V::NavLookRight,
        ),
    ]
}
/// Cache button bits before callbacks, including unmapped bits; `dispatch_space_mouse_keys(&mut target, &report)?`.
pub fn dispatch_space_mouse_keys<T: MartensiteSpaceMouseInput + ?Sized>(
    target: &mut T,
    event: &WntHidSpaceMouse<'_>,
) -> Result<bool, Failure> {
    if !event.is_key_state() {
        return Ok(false);
    }
    let time = target.space_mouse_state().event_timer.elapsed_time();
    let key_state = event.key_state();
    let mut changed = false;
    for bit in 0..32u16 {
        changed = dispatch_space_button(target, event, bit, key_state, time)? || changed;
    }
    Ok(changed)
}
fn dispatch_space_button<T: MartensiteSpaceMouseInput + ?Sized>(
    target: &mut T,
    event: &WntHidSpaceMouse<'_>,
    bit: u16,
    key_state: u32,
    time: f64,
) -> Result<bool, Failure> {
    let Some((pressed, key)) = cache_space_button(target, event, bit, key_state) else {
        return Ok(false);
    };
    if pressed {
        target.space_mouse_press(key, time)?;
    } else {
        target.space_mouse_release(key, time)?;
    }
    Ok(true)
}

fn cache_space_button<T: MartensiteSpaceMouseInput + ?Sized>(
    target: &mut T,
    event: &WntHidSpaceMouse<'_>,
    bit: u16,
    key_state: u32,
) -> Option<(bool, AspectVKey)> {
    let pressed = key_state & (1 << bit) != 0;
    let state = &mut target.space_mouse_state_mut().mouse3d_button_state[usize::from(bit)];
    let released = *state && !pressed;
    *state = pressed;
    let key = event.hid_to_space_key(bit);
    if (!released && !pressed) || key == V::Unknown as AspectVKey {
        return None;
    }
    Some((pressed, key))
}
