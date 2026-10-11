//! Frozen named clocks and native keyboard snapshots for controller regressions.
use super::*;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::osd::{CpuTimes, Timer, WallClock};
use rustycad_tkservice::aspect::{AspectScrollDelta, AspectVKey, AspectVKeyBasic as V};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
pub(super) const NAVIGATION_KEYS: [V; 12] = [
    V::NavForward,
    V::NavBackward,
    V::NavSlideLeft,
    V::NavSlideRight,
    V::NavLookLeft,
    V::NavLookRight,
    V::NavLookUp,
    V::NavLookDown,
    V::NavRollCCW,
    V::NavRollCW,
    V::NavSlideUp,
    V::NavSlideDown,
];
pub(super) struct FrozenNavigationClock(AtomicU64);
impl FrozenNavigationClock {
    pub(super) fn set(&self, seconds: f64) {
        self.0.store(seconds.to_bits(), Ordering::Relaxed);
    }
}
impl WallClock for FrozenNavigationClock {
    fn now(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::Relaxed))
    }
}
struct FrozenNavigationCpu;
impl CpuTimes for FrozenNavigationCpu {
    fn process_cpu(&self) -> (f64, f64) {
        (0.0, 0.0)
    }
    fn thread_cpu(&self) -> (f64, f64) {
        (0.0, 0.0)
    }
}
pub(super) fn prepare(
    previous: f64,
    current: f64,
) -> (AisViewController, Arc<FrozenNavigationClock>) {
    let clock = Arc::new(FrozenNavigationClock(AtomicU64::new(0.0_f64.to_bits())));
    let timer = Timer::with_cpu(false, Arc::new(FrozenNavigationCpu), clock.clone());
    let mut c = AisViewController::with_event_timer(timer);
    c.navigation_state.last_events_time = previous;
    clock.set(current);
    (c, clock)
}
pub(super) fn frame(
    c: &mut AisViewController,
    clock: &FrozenNavigationClock,
    previous: f64,
    current: f64,
) {
    c.navigation_state.last_events_time = previous;
    clock.set(current);
}
pub(super) fn assert_native(actual: &[f64], name: &str) {
    let row = include_str!("../../tests/fixtures/view-controller-keys-native.txt")
        .lines()
        .find(|row| row.split_whitespace().next() == Some(name))
        .expect(name);
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|v| v.parse().unwrap())
        .collect();
    assert_eq!(actual.len(), expected.len(), "native {name} shape");
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_native_number(*actual, expected, name, index);
    }
}
fn assert_native_number(actual: f64, expected: f64, name: &str, index: usize) {
    if actual.is_nan() && expected.is_nan() {
        return;
    }
    assert_eq!(actual, expected, "native {name}, column {index}");
    if expected == 0.0 {
        assert_eq!(
            actual.is_sign_negative(),
            expected.is_sign_negative(),
            "native {name}, column {index} zero sign"
        );
    }
}
pub(super) fn fetch(c: &mut AisViewController, name: &str, crouch: f64, run: f64) {
    let w = c.fetch_navigation_keys(crouch, run).unwrap();
    let mut values = component_values(&w);
    values.extend(walk_queries(&w));
    values.extend([
        c.to_ask_next_frame() as u8 as f64,
        c.navigation_state.has_thrust as u8 as f64,
        c.navigation_state.last_events_time,
        c.event_time(),
        c.keys().modifiers() as f64,
    ]);
    values.extend(
        NAVIGATION_KEYS
            .into_iter()
            .chain([V::Shift, V::NavJump, V::NavCrouch])
            .flat_map(|key| key_values(c, key)),
    );
    assert_native(&values, name);
}
fn component_values(w: &AisWalkDelta) -> Vec<f64> {
    [
        AisWalkTranslation::Forward,
        AisWalkTranslation::Side,
        AisWalkTranslation::Up,
    ]
    .into_iter()
    .map(|axis| w[axis])
    .chain(
        [
            AisWalkRotation::Yaw,
            AisWalkRotation::Pitch,
            AisWalkRotation::Roll,
        ]
        .into_iter()
        .map(|axis| w[axis]),
    )
    .flat_map(|p| [p.value, p.pressure, p.duration])
    .collect()
}
fn walk_queries(w: &AisWalkDelta) -> [f64; 7] {
    [
        w.is_empty(),
        w.to_move(),
        w.to_rotate(),
        w.is_defined(),
        w.is_jumping(),
        w.is_crouching(),
        w.is_running(),
    ]
    .map(|v| v as u8 as f64)
}
fn key_values(c: &AisViewController, key: V) -> [f64; 2] {
    [
        c.keys().is_key_down(key as AspectVKey).unwrap(),
        c.keys().is_free_key(key as AspectVKey).unwrap(),
    ]
    .map(|v| v as u8 as f64)
}
pub(super) fn seed_reset(c: &mut AisViewController) {
    c.input_base.mouse_pressed = rustycad_tkservice::aspect::ASPECT_VKEY_MOUSE_LEFT_BUTTON;
    c.input_base.mouse_modifiers = rustycad_tkservice::aspect::ASPECT_VKEY_FLAGS_CTRL;
    c.click_state.single_button = 1;
    c.click_state.active_gesture = AisMouseGesture::Zoom;
    c.click_state.timer.reset_to(0.75);
    c.click_state.counter = 2;
    c.navigation_state.has_thrust = true;
    c.set_ask_next_frame(true);
    c.key_down(V::NavForward as AspectVKey, 1.25, 0.25).unwrap();
    c.key_down(V::Shift as AspectVKey, 1.5, 1.0).unwrap();
    seed_reset_requests(&mut c.ui_input);
}
fn seed_reset_requests(b: &mut AisViewInputBuffer) {
    b.is_new_gesture = true;
    b.orientation.to_fit_all = true;
    b.orientation.to_set_view_orient = true;
    b.move_to.to_hilight = true;
    b.selection.to_apply_tool = true;
    b.panning.to_start = true;
    b.panning.to_pan = true;
    b.dragging.to_start = true;
    b.dragging.to_confirm = true;
    b.dragging.to_move = true;
    b.dragging.to_stop = true;
    b.orbit_rotation.to_start = true;
    b.orbit_rotation.to_rotate = true;
    b.view_rotation.to_start = true;
    b.view_rotation.to_rotate = true;
    b.z_rotate.to_rotate = true;
    seed_reset_points(b);
}
fn seed_reset_points(b: &mut AisViewInputBuffer) {
    b.selection.points.append(Vec2::new(1, 2));
    b.zoom_actions.append(AspectScrollDelta::with_delta(0.5, 0));
}
pub(super) fn input_snapshot(c: &AisViewController, name: &str) {
    let mut values = input_bookkeeping(c);
    values.extend(input_flags(&c.ui_input));
    let b = &c.ui_input;
    let point = b.selection.points.first().unwrap();
    values.extend([
        b.selection.points.len() as f64,
        point.x as f64,
        point.y as f64,
        b.zoom_actions.len() as f64,
        b.zoom_actions.first().unwrap().delta,
        c.render_input.dragging.to_abort as u8 as f64,
    ]);
    assert_native(&values, name);
}
fn input_bookkeeping(c: &AisViewController) -> Vec<f64> {
    let mut values = vec![
        c.input_base.mouse_pressed as f64,
        c.input_base.mouse_modifiers as f64,
        c.click_state.single_button as f64,
        c.click_state.active_gesture as i32 as f64,
        c.click_state.counter as f64,
        c.click_state.timer.elapsed_time(),
        c.click_state.timer.is_started() as u8 as f64,
        c.navigation_state.last_events_time,
        c.navigation_state.has_thrust as u8 as f64,
        c.to_ask_next_frame() as u8 as f64,
        c.keys().modifiers() as f64,
    ];
    values.extend(key_values(c, V::NavForward));
    values.extend(key_values(c, V::Shift));
    values
}
fn input_flags(b: &AisViewInputBuffer) -> Vec<f64> {
    let mut flags = primary_input_flags(b);
    flags.extend(
        [
            b.orbit_rotation.to_start,
            b.orbit_rotation.to_rotate,
            b.view_rotation.to_start,
            b.view_rotation.to_rotate,
            b.z_rotate.to_rotate,
        ]
        .map(|value| value as u8 as f64),
    );
    flags
}
fn primary_input_flags(b: &AisViewInputBuffer) -> Vec<f64> {
    [
        b.is_new_gesture,
        b.orientation.to_fit_all,
        b.orientation.to_set_view_orient,
        b.move_to.to_hilight,
        b.selection.to_apply_tool,
        b.panning.to_start,
        b.panning.to_pan,
        b.dragging.to_start,
        b.dragging.to_confirm,
        b.dragging.to_move,
        b.dragging.to_stop,
        b.dragging.to_abort,
    ]
    .into_iter()
    .map(|value| value as u8 as f64)
    .collect()
}
