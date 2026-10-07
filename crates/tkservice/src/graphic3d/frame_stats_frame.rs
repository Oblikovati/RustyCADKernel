//! `Graphic3d_FrameStats::FrameStart` and `FrameEnd`: the frame timing, the FPS meters and
//! the rotation of the data frame history.
use super::frame_stats::Graphic3dFrameStatsFields;
use super::{
    GRAPHIC3D_FRAME_STATS_COUNTER_IMMEDIATE_LOWER, GRAPHIC3D_FRAME_STATS_COUNTER_RENDERED_LOWER,
    GRAPHIC3D_FRAME_STATS_COUNTER_RENDERED_UPPER, Graphic3dCView, Graphic3dCViewMembers as _,
    Graphic3dFrameStats, Graphic3dFrameStatsCounter as Counter, Graphic3dFrameStatsData,
    Graphic3dFrameStatsTimer as Timer, Graphic3dRenderingParamsPerfCounters as Perf,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp;

/// The collected statistics flags of the view (none without a view) and whether an
/// immediate-only frame is skipped.
fn skipped(view: &Handle<dyn Graphic3dCView>, is_immediate_only: bool) -> bool {
    let bits = view
        .get()
        .map_or(Perf::NONE, |v| v.rendering_params().stats.collected_stats);
    is_immediate_only && (bits.0 & Perf::SKIP_IMMEDIATE.0) != 0
}

/// The native `FrameStart` body: resizes the history to the view's frame count and starts
/// the FPS timer of a new measurement.
pub(crate) fn frame_start<S: Graphic3dFrameStats + ?Sized>(
    stats: &S,
    view: &Handle<dyn Graphic3dCView>,
    is_immediate_only: bool,
) {
    if skipped(view, is_immediate_only) {
        return;
    }
    let nb_frames = view
        .get()
        .map_or(1, |v| v.rendering_params().stats.stats_nb_frames)
        .max(1);
    let mut fields = stats.frame_stats_base().fields.write();
    if fields.counters.len() != nb_frames as usize {
        fields.counters = vec![Graphic3dFrameStatsData::default(); nb_frames as usize];
        fields.last_frame_index = nb_frames - 1;
    }
    fields.frame_start_time = fields.fps_timer.elapsed_time();
    if !fields.fps_timer.is_started() {
        fields.fps_timer.reset();
        fields.fps_timer.start();
        fields.fps_frame_count = 0;
    }
}

/// Closes a measurement slice: the elapsed and CPU frame rates (immediate or not), the
/// averaged timers, the maxima, and a restarted FPS timer.
fn close_measurement(
    fields: &mut Graphic3dFrameStatsFields,
    time: f64,
    previous: &Graphic3dFrameStatsData,
    is_immediate_only: bool,
) {
    fields.fps_timer.stop();
    let cpu = fields.fps_timer.user_time_cpu();
    let count = fields.fps_frame_count as f64;
    let cpu_rate = if cpu > gp::resolution() {
        count / cpu
    } else {
        -1.0
    };
    let tmp = &mut fields.counters_tmp;
    tmp[Timer::ElapsedFrame] = time;
    tmp[Timer::CpuFrame] = cpu;
    let rates = if is_immediate_only {
        [
            previous.frame_rate(),
            previous.frame_rate_cpu(),
            count / time,
            cpu_rate,
        ]
    } else {
        [count / time, cpu_rate, -1.0, -1.0]
    };
    *tmp.change_frame_rate() = rates[0];
    *tmp.change_frame_rate_cpu() = rates[1];
    *tmp.change_immediate_frame_rate() = rates[2];
    *tmp.change_immediate_frame_rate_cpu() = rates[3];
    tmp.flush_timers(fields.fps_frame_count, true);
    let flushed = (*fields.counters_tmp).clone();
    fields.counters_max.fill_max(&flushed);
    fields.fps_timer.reset();
    fields.fps_timer.start();
    fields.fps_frame_count = 0;
}

/// Copies the rendered counters of an immediate frame to the immediate counters and takes
/// the rendered counters (and the geometry memory) of the previous frame.
fn merge_immediate(fields: &mut Graphic3dFrameStatsFields, previous: &Graphic3dFrameStatsData) {
    let shift = (GRAPHIC3D_FRAME_STATS_COUNTER_IMMEDIATE_LOWER
        - GRAPHIC3D_FRAME_STATS_COUNTER_RENDERED_LOWER) as usize;
    let rendered = GRAPHIC3D_FRAME_STATS_COUNTER_RENDERED_LOWER as usize
        ..=GRAPHIC3D_FRAME_STATS_COUNTER_RENDERED_UPPER as usize;
    let counters = &mut fields.counters_tmp.counters;
    for index in rendered.clone() {
        counters[index + shift] = counters[index];
    }
    for index in rendered {
        counters[index] = previous.counters[index];
    }
    fields.counters_tmp[Counter::EstimatedBytesGeom] = previous[Counter::EstimatedBytesGeom];
}

/// Stores the active frame at the last index of the history and resets it.
fn store_active(fields: &mut Graphic3dFrameStatsFields) {
    let index = usize::try_from(fields.last_frame_index).unwrap_or(0);
    let frame = (*fields.counters_tmp).clone();
    if let Some(slot) = fields.counters.get_mut(index) {
        *slot = frame;
    }
    fields.counters_tmp.reset();
}

/// The native `FrameEnd` body: the frame duration, the meters once the update interval is
/// over, the statistics of the view and the next data frame.
pub(crate) fn frame_end<S: Graphic3dFrameStats + ?Sized>(
    stats: &S,
    view: &Handle<dyn Graphic3dCView>,
    is_immediate_only: bool,
) {
    if skipped(view, is_immediate_only) {
        return;
    }
    let previous = {
        let mut fields = stats.frame_stats_base().fields.write();
        let time = fields.fps_timer.elapsed_time();
        fields.frame_duration = time - fields.frame_start_time;
        fields.fps_frame_count += 1;
        if let Some(view) = view.get() {
            fields.update_interval = f64::from(view.rendering_params().stats.stats_update_interval);
        }
        if time < fields.update_interval {
            let count = fields.fps_frame_count;
            fields.counters_tmp.flush_timers(count, false);
            return;
        }
        let previous = fields.last_data_frame().clone();
        if time > gp::resolution() {
            close_measurement(&mut fields, time, &previous, is_immediate_only);
        }
        if view.is_null() {
            store_active(&mut fields);
            return;
        }
        previous
    };
    stats.update_statistics(view, is_immediate_only);
    let mut fields = stats.frame_stats_base().fields.write();
    fields.last_frame_index += 1;
    if fields.last_frame_index > fields.counters.len() as i32 - 1 {
        fields.last_frame_index = 0;
    }
    if is_immediate_only {
        merge_immediate(&mut fields, &previous);
    }
    store_active(&mut fields);
}
