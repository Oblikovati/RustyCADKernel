//! OCCT 3d097a0 adds only forward/backward; other opposing axes overwrite in native order.
use super::{AisWalkDelta, AisWalkPart, AisWalkRotation as R, AisWalkTranslation as T};
use rustycad_tkernel::standard::Failure;
use rustycad_tkservice::aspect::{AspectVKey, AspectVKeyBasic as V, AspectVKeySet};
pub(super) fn horizontal_keys(
    keys: &AspectVKeySet,
    walk: &mut AisWalkDelta,
    now: f64,
    interval: f64,
    ratio: f64,
) -> Result<(), Failure> {
    for (key, axis, sign) in [
        (V::NavForward, T::Forward, 1.0),
        (V::NavBackward, T::Forward, -1.0),
        (V::NavSlideLeft, T::Side, -1.0),
        (V::NavSlideRight, T::Side, 1.0),
    ] {
        translation_key(keys, walk, key, axis, now, interval, sign * ratio)?;
    }
    Ok(())
}
pub(super) fn vertical_keys(
    keys: &AspectVKeySet,
    walk: &mut AisWalkDelta,
    now: f64,
    interval: f64,
) -> Result<(), Failure> {
    translation_key(keys, walk, V::NavSlideUp, T::Up, now, interval, 1.0)?;
    translation_key(keys, walk, V::NavSlideDown, T::Up, now, interval, -1.0)
}
pub(super) fn rotation_keys(
    keys: &AspectVKeySet,
    walk: &mut AisWalkDelta,
    now: f64,
    interval: f64,
    invert_pitch: bool,
) -> Result<(), Failure> {
    let pitch = if invert_pitch { 1.0 } else { -1.0 };
    for (key, axis, sign) in rotation_bindings(pitch) {
        if let Some(mut part) = navigation_component(keys, key, now, interval)? {
            part.value *= part.pressure * sign;
            walk[axis] = part;
            walk.set_defined(true);
        }
    }
    Ok(())
}
fn translation_key(
    keys: &AspectVKeySet,
    walk: &mut AisWalkDelta,
    key: V,
    axis: T,
    now: f64,
    interval: f64,
    factor: f64,
) -> Result<(), Failure> {
    if let Some(mut part) = navigation_component(keys, key, now, interval)? {
        part.value *= factor;
        if axis == T::Forward {
            part.value += walk[axis].value;
        }
        walk[axis] = part;
        walk.set_defined(true);
    }
    Ok(())
}
fn navigation_component(
    keys: &AspectVKeySet,
    key: V,
    now: f64,
    interval: f64,
) -> Result<Option<AisWalkPart>, Failure> {
    let (held, duration, pressure) = keys.hold_duration_pressure(key as AspectVKey, now, 1.0)?;
    if !held {
        return Ok(None);
    }
    let value = bounded_navigation_progress(interval, duration);
    Ok(Some(AisWalkPart {
        value,
        pressure,
        duration,
    }))
}
fn bounded_navigation_progress(interval: f64, duration: f64) -> f64 {
    // std::min(interval, duration) keeps its first operand for NaN, unlike f64::min.
    let bounded = if duration < interval {
        duration
    } else {
        interval
    };
    bounded.abs()
}

fn rotation_bindings(pitch: f64) -> [(V, R, f64); 6] {
    [
        (V::NavLookLeft, R::Yaw, 1.0),
        (V::NavLookRight, R::Yaw, -1.0),
        (V::NavLookUp, R::Pitch, pitch),
        (V::NavLookDown, R::Pitch, -pitch),
        (V::NavRollCCW, R::Roll, -1.0),
        (V::NavRollCW, R::Roll, 1.0),
    ]
}
