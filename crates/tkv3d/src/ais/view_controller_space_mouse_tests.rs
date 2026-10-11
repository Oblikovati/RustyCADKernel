//! Replay full native HID-to-key lifecycle with injected event time and no OCCT dependency.
use super::AisViewController;
use super::view_controller_keyboard_fixture::prepare;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkservice::{aspect::ASPECT_VKEY_UPPER, wnt::WntHidSpaceMouse};
fn configure_space_controller(c: &mut AisViewController, index: usize) {
    let trans = [2.0, 0.0, -2.0, f32::INFINITY, f32::NAN];
    let rot = [4.0, 0.0, -1.0, f32::INFINITY, f32::NAN];
    c.set_allow_rotation(!index.is_multiple_of(3));
    let base = c.input_base_mut();
    base.mouse3d_is_quadric = index.is_multiple_of(2);
    base.mouse3d_accel_trans = trans[index % 5];
    base.mouse3d_accel_rotate = rot[index % 5];
    base.mouse3d_no_rotate = Vec3::new(
        (index / 3) & 1 != 0,
        (index / 3) & 2 != 0,
        (index / 3) & 4 != 0,
    );
    base.mouse3d_to_reverse = Vec3::new(
        (index / 7) & 1 != 0,
        (index / 7) & 2 != 0,
        (index / 7) & 4 != 0,
    );
}
fn space_controller_snapshot(
    c: &AisViewController,
    result: bool,
    report: &WntHidSpaceMouse<'_>,
    time: f64,
) -> Vec<f64> {
    let mut v = vec![
        result as i32 as f64,
        report.raw_value_range() as f64,
        c.keys().modifiers() as f64,
    ];
    v.extend(
        c.input_base()
            .mouse3d_button_state
            .map(|pressed| pressed as i32 as f64),
    );
    append_space_key_lifecycles(c, time, &mut v);
    v
}
#[test]
fn controller_space_mouse_matches_native_key_state_and_hold_lifecycles() {
    let (mut c, clock) = prepare(0.0, 0.0);
    for line in include_str!("../../tests/fixtures/view-controller-space-mouse-native.txt").lines()
    {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|n| n.parse().unwrap())
            .collect();
        if row[1] == 0.0 {
            c = AisViewController::with_event_timer(c.input_base().event_timer.clone());
            configure_space_controller(&mut c, row[0] as usize);
        }
        let time = 1.0 + row[1] * 0.5;
        // A cloned running timer retains its old start offset; reset before each injected frame.
        c.input_base_mut().event_timer.stop();
        c.input_base_mut().event_timer.reset_to(time);
        clock.set(time);
        replay_space_row(&mut c, &row, time);
    }
}
fn replay_space_row(c: &mut AisViewController, row: &[f64], time: f64) {
    let bytes: Vec<u8> = row[4..17].iter().map(|b| *b as u8).collect();
    let report = WntHidSpaceMouse::new(row[2] as u64, &bytes[..row[3] as usize]);
    let result = c.update_3d_mouse(&report).unwrap();
    let actual = space_controller_snapshot(c, result, &report, time);
    assert_eq!(actual.len(), row.len() - 17);
    for (column, (actual, expected)) in actual.iter().zip(&row[17..]).enumerate() {
        assert!(
            actual == expected || (actual.is_nan() && expected.is_nan()),
            "case {} step {} column {column}: got {actual}, expected {expected}",
            row[0],
            row[1]
        );
    }
    assert!(!c.ui_input.is_new_gesture && c.ui_input.zoom_actions.is_empty());
    assert!(!c.ui_input.orbit_rotation.to_rotate && !c.ui_input.view_rotation.to_rotate);
}

fn append_space_key_lifecycles(c: &AisViewController, time: f64, v: &mut Vec<f64>) {
    for key in 0..=ASPECT_VKEY_UPPER {
        let down = c.keys().is_key_down(key).unwrap();
        let free = c.keys().is_free_key(key).unwrap();
        let (held, duration, pressure) = c
            .keys()
            .hold_duration_pressure(key, time + 0.125, -1.0)
            .unwrap();
        v.extend([
            down as i32 as f64,
            free as i32 as f64,
            held as i32 as f64,
            duration,
            pressure,
        ]);
    }
}
