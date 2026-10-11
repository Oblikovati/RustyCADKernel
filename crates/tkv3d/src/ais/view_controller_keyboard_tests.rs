//! All frozen native keyboard scenarios replay without an OCCT runtime dependency.
use super::view_controller_keyboard_fixture::*;
use rustycad_tkservice::aspect::{AspectVKey, AspectVKeyBasic as V};
#[test]
fn controller_all_navigation_keys_preserve_release_consumption() {
    for (index, key) in NAVIGATION_KEYS.into_iter().enumerate() {
        let (mut c, clock) = prepare(1., 2.);
        c.key_down(key as AspectVKey, 1.25, 0.25).unwrap();
        fetch(&mut c, &format!("single-{index}"), 0.5, 2.);
        c.key_up(key as AspectVKey, 2.25).unwrap();
        frame(&mut c, &clock, 2., 3.);
        fetch(&mut c, &format!("released-{index}"), 0.5, 2.);
        frame(&mut c, &clock, 3., 4.);
        fetch(&mut c, &format!("consumed-{index}"), 0.5, 2.);
    }
}
#[test]
fn controller_opposing_keys_follow_native_assignment_order_and_pitch_inversion() {
    for pair in 0..6 {
        let (mut c, clock) = prepare(1., 2.);
        c.key_down(NAVIGATION_KEYS[pair * 2] as AspectVKey, 1.25, 0.25)
            .unwrap();
        c.key_down(NAVIGATION_KEYS[pair * 2 + 1] as AspectVKey, 1.5, 0.75)
            .unwrap();
        fetch(&mut c, &format!("pair-{pair}"), 0.5, 2.);
        c.set_invert_pitch(true);
        frame(&mut c, &clock, 2., 3.);
        fetch(&mut c, &format!("pair-inverted-{pair}"), 0.5, 2.);
    }
}
#[test]
fn controller_running_crouching_jumping_precedence_matches_native() {
    for mask in 0..32 {
        check_stance(mask);
    }
}
fn check_stance(mask: i32) {
    let (mut c, _) = prepare(1., 2.);
    for (key, time, pressure) in [
        (V::NavForward, 1.25, 0.25),
        (V::NavSlideLeft, 1.5, 0.5),
        (V::NavSlideUp, 1.75, 0.75),
    ] {
        c.key_down(key as AspectVKey, time, pressure).unwrap();
    }
    for (bit, key) in [(1, V::Shift), (2, V::NavCrouch), (4, V::NavJump)] {
        if mask & bit != 0 {
            c.key_down(key as AspectVKey, 1., 1.).unwrap();
        }
    }
    let crouch = if mask & 16 != 0 { 0.5 } else { 1. };
    let run = if mask & 8 != 0 { 2. } else { 1. };
    fetch(&mut c, &format!("stance-{mask}"), crouch, run);
}
#[test]
fn controller_jump_release_is_consumed_once_after_the_initial_jump() {
    let (mut c, clock) = prepare(1., 2.);
    c.key_down(V::NavJump as AspectVKey, 1., 1.).unwrap();
    fetch(&mut c, "jump-first", 0.5, 2.);
    frame(&mut c, &clock, 2., 3.);
    fetch(&mut c, "jump-released", 0.5, 2.);
    frame(&mut c, &clock, 3., 4.);
    fetch(&mut c, "jump-consumed", 0.5, 2.);
}
#[test]
fn controller_pressure_is_metadata_for_translation_and_a_factor_for_rotation() {
    for (index, key) in NAVIGATION_KEYS.into_iter().enumerate() {
        for (value, pressure) in [-1., 0., 2., f64::NAN, f64::INFINITY]
            .into_iter()
            .enumerate()
        {
            let (mut c, _) = prepare(1., 2.);
            c.key_down(key as AspectVKey, 1.25, pressure).unwrap();
            fetch(&mut c, &format!("pressure-{index}-{value}"), 0.5, 2.);
        }
    }
}
#[test]
fn controller_frame_and_key_time_boundaries_preserve_native_minimum_semantics() {
    let times = [
        [1., 2., 3.],
        [2., 1., 0.],
        [f64::NAN, 2., 1.],
        [1., f64::NAN, 1.],
        [1., 2., f64::NAN],
        [1., 2., f64::INFINITY],
        [1., 2., f64::NEG_INFINITY],
        [1., f64::INFINITY, 1.],
    ];
    for (index, [previous, current, down]) in times.into_iter().enumerate() {
        let (mut c, _) = prepare(previous, current);
        c.key_down(V::NavForward as AspectVKey, down, 0.25).unwrap();
        fetch(&mut c, &format!("timing-{index}"), 0.5, 2.);
    }
}
#[test]
fn controller_thrust_first_next_stop_resume_transitions_match_native() {
    for (index, speed) in [
        0_f32,
        -0.,
        1.,
        -1.,
        f32::from_bits(1),
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NAN,
    ]
    .into_iter()
    .enumerate()
    {
        check_thrust(index, speed);
    }
}
fn check_thrust(index: usize, speed: f32) {
    let (mut c, clock) = prepare(1., 2.);
    c.set_thrust_speed(speed);
    fetch(&mut c, &format!("thrust-first-{index}"), 0.5, 2.);
    frame(&mut c, &clock, 2., 3.);
    fetch(&mut c, &format!("thrust-next-{index}"), 0.5, 2.);
    c.set_thrust_speed(0.);
    c.set_ask_next_frame(false);
    frame(&mut c, &clock, 3., 4.);
    fetch(&mut c, &format!("thrust-stop-{index}"), 0.5, 2.);
    c.set_thrust_speed(speed);
    frame(&mut c, &clock, 4., 5.);
    fetch(&mut c, &format!("thrust-resume-{index}"), 0.5, 2.);
}
#[test]
fn controller_signed_axis_pressure_and_nonfinite_values_match_native_cache() {
    for (index, pressure) in [-0.75, 0.25, 0., f64::NAN, f64::INFINITY, f64::NEG_INFINITY]
        .into_iter()
        .enumerate()
    {
        let (mut c, _) = prepare(1., 2.);
        c.key_from_axis(
            V::NavBackward as AspectVKey,
            V::NavForward as AspectVKey,
            1.25,
            pressure,
        )
        .unwrap();
        fetch(&mut c, &format!("axis-{index}"), 0.5, 2.);
    }
}
#[test]
fn controller_axis_changes_preserve_the_released_opposite_key_for_one_fetch() {
    let (mut c, clock) = prepare(1., 2.);
    for (name, previous, current, pressure) in [
        ("axis-negative", 1., 2., -0.75),
        ("axis-positive", 2., 3., 0.25),
        ("axis-zero", 3., 4., 0.),
    ] {
        frame(&mut c, &clock, previous, current);
        c.key_from_axis(
            V::NavBackward as AspectVKey,
            V::NavForward as AspectVKey,
            previous + 0.25,
            pressure,
        )
        .unwrap();
        fetch(&mut c, name, 0.5, 2.);
    }
    frame(&mut c, &clock, 4., 5.);
    fetch(&mut c, "axis-consumed", 0.5, 2.);
}
#[test]
fn controller_focus_loss_and_native_empty_callbacks_preserve_other_pending_events() {
    let (mut c, _) = prepare(1., 2.);
    seed_reset(&mut c);
    input_snapshot(&c, "input-seeded");
    c.process_expose();
    c.process_configure(false);
    c.process_configure(true);
    c.process_input();
    c.process_close();
    input_snapshot(&c, "input-noops");
    c.process_focus(true);
    input_snapshot(&c, "input-active");
    c.process_focus(false);
    input_snapshot(&c, "input-focus-lost");
    c.reset_view_input();
    input_snapshot(&c, "input-reset-again");
}
#[test]
fn controller_input_reset_stops_the_injected_click_timer_without_zeroing_elapsed() {
    let (mut c, clock) = prepare(1., 2.);
    seed_reset(&mut c);
    c.click_state.timer.start();
    clock.set(2.5);
    c.reset_view_input();
    assert!(!c.click_state.timer.is_started());
    assert_eq!(c.click_state.timer.elapsed_time(), 1.25);
}
#[test]
fn controller_event_times_advance_only_when_requested_with_injected_clocks() {
    let (mut c, clock) = prepare(0., 1.);
    assert_eq!(c.input_base().event_timer.elapsed_time(), 1.);
    let (previous, current) = c.update_events_time();
    assert_native(&[previous, current], "events-first");
    clock.set(2.);
    assert_eq!(c.event_time(), 2.);
    let (previous, current) = c.update_events_time();
    assert_native(&[previous, current], "events-second");
    c.input_base_mut().event_timer.reset_to(3.);
    assert_eq!(c.event_time(), 3.);
}
#[test]
fn controller_keyboard_accessors_share_the_native_cache() {
    let (mut c, _) = prepare(1., 2.);
    let address = c.keys() as *const _;
    assert_eq!(address, c.change_keys() as *const _);
    c.change_keys()
        .key_down(V::NavForward as AspectVKey, 1.25, 0.25)
        .unwrap();
    fetch(&mut c, "single-0", 0.5, 2.);
}
#[test]
fn controller_keyboard_forwarders_report_out_of_range_keys_without_mutating_input() {
    let (mut c, _) = prepare(1., 2.);
    assert!(c.key_down(u32::MAX, 1.25, 1.).is_err());
    assert!(c.key_up(u32::MAX, 1.25).is_err());
    assert!(c.key_from_axis(u32::MAX, u32::MAX, 1.25, 1.).is_err());
    assert_eq!(c.keys().modifiers(), 0);
}

