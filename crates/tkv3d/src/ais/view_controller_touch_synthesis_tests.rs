//! Native snapshots include contacts, queued motion, starts and real view invalidation.
use super::view_controller_touch_synthesis_fixture::*;
use super::{AisViewController, view_controller_transfer_fixture::queue_snapshot};
use rustycad_tkernel::{ncollection::Vec2, standard::Handle};
use rustycad_tkservice::aspect::AspectTouch;
#[test]
fn pinch_negates_y_before_converting_at_signed_integer_boundary() {
    let mut c = AisViewController::new().unwrap();
    c.touch_dispatch.update_start_pan = false;
    for index in 0..2 {
        let mut touch = AspectTouch::at(f64::from(index * 20), -1_073_741_824., false);
        touch.to = Vec2::new(f64::from(index * 20), 1_073_741_824.);
        c.input_base.touch_points.add(index as usize, touch);
    }
    c.flush_gestures(&Handle::default(), &Handle::default())
        .unwrap();
    let expected: Vec<i32> = include_str!("../../tests/fixtures/touch-pan-boundary-native.txt")
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(
        c.render_input.panning.delta,
        Vec2::new(expected[0], expected[1])
    );
}
#[test]
fn gesture_synthesis_matches_native_contact_modes_tolerances_and_three_frames() {
    for fixture in [
        include_str!("../../tests/fixtures/view-controller-gestures-0-native.txt"),
        include_str!("../../tests/fixtures/view-controller-gestures-1-native.txt"),
        include_str!("../../tests/fixtures/view-controller-gestures-2-native.txt"),
    ] {
        replay_synthesis_chunk(fixture);
    }
}
fn replay_synthesis_chunk(fixture: &str) {
    let (_viewer, view) = synthesis_view();
    let mut c = AisViewController::new().unwrap();
    for line in fixture.lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        if row[1] == 0.0 {
            c = synthesis_case(row[0] as usize);
        }
        replay_synthesis_frame(&mut c, &view, &row);
    }
}
fn replay_synthesis_frame(
    c: &mut AisViewController,
    view: &Handle<dyn crate::v3d::V3dViewSource>,
    row: &[f64],
) {
    advance_touch_case(c, row[0] as usize, row[1] as usize);
    view.get().unwrap().view().get().unwrap().redraw();
    c.flush_gestures(&Handle::default(), view).unwrap();
    assert_eq!(
        synthesis_snapshot(c, view),
        row[2..],
        "case {} frame {}",
        row[0],
        row[1]
    );
}

#[test]
fn synthesis_preserves_ui_and_unrelated_render_requests() {
    let (_viewer, view) = synthesis_view();
    let mut c = synthesis_case(1);
    c.ui_input.is_new_gesture = true;
    c.ui_input.selection.points.append(Vec2::new(2, 3));
    c.render_input.dragging.to_abort = true;
    let ui_before = queue_snapshot(&c.ui_input);
    c.flush_gestures(&Handle::default(), &view).unwrap();
    assert_eq!(queue_snapshot(&c.ui_input), ui_before);
    assert!(c.render_input.dragging.to_abort);
}
#[test]
fn missing_view_failure_preserves_native_start_mutations() {
    let mut c = synthesis_case(1);
    assert!(
        c.flush_gestures(&Handle::default(), &Handle::default())
            .is_err()
    );
    assert!(!c.touch_dispatch.update_start_rotation);
    assert!(c.render_input.orbit_rotation.to_start);
    assert!(c.render_input.is_new_gesture);
    assert_eq!(c.touch_dispatch.last_touch_count, 1);
}
#[test]
fn contact_count_transitions_are_detected_without_motion_or_view() {
    let mut c = AisViewController::new().unwrap();
    c.flush_gestures(&Handle::default(), &Handle::default())
        .unwrap();
    assert!(!c.render_input.is_new_gesture);
    for index in 0..3 {
        c.input_base.touch_points.add(index, AspectTouch::default());
    }
    c.flush_gestures(&Handle::default(), &Handle::default())
        .unwrap();
    assert_eq!(c.touch_dispatch.last_touch_count, 3);
    assert!(c.render_input.is_new_gesture);
    c.render_input.is_new_gesture = false;
    c.input_base.touch_points.clear(false);
    c.flush_gestures(&Handle::default(), &Handle::default())
        .unwrap();
    assert!(c.render_input.is_new_gesture);
    assert_eq!(c.touch_dispatch.last_touch_count, 0);
}
#[test]
fn pinch_view_failure_retains_queued_zoom_without_advancing_contact_origins() {
    let mut c = synthesis_case(2);
    c.touch_dispatch.update_start_pan = true;
    c.input_base.touch_points.change_from_index(1).unwrap().to = Vec2::new(10.0, 20.0);
    c.input_base.touch_points.change_from_index(2).unwrap().to = Vec2::new(60.0, 20.0);
    let origin = c.input_base.touch_points.find_from_index(1).unwrap().from;
    assert!(
        c.flush_gestures(&Handle::default(), &Handle::default())
            .is_err()
    );
    assert_eq!(c.render_input.zoom_actions.len(), 1);
    assert!(c.render_input.panning.to_start);
    assert!(!c.render_input.panning.to_pan);
    assert!(!c.touch_dispatch.update_start_pan);
    assert_eq!(
        c.input_base.touch_points.find_from_index(1).unwrap().from,
        origin
    );
}
