//! Graphic3d_FrameStats and Graphic3d_FrameStatsData, the twin of
//! tools/oracle/tkservice_probe_stats.cxx.
use super::common::{Reader, end, flag, int, real, text};
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_FRAME_STATS_COUNTER_NB, GRAPHIC3D_FRAME_STATS_TIMER_NB, Graphic3dCView,
    Graphic3dFrameStats, Graphic3dFrameStatsBase, Graphic3dFrameStatsCounter as Counter,
    Graphic3dFrameStatsData, Graphic3dFrameStatsDataTmp, Graphic3dFrameStatsDict,
    Graphic3dFrameStatsMembers, Graphic3dFrameStatsTimer as Timer,
    Graphic3dRenderingParamsPerfCounters,
};

/// The abstract statistics with an empty `updateStatistics`.
#[derive(Default)]
struct SvFrameStats {
    transient: StandardTransient,
    base: Graphic3dFrameStatsBase,
}
rustycad_tkernel::impl_transient!(SvFrameStats, "SvFrameStats", dyn Graphic3dFrameStats, transient,
    interfaces: [dyn Graphic3dFrameStats]);
impl Graphic3dFrameStats for SvFrameStats {
    fn frame_stats_base(&self) -> &Graphic3dFrameStatsBase {
        &self.base
    }
    fn update_statistics(&self, _view: &Handle<dyn Graphic3dCView>, _is_immediate_only: bool) {}
}

const COUNTERS: [Counter; GRAPHIC3D_FRAME_STATS_COUNTER_NB as usize] = [
    Counter::NbLayers,
    Counter::NbStructs,
    Counter::EstimatedBytesGeom,
    Counter::EstimatedBytesFbos,
    Counter::EstimatedBytesTextures,
    Counter::NbLayersNotCulled,
    Counter::NbStructsNotCulled,
    Counter::NbGroupsNotCulled,
    Counter::NbElemsNotCulled,
    Counter::NbElemsFillNotCulled,
    Counter::NbElemsLineNotCulled,
    Counter::NbElemsPointNotCulled,
    Counter::NbElemsTextNotCulled,
    Counter::NbTrianglesNotCulled,
    Counter::NbLinesNotCulled,
    Counter::NbPointsNotCulled,
    Counter::NbLayersImmediate,
    Counter::NbStructsImmediate,
    Counter::NbGroupsImmediate,
    Counter::NbElemsImmediate,
    Counter::NbElemsFillImmediate,
    Counter::NbElemsLineImmediate,
    Counter::NbElemsPointImmediate,
    Counter::NbElemsTextImmediate,
    Counter::NbTrianglesImmediate,
    Counter::NbLinesImmediate,
    Counter::NbPointsImmediate,
];
const TIMERS: [Timer; GRAPHIC3D_FRAME_STATS_TIMER_NB as usize] = [
    Timer::ElapsedFrame,
    Timer::CpuFrame,
    Timer::CpuCulling,
    Timer::CpuPicking,
    Timer::CpuDynamics,
];

fn size(value: usize) {
    print!("{value} ");
}

/// Reads the 27 counters, the 5 timers and the 4 rates of a data frame.
fn fill(r: &mut Reader<'_>, tmp: &mut Graphic3dFrameStatsDataTmp) {
    COUNTERS.iter().for_each(|c| tmp[*c] = r.real() as usize);
    TIMERS.iter().for_each(|t| tmp[*t] = r.real());
    *tmp.change_frame_rate() = r.real();
    *tmp.change_frame_rate_cpu() = r.real();
    *tmp.change_immediate_frame_rate() = r.real();
    *tmp.change_immediate_frame_rate_cpu() = r.real();
}

fn print_data(d: &Graphic3dFrameStatsData) {
    COUNTERS.iter().for_each(|c| size(d[*c]));
    TIMERS.iter().for_each(|t| real(d[*t]));
    [
        d.frame_rate(),
        d.frame_rate_cpu(),
        d.immediate_frame_rate(),
        d.immediate_frame_rate_cpu(),
    ]
    .into_iter()
    .for_each(real);
}

fn print_dict(stats: &SvFrameStats, flags: Graphic3dRenderingParamsPerfCounters) {
    let mut dict = Graphic3dFrameStatsDict::default();
    dict.add(AsciiString::from("Other"), AsciiString::from("o"));
    dict.add(AsciiString::from("FPS"), AsciiString::from("x"));
    stats.format_stats_dict(&mut dict, flags);
    int(dict.extent() as i32);
    for (key, value) in dict.items() {
        text(key.as_bytes());
        text(value.as_bytes());
    }
}

fn print_members(stats: &SvFrameStats) {
    flag(stats.has_culled_layers());
    flag(stats.has_culled_structs());
    flag(stats.is_long_line_format());
    size(stats.counter_value(Counter::NbLayers));
    real(stats.timer_value(Timer::CpuFrame));
    real(stats.frame_rate());
    real(stats.frame_rate_cpu());
    int(stats.last_data_frame_index());
    real(stats.update_interval());
    real(stats.frame_duration());
    int(stats.data_frames().len() as i32);
}

/// FlushTimers, FillMax and Reset on the active frame and a copy of the maxima.
fn data_ops(tmp: &mut Graphic3dFrameStatsDataTmp, max: &Graphic3dFrameStatsData, nb: usize) {
    tmp.flush_timers(nb, false);
    print_data(tmp);
    tmp.flush_timers(nb, true);
    print_data(tmp);
    let mut filled = Graphic3dFrameStatsData::default();
    filled.fill_max(tmp);
    print_data(&filled);
    filled = max.clone();
    filled.fill_max(tmp);
    print_data(&filled);
    tmp.assign_data(max);
    print_data(tmp);
    tmp.reset();
    print_data(tmp);
    filled.reset();
    print_data(&filled);
}

/// `sv_stats`: the text and dictionary statistics of a frame, then the data frame operations.
pub fn stats_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let stats = Handle::new(SvFrameStats::default());
    stats.set_long_line_format(r.flag());
    let flags = Graphic3dRenderingParamsPerfCounters(r.integer());
    stats.active_data_frame(&mut |tmp| fill(&mut r, tmp));
    let active: Graphic3dFrameStatsData = (*stats.base.fields.read().counters_tmp).clone();
    stats.change_data_frames(&mut |frames| frames[0] = active.clone());
    let mut max_tmp = Graphic3dFrameStatsDataTmp::default();
    fill(&mut r, &mut max_tmp);
    stats.base.fields.write().counters_max = (*max_tmp).clone();
    let nb = r.real() as usize;
    text(stats.format_stats(flags).as_bytes());
    print_dict(&stats, flags);
    print_members(&stats);
    let max = stats.base.fields.read().counters_max.clone();
    stats.active_data_frame(&mut |tmp| data_ops(tmp, &max, nb));
    end();
}