#[test]
fn controller_stance_ratios_preserve_unclamped_negative_zero_and_nonfinite_values() {
    for (index, (crouch, run)) in [
        (-1., -1.),
        (f64::NAN, f64::INFINITY),
        (f64::NEG_INFINITY, f64::NAN),
        (f64::INFINITY, f64::INFINITY),
        (0., 2.),
    ]
    .into_iter()
    .enumerate()
    {
        let (mut c, _) = prepare(1., 2.);
        c.key_down(V::Shift as AspectVKey, 1., 1.).unwrap();
        c.key_down(V::NavCrouch as AspectVKey, 1., 1.).unwrap();
        c.key_down(V::NavForward as AspectVKey, 1.25, 0.25).unwrap();
        c.key_down(V::NavSlideLeft as AspectVKey, 1.5, 0.5).unwrap();
        c.key_down(V::NavSlideUp as AspectVKey, 1.75, 0.75).unwrap();
        fetch(&mut c, &format!("ratio-{index}"), crouch, run);
    }
}
#[test]
fn controller_repeated_key_press_changes_pressure_without_restarting_duration() {
    let (mut c, _) = prepare(1., 2.);
    c.key_down(V::NavForward as AspectVKey, 1.25, 0.25).unwrap();
    c.key_down(V::NavForward as AspectVKey, 1.5, 0.75).unwrap();
    fetch(&mut c, "key-repeated", 0.5, 2.);
}
