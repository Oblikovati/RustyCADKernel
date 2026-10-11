//! Replay every native touch transition without wall-clock waits or OCCT linkage.
use super::view_controller_keyboard_fixture::prepare;
use super::*;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkservice::aspect::ASPECT_TOUCH_ID_ALL;
fn touch_case(index: usize) -> AisViewController {
    let (mut c, _) = prepare(0.0, 0.0);
    let scales = [1.0, 1.0, 1.0, 0.0, -1.0, 2.0, f32::INFINITY, f32::NAN];
    c.set_touch_tolerance_scale(scales[index % 8]);
    c.set_allow_dragging(index.is_multiple_of(2));
    c.ui_input.move_to.to_hilight = true;
    c.touch_dispatch.update_start_rotation = false;
    c.touch_dispatch.update_start_pan = false;
    if (16..20).contains(&index) {
        seed_touch_timer(&mut c, index);
    }
    c
}
fn seed_touch_timer(c: &mut AisViewController, index: usize) {
    let elapsed = if index == 16 {
        0.2
    } else if index == 17 {
        1.0
    } else {
        0.0
    };
    c.touch_dispatch.double_tap_timer.reset_to(elapsed);
    c.touch_dispatch.double_tap_timer.start();
    if index == 18 {
        c.set_mouse_double_click_interval(-1.0);
    }
    if index == 19 {
        c.set_mouse_double_click_interval(f64::NAN);
    }
}
fn touch_action(c: &mut AisViewController, row: &[f64]) -> bool {
    let id = if row[3] < 0.0 {
        ASPECT_TOUCH_ID_ALL
    } else {
        row[3] as usize
    };
    let point = Vec2::new(row[4], row[5]);
    match row[2] as i32 {
        0 => {
            c.add_touch_point(id, point, row[6] != 0.0).unwrap();
            true
        }
        1 => {
            c.update_touch_point(id, point).unwrap();
            true
        }
        2 => c.remove_touch_point(id, row[6] != 0.0).unwrap(),
        op => panic!("touch operation {op}; expected 0, 1 or 2"),
    }
}
fn touch_snapshot(c: &AisViewController, result: bool) -> Vec<f64> {
    let mut values = vec![
        result as i32 as f64,
        c.input_base.touch_points.extent() as f64,
    ];
    append_touch_coordinates(c, &mut values);
    append_touch_requests(c, &mut values);
    append_touch_selection(c, &mut values);
    append_touch_slots(c, &mut values);
    values
}
fn append_touch_coordinates(c: &AisViewController, values: &mut Vec<f64>) {
    let t = &c.touch_dispatch;
    values.extend([
        t.click.from.x,
        t.click.from.y,
        t.start_rotation.x,
        t.start_rotation.y,
        t.start_pan.x,
        t.start_pan.y,
        t.update_start_rotation as i32 as f64,
        t.update_start_pan as i32 as f64,
    ]);
}
fn append_touch_requests(c: &AisViewController, values: &mut Vec<f64>) {
    let ui = &c.ui_input;
    values.extend([
        ui.move_to.to_hilight as i32 as f64,
        ui.is_new_gesture as i32 as f64,
        ui.dragging.to_start as i32 as f64,
        ui.dragging.to_abort as i32 as f64,
        ui.dragging.to_stop as i32 as f64,
        ui.dragging.point_start.x as f64,
        ui.dragging.point_start.y as f64,
        ui.selection.to_apply_tool as i32 as f64,
    ]);
}
fn append_touch_selection(c: &AisViewController, values: &mut Vec<f64>) {
    let selection = &c.ui_input.selection;
    let clicked = selection.points.last().copied().unwrap_or(Vec2::new(0, 0));
    values.extend([
        selection.tool as i32 as f64,
        selection.scheme as i32 as f64,
        selection.points.len() as f64,
        selection.points.len() as f64,
        0.0,
        clicked.x as f64,
        clicked.y as f64,
        c.touch_dispatch.double_tap_timer.is_started() as i32 as f64,
    ]);
}
fn append_touch_slots(c: &AisViewController, values: &mut Vec<f64>) {
    let touches = &c.input_base.touch_points;
    for index in 1..=3 {
        let Ok(touch) = touches.find_from_index(index) else {
            values.extend([-1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
            continue;
        };
        values.extend([
            *touches.find_key(index).unwrap() as f64,
            touch.from.x,
            touch.from.y,
            touch.to.x,
            touch.to.y,
            touch.is_precise_device as i32 as f64,
        ]);
    }
}
fn assert_touch_snapshot(c: &AisViewController, result: bool, row: &[f64]) {
    let actual = touch_snapshot(c, result);
    assert_eq!(row.len(), 51, "native touch snapshot; expected 51 columns");
    for (index, (&value, &expected)) in actual.iter().zip(&row[7..]).enumerate() {
        if index == 22 {
            continue;
        } // The native controller ignores the double-click callback flag.
        if value.is_nan() && expected.is_nan() {
            continue;
        }
        assert_eq!(
            value,
            expected,
            "case {} step {} column {}",
            row[0],
            row[1],
            index + 7
        );
    }
}
#[test]
fn touch_sequences_replay_all_committed_native_add_move_remove_transitions() {
    let fixture = include_str!("../../tests/fixtures/view-controller-touch-native.txt");
    assert_eq!(fixture.lines().count(), 254);
    let mut c = touch_case(0);
    for line in fixture.lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        if row[1] == 0.0 {
            c = touch_case(row[0] as usize);
        }
        let result = touch_action(&mut c, &row);
        assert_touch_snapshot(&c, result, &row);
        assert!(c.render_input.selection.points.is_empty());
    }
}
#[test]
fn touch_double_tap_interval_matches_native_recent_expired_negative_and_nan_cases() {
    for index in 16..20 {
        let prefix = format!("{index} 1 ");
        let expected = include_str!("../../tests/fixtures/view-controller-touch-native.txt")
            .lines()
            .find(|row| row.starts_with(&prefix))
            .unwrap()
            .split_whitespace()
            .nth(29)
            .unwrap()
            == "1";
        let mut c = touch_case(index);
        assert_eq!(c.touch_double_click(), expected);
        assert!(c.touch_dispatch.double_tap_timer.is_started());
    }
}
#[test]
fn touch_double_tap_accepts_exact_interval_and_restarts_after_expiration() {
    let (mut c, clock) = prepare(0.0, 0.0);
    assert!(!c.touch_double_click());
    clock.set(0.4);
    assert!(c.touch_double_click());
    clock.set(0.4001);
    assert!(!c.touch_double_click());
    clock.set(0.4002);
    assert!(c.touch_double_click());
}
#[test]
fn touch_ordered_nan_component_comparisons_preserve_native_click_cancel_rules() {
    let mut c = touch_case(0);
    c.add_touch_point(1, Vec2::new(1.0, 2.0), false).unwrap();
    c.update_touch_point(1, Vec2::new(200.0, f64::NAN)).unwrap();
    assert_eq!(c.touch_dispatch.click.from, Vec2::new(1.0, 2.0));
    c.update_touch_point(1, Vec2::new(1.0, 200.0)).unwrap();
    assert_eq!(c.touch_dispatch.click.from, Vec2::new(-1.0, -1.0));
}

#[test]
fn failing_touch_click_pause_retains_prior_removal_but_does_not_mark_new_gesture() {
    use super::view_controller_click_tests::ClickPauseRecorder;
    use rustycad_tkernel::standard::Handle;
    let mut c = touch_case(0);
    let animation = Handle::new(ClickPauseRecorder::new(true));
    animation.start(false).unwrap();
    c.set_objects_animation(&animation.cast::<dyn AisAnimationSource>());
    c.set_pause_objects_animation(true);
    c.add_touch_point(1, Vec2::new(1.0, 2.0), false).unwrap();
    c.ui_input.is_new_gesture = false;
    assert!(c.remove_touch_point(1, true).is_err());
    assert!(c.input_base.touch_points.is_empty());
    assert!(c.ui_input.dragging.to_stop && c.ui_input.selection.to_apply_tool);
    assert!(!c.ui_input.is_new_gesture);
    assert!(c.ui_input.selection.points.is_empty());
}
