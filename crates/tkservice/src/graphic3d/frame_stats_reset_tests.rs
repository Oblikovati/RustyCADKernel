use super::*;

type ResetStorageAddresses = [usize; 4];

fn reset_storage_addresses(frame: &Graphic3dFrameStatsData) -> ResetStorageAddresses {
    [
        frame.counters.as_ptr() as usize,
        frame.timers.as_ptr() as usize,
        frame.timers_min.as_ptr() as usize,
        frame.timers_max.as_ptr() as usize,
    ]
}

fn seed_reset_frame(frame: &mut Graphic3dFrameStatsData) {
    frame.counters.fill(42);
    frame.timers.fill(f64::NAN);
    frame.timers_min.fill(-3.0);
    frame.timers_max.fill(f64::INFINITY);
    frame.fps = 60.0;
    frame.fps_cpu = 90.0;
    frame.fps_immediate = 120.0;
    frame.fps_cpu_immediate = 180.0;
}

fn assert_native_reset_values(frame: &Graphic3dFrameStatsData) {
    assert!(frame.counters.iter().all(|value| *value == 0));
    assert!(frame.timers.iter().all(|value| value.to_bits() == 0));
    assert!(frame.timers_min.iter().all(|value| *value == real_last()));
    assert!(frame.timers_max.iter().all(|value| value.to_bits() == 0));
    assert_eq!(frame.frame_rate(), -1.0);
    assert_eq!(frame.frame_rate_cpu(), -1.0);
    assert_eq!(frame.immediate_frame_rate(), -1.0);
    assert_eq!(frame.immediate_frame_rate_cpu(), -1.0);
}

#[test]
fn frame_reset_preserves_all_four_native_vector_storage_addresses() {
    let mut frame = Graphic3dFrameStatsData::default();
    seed_reset_frame(&mut frame);
    let original = reset_storage_addresses(&frame);
    frame.reset();
    assert_eq!(reset_storage_addresses(&frame), original);
    assert_native_reset_values(&frame);
    frame.reset();
    assert_eq!(reset_storage_addresses(&frame), original);
    assert_native_reset_values(&frame);
}

fn reset_public_field_addresses(frame: &mut Graphic3dFrameStatsDataTmp) -> [usize; 2] {
    let counter =
        std::ptr::from_mut(frame.change_counter_value(Graphic3dFrameStatsCounter::NbLayers));
    let timer = std::ptr::from_mut(frame.change_timer_value(Graphic3dFrameStatsTimer::CpuFrame));
    [counter as usize, timer as usize]
}

#[test]
fn temporary_frame_reset_keeps_actual_counter_and_timer_fields() {
    let mut frame = Graphic3dFrameStatsDataTmp::default();
    seed_reset_frame(&mut frame);
    let original = reset_storage_addresses(&frame);
    let fields = reset_public_field_addresses(&mut frame);
    frame.reset();
    assert_eq!(reset_storage_addresses(&frame), original);
    assert_eq!(reset_public_field_addresses(&mut frame), fields);
    assert_native_reset_values(&frame);
}

fn reset_native_row(
    frame: &Graphic3dFrameStatsData,
    original: ResetStorageAddresses,
    step: usize,
) -> [usize; 6] {
    let unchanged = reset_storage_addresses(frame);
    [
        step,
        usize::from(unchanged[0] == original[0]),
        usize::from(unchanged[1] == original[1]),
        usize::from(unchanged[2] == original[2]),
        usize::from(unchanged[3] == original[3]),
        usize::from(frame == &Graphic3dFrameStatsData::default()),
    ]
}

fn reset_native_trace() -> Vec<[usize; 6]> {
    let mut frame = Graphic3dFrameStatsDataTmp::default();
    seed_reset_frame(&mut frame);
    let original = reset_storage_addresses(&frame);
    frame.data.reset();
    let mut rows = vec![reset_native_row(&frame, original, 1)];
    seed_reset_frame(&mut frame);
    frame.reset();
    rows.push(reset_native_row(&frame, original, 2));
    frame.reset();
    rows.push(reset_native_row(&frame, original, 3));
    rows
}

#[test]
fn reset_replays_native_base_temporary_and_repeated_reset_fields() {
    let expected: Vec<[usize; 6]> = include_str!("../../tests/fixtures/frame-stats-reset.txt")
        .lines()
        .map(|line| {
            let values: Vec<usize> = line
                .split_whitespace()
                .map(|value| value.parse().unwrap())
                .collect();
            values.try_into().unwrap_or_else(|actual| {
                panic!("native reset row={actual:?}; expected six integer columns")
            })
        })
        .collect();
    assert_eq!(reset_native_trace(), expected);
}
