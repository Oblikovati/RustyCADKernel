//! Named keyboard targets expose callback routing and partial failure ordering.
use super::*;
use crate::wnt::WntHidSpaceMouse;
use rustycad_tkernel::standard::Failure;
struct SpaceCallbackRecorder {
    base: AspectWindowInputListenerBase,
    axes: Vec<(AspectVKey, AspectVKey, f64, f64)>,
    buttons: Vec<(bool, AspectVKey, f64)>,
    fail_axis: bool,
    fail_button: bool,
}
impl SpaceCallbackRecorder {
    fn frozen() -> Self {
        let mut base = AspectWindowInputListenerBase::default();
        base.event_timer.stop();
        base.event_timer.reset_to(1.25);
        Self {
            base,
            axes: Vec::new(),
            buttons: Vec::new(),
            fail_axis: false,
            fail_button: false,
        }
    }
}
impl MartensiteSpaceMouseInput for SpaceCallbackRecorder {
    fn space_mouse_state(&self) -> &AspectWindowInputListenerBase {
        &self.base
    }
    fn space_mouse_state_mut(&mut self) -> &mut AspectWindowInputListenerBase {
        &mut self.base
    }
    fn space_mouse_axis(
        &mut self,
        negative: AspectVKey,
        positive: AspectVKey,
        time: f64,
        pressure: f64,
    ) -> Result<(), Failure> {
        self.axes.push((negative, positive, time, pressure));
        if self.fail_axis {
            return Err(Failure::new(&format!(
                "axis {negative}/{positive} failed; expected successful callback"
            )));
        }
        self.base.key_from_axis(negative, positive, time, pressure)
    }
    fn space_mouse_press(&mut self, key: AspectVKey, time: f64) -> Result<(), Failure> {
        self.record_button(true, key, time)?;
        self.base.key_down(key, time, 1.0)
    }
    fn space_mouse_release(&mut self, key: AspectVKey, time: f64) -> Result<(), Failure> {
        self.record_button(false, key, time)?;
        self.base.key_up(key, time)
    }
}
impl SpaceCallbackRecorder {
    fn record_button(&mut self, pressed: bool, key: AspectVKey, time: f64) -> Result<(), Failure> {
        self.buttons.push((pressed, key, time));
        if self.fail_button {
            return Err(Failure::new(&format!(
                "key {key} failed; expected successful callback"
            )));
        }
        Ok(())
    }
}
#[test]
fn translation_bypasses_callbacks_and_uses_direct_cache() {
    let mut target = SpaceCallbackRecorder::frozen();
    target.fail_axis = true;
    let report = WntHidSpaceMouse::new(0xC62B, &[1, 94, 1, 0, 0, 0, 0]);
    assert!(dispatch_space_mouse_translation(&mut target, &report).unwrap());
    assert!(target.axes.is_empty());
    assert!(
        target
            .base
            .keys
            .is_key_down(AspectVKeyBasic::NavSlideRight as AspectVKey)
            .unwrap()
    );
    assert!(!dispatch_space_mouse_rotation(&mut target, &report).unwrap());
    assert!(!dispatch_space_mouse_keys(&mut target, &report).unwrap());
}
#[test]
fn rotation_error_stops_before_later_axis_callbacks() {
    let mut target = SpaceCallbackRecorder::frozen();
    target.fail_axis = true;
    let report = WntHidSpaceMouse::new(0xC62B, &[2, 94, 1, 94, 1, 94, 1]);
    assert!(dispatch_space_mouse_rotation(&mut target, &report).is_err());
    assert_eq!(target.axes.len(), 1);
    assert_eq!(target.axes[0].2, 1.25);
    assert!(!dispatch_space_mouse_translation(&mut target, &report).unwrap());
}
#[test]
fn skipped_rotation_axes_do_not_dispatch_or_request_updates() {
    use rustycad_tkernel::ncollection::Vec3;
    let mut target = SpaceCallbackRecorder::frozen();
    target.base.mouse3d_no_rotate = Vec3::new(true, true, true);
    let report = WntHidSpaceMouse::new(0xC62B, &[2, 94, 1, 94, 1, 94, 1]);
    assert!(!dispatch_space_mouse_rotation(&mut target, &report).unwrap());
    assert!(target.axes.is_empty());
}
#[test]
fn failed_key_press_caches_only_bits_processed_before_exception() {
    let mut target = SpaceCallbackRecorder::frozen();
    target.fail_button = true;
    let report = WntHidSpaceMouse::new(0xC62B, &[3, 6, 0, 0, 0]);
    assert!(dispatch_space_mouse_keys(&mut target, &report).is_err());
    assert!(target.base.mouse3d_button_state[1]);
    assert!(!target.base.mouse3d_button_state[2]);
    assert_eq!(
        target.buttons,
        [(true, AspectVKeyBasic::ViewFitAll as AspectVKey, 1.25)]
    );
}
#[test]
fn held_buttons_repeat_callbacks_and_releases_are_dispatched_once() {
    let mut target = SpaceCallbackRecorder::frozen();
    let press = WntHidSpaceMouse::new(0xC62B, &[3, 2, 0, 0, 0]);
    let release = WntHidSpaceMouse::new(0xC62B, &[3, 0, 0, 0, 0]);
    assert!(dispatch_space_mouse_keys(&mut target, &press).unwrap());
    assert!(dispatch_space_mouse_keys(&mut target, &press).unwrap());
    assert!(dispatch_space_mouse_keys(&mut target, &release).unwrap());
    assert!(!dispatch_space_mouse_keys(&mut target, &release).unwrap());
    assert_eq!(target.buttons.len(), 3);
    assert_eq!(
        target.buttons.iter().map(|b| b.0).collect::<Vec<_>>(),
        [true, true, false]
    );
}
#[test]
fn failed_key_release_retains_held_key_but_clears_native_button_cache() {
    let mut target = SpaceCallbackRecorder::frozen();
    let press = WntHidSpaceMouse::new(0xC62B, &[3, 2, 0, 0, 0]);
    let release = WntHidSpaceMouse::new(0xC62B, &[3, 0, 0, 0, 0]);
    dispatch_space_mouse_keys(&mut target, &press).unwrap();
    target.fail_button = true;
    assert!(dispatch_space_mouse_keys(&mut target, &release).is_err());
    assert!(!target.base.mouse3d_button_state[1]);
    assert!(
        target
            .base
            .keys
            .is_key_down(AspectVKeyBasic::ViewFitAll as AspectVKey)
            .unwrap()
    );
}
