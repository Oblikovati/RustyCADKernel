use super::super::{Graphic3dFrameStatsCounter as Counter, Graphic3dFrameStatsTimer as TimerKind};
use super::*;

type AssignmentRow = [f64; 11];
type AssignmentFrame = Graphic3dFrameStatsDataTmp;

fn assignment_source_frame() -> Graphic3dFrameStatsDataTmp {
    let mut source = Graphic3dFrameStatsDataTmp::default();
    *source.change_frame_rate() = 60.0;
    *source.change_frame_rate_cpu() = 90.0;
    *source.change_immediate_frame_rate() = 120.0;
    *source.change_immediate_frame_rate_cpu() = 180.0;
    source[Counter::NbLayers] = 42;
    source[TimerKind::CpuFrame] = 3.0;
    source.flush_timers(1, false);
    source
}

fn assignment_frame_row(frame: &mut AssignmentFrame, step: u32, timer: usize) -> AssignmentRow {
    let running = frame.change_timer(TimerKind::CpuFrame).is_started();
    let same_timer = std::ptr::from_ref(frame.change_timer(TimerKind::CpuFrame)) as usize == timer;
    [
        f64::from(step),
        frame.frame_rate(),
        frame.frame_rate_cpu(),
        frame.immediate_frame_rate(),
        frame.immediate_frame_rate_cpu(),
        frame.counter_value(Counter::NbLayers) as f64,
        frame.timer_value(TimerKind::CpuFrame),
        frame.data.timers_min[TimerKind::CpuFrame as usize],
        frame.data.timers_max[TimerKind::CpuFrame as usize],
        f64::from(running),
        f64::from(same_timer),
    ]
}

fn assignment_frame_trace() -> Vec<AssignmentRow> {
    let mut frame = Graphic3dFrameStatsDataTmp::default();
    let mut source = assignment_source_frame();
    let timer = std::ptr::from_ref(frame.change_timer(TimerKind::CpuFrame)) as usize;
    frame.change_timer(TimerKind::CpuFrame).start();
    frame[TimerKind::CpuFrame] = 8.0;
    frame.flush_timers(1, false);
    let mut trace = vec![assignment_frame_row(&mut frame, 0, timer)];
    frame.assign_data(&source);
    trace.push(assignment_frame_row(&mut frame, 1, timer));
    frame[TimerKind::CpuFrame] = 11.0;
    frame.flush_timers(1, false);
    trace.push(assignment_frame_row(&mut frame, 2, timer));
    finish_assignment_frame_trace(&mut frame, &mut source, timer, &mut trace);
    trace
}

fn finish_assignment_frame_trace(
    frame: &mut Graphic3dFrameStatsDataTmp,
    source: &mut Graphic3dFrameStatsDataTmp,
    timer: usize,
    trace: &mut Vec<AssignmentRow>,
) {
    source[TimerKind::CpuFrame] = 4.0;
    source.flush_timers(1, false);
    frame.assign_data(source);
    trace.push(assignment_frame_row(frame, 3, timer));
    frame[TimerKind::CpuFrame] = 16.0;
    frame.flush_timers(2, true);
    trace.push(assignment_frame_row(frame, 4, timer));
    frame.change_timer(TimerKind::CpuFrame).stop();
    frame.reset();
    trace.push(assignment_frame_row(frame, 5, timer));
}

fn expected_assignment_frame_rows() -> Vec<AssignmentRow> {
    include_str!("../../tests/fixtures/frame-stats-assignment.txt")
        .lines()
        .map(|line| {
            let columns: Vec<f64> = line
                .split_whitespace()
                .map(|v| v.parse().unwrap())
                .collect();
            assert_eq!(columns.len(), 11, "native row={line}; expected 11 columns");
            columns.try_into().unwrap()
        })
        .collect()
}

#[test]
fn actual_native_assignment_preserves_running_timer_and_previous_samples() {
    let actual = assignment_frame_trace();
    let expected = expected_assignment_frame_rows();
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(actual.map(f64::to_bits), expected.map(f64::to_bits));
    }
}
