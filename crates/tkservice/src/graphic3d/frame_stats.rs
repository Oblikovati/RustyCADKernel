//! `Graphic3d_FrameStats`: the rendering statistics of a view. The class is abstract
//! (`updateStatistics` is pure virtual, `FormatStats` virtual), so it is a trait over an
//! embedded base, with the non-virtual members in [`Graphic3dFrameStatsMembers`].
use super::frame_stats_format::{format_stats_dict, format_stats_text};
use super::frame_stats_text::Graphic3dFrameStatsDict;
use super::{
    Graphic3dFrameStatsCounter, Graphic3dFrameStatsData, Graphic3dFrameStatsDataTmp,
    Graphic3dFrameStatsTimer, Graphic3dRenderingParamsPerfCounters,
};
use rustycad_tkernel::osd::Timer;
use rustycad_tkernel::standard::{SharedState, StandardTransient, Transient};
use rustycad_tkernel::tcollection::AsciiString;

/// The protected fields of the frame statistics.
#[derive(Clone)]
pub struct Graphic3dFrameStatsFields {
    /// `myFpsTimer`: the timer of the FPS measurements.
    pub fps_timer: Timer,
    /// `myFrameStartTime`: the time at the beginning of the frame redraw.
    pub frame_start_time: f64,
    /// `myFrameDuration`.
    pub frame_duration: f64,
    /// `myUpdateInterval`: the interval to update the meters.
    pub update_interval: f64,
    /// `myFpsFrameCount`: the frames within the measurement slice.
    pub fps_frame_count: usize,
    /// `myCounters`: the data frame history (native lower bound 0).
    pub counters: Vec<Graphic3dFrameStatsData>,
    /// `myCountersTmp`: the frame filled between `FrameStart` and `FrameEnd`.
    pub counters_tmp: Graphic3dFrameStatsDataTmp,
    /// `myCountersMax`: the absolute maxima of the history.
    pub counters_max: Graphic3dFrameStatsData,
    /// `myLastFrameIndex`.
    pub last_frame_index: i32,
    /// `myIsLongLineFormat`: longer lines over more lines.
    pub is_long_line_format: bool,
}

impl Default for Graphic3dFrameStatsFields {
    /// One data frame, an update interval of one second, the short line format.
    fn default() -> Self {
        Self {
            fps_timer: Timer::new(true),
            frame_start_time: 0.0,
            frame_duration: 0.0,
            update_interval: 1.0,
            fps_frame_count: 0,
            counters: vec![Graphic3dFrameStatsData::default()],
            counters_tmp: Graphic3dFrameStatsDataTmp::default(),
            counters_max: Graphic3dFrameStatsData::default(),
            last_frame_index: 0,
            is_long_line_format: false,
        }
    }
}

impl Graphic3dFrameStatsFields {
    /// `LastDataFrame`: the frame at `last_frame_index` (always in range: the native only
    /// moves the index inside the history); `fields.last_data_frame().frame_rate()`.
    pub fn last_data_frame(&self) -> &Graphic3dFrameStatsData {
        let index = usize::try_from(self.last_frame_index).unwrap_or(0);
        &self.counters[index.min(self.counters.len() - 1)]
    }
    /// `HasCulledLayers`: the rendered layer count differs from the layer count.
    pub fn has_culled_layers(&self) -> bool {
        let frame = self.last_data_frame();
        frame[Graphic3dFrameStatsCounter::NbLayersNotCulled]
            != frame[Graphic3dFrameStatsCounter::NbLayers]
    }
    /// `HasCulledStructs`: the rendered structure count differs from the structure count.
    pub fn has_culled_structs(&self) -> bool {
        let frame = self.last_data_frame();
        frame[Graphic3dFrameStatsCounter::NbStructsNotCulled]
            != frame[Graphic3dFrameStatsCounter::NbStructs]
    }
}

/// The embedded state of a frame statistics class.
#[derive(Default)]
pub struct Graphic3dFrameStatsBase {
    /// The fields, shared by every handle.
    pub fields: SharedState<Graphic3dFrameStatsFields>,
}

/// A frame statistics class; the formatters may be overridden.
pub trait Graphic3dFrameStats: Transient {
    /// The embedded base; `stats.frame_stats_base()`.
    fn frame_stats_base(&self) -> &Graphic3dFrameStatsBase;
    /// `FormatStats(PerfCounters)`: the statistics as text lines;
    /// `stats.format_stats(Graphic3dRenderingParamsPerfCounters::BASIC)`.
    fn format_stats(&self, flags: Graphic3dRenderingParamsPerfCounters) -> AsciiString {
        AsciiString::from(format_stats_text(&self.frame_stats_base().fields.read(), flags).as_str())
    }
    /// `FormatStats(dict, PerfCounters)`: the statistics as key/value pairs;
    /// `stats.format_stats_dict(&mut dict, Graphic3dRenderingParamsPerfCounters::ALL)`.
    fn format_stats_dict(
        &self,
        dict: &mut Graphic3dFrameStatsDict,
        flags: Graphic3dRenderingParamsPerfCounters,
    ) {
        format_stats_dict(&self.frame_stats_base().fields.read(), dict, flags);
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dFrameStats,
    "Graphic3d_FrameStats",
    StandardTransient
);

/// The non-virtual members of every frame statistics class.
pub trait Graphic3dFrameStatsMembers: Graphic3dFrameStats {
    /// `UpdateInterval`; `stats.update_interval()`.
    fn update_interval(&self) -> f64 {
        self.frame_stats_base().fields.read().update_interval
    }
    /// `SetUpdateInterval`; `stats.set_update_interval(0.5)`.
    fn set_update_interval(&self, interval: f64) {
        self.frame_stats_base().fields.write().update_interval = interval;
    }
    /// `IsLongLineFormat`; `stats.is_long_line_format()`.
    fn is_long_line_format(&self) -> bool {
        self.frame_stats_base().fields.read().is_long_line_format
    }
    /// `SetLongLineFormat`; `stats.set_long_line_format(true)`.
    fn set_long_line_format(&self, value: bool) {
        self.frame_stats_base().fields.write().is_long_line_format = value;
    }
    /// `FrameDuration`; `stats.frame_duration()`.
    fn frame_duration(&self) -> f64 {
        self.frame_stats_base().fields.read().frame_duration
    }
    /// `FrameRate` of the last data frame; `stats.frame_rate()`.
    fn frame_rate(&self) -> f64 {
        self.frame_stats_base()
            .fields
            .read()
            .last_data_frame()
            .frame_rate()
    }
    /// `FrameRateCpu` of the last data frame; `stats.frame_rate_cpu()`.
    fn frame_rate_cpu(&self) -> f64 {
        self.frame_stats_base()
            .fields
            .read()
            .last_data_frame()
            .frame_rate_cpu()
    }
    /// `CounterValue` of the last data frame; `stats.counter_value(Graphic3dFrameStatsCounter::NbLayers)`.
    fn counter_value(&self, counter: Graphic3dFrameStatsCounter) -> usize {
        self.frame_stats_base().fields.read().last_data_frame()[counter]
    }
    /// `TimerValue` of the last data frame; `stats.timer_value(Graphic3dFrameStatsTimer::CpuFrame)`.
    fn timer_value(&self, timer: Graphic3dFrameStatsTimer) -> f64 {
        self.frame_stats_base().fields.read().last_data_frame()[timer]
    }
    /// `HasCulledLayers`; `stats.has_culled_layers()`.
    fn has_culled_layers(&self) -> bool {
        self.frame_stats_base().fields.read().has_culled_layers()
    }
    /// `HasCulledStructs`; `stats.has_culled_structs()`.
    fn has_culled_structs(&self) -> bool {
        self.frame_stats_base().fields.read().has_culled_structs()
    }
    /// `LastDataFrame` (a copy); `stats.last_data_frame().frame_rate()`.
    fn last_data_frame(&self) -> Graphic3dFrameStatsData {
        self.frame_stats_base()
            .fields
            .read()
            .last_data_frame()
            .clone()
    }
    /// `LastDataFrameIndex`; `stats.last_data_frame_index()`.
    fn last_data_frame_index(&self) -> i32 {
        self.frame_stats_base().fields.read().last_frame_index
    }
    /// `DataFrames` (a copy); `stats.data_frames().len()`.
    fn data_frames(&self) -> Vec<Graphic3dFrameStatsData> {
        self.frame_stats_base().fields.read().counters.clone()
    }
    /// `ChangeDataFrames`: edits the history; `stats.change_data_frames(|frames| frames[0].reset())`.
    fn change_data_frames(&self, edit: &mut dyn FnMut(&mut Vec<Graphic3dFrameStatsData>)) {
        edit(&mut self.frame_stats_base().fields.write().counters);
    }
    /// `ActiveDataFrame`: edits the frame being filled;
    /// `stats.active_data_frame(&mut |tmp| *tmp.change_frame_rate() = 60.0)`.
    fn active_data_frame(&self, edit: &mut dyn FnMut(&mut Graphic3dFrameStatsDataTmp)) {
        edit(&mut self.frame_stats_base().fields.write().counters_tmp);
    }
    /// `ChangeCounter`: adds `delta` to a counter of the active frame (the native returns
    /// a reference that callers increment); `stats.change_counter(Graphic3dFrameStatsCounter::NbStructs, 1)`.
    fn change_counter(&self, counter: Graphic3dFrameStatsCounter, delta: usize) {
        let mut fields = self.frame_stats_base().fields.write();
        let slot = fields.counters_tmp.change_counter_value(counter);
        *slot = slot.wrapping_add(delta);
    }
    /// `ChangeTimer`: adds `delta` seconds to a timer of the active frame;
    /// `stats.change_timer(Graphic3dFrameStatsTimer::CpuPicking, 0.01)`.
    fn change_timer(&self, timer: Graphic3dFrameStatsTimer, delta: f64) {
        *self
            .frame_stats_base()
            .fields
            .write()
            .counters_tmp
            .change_timer_value(timer) += delta;
    }
}
impl<T: Graphic3dFrameStats + ?Sized> Graphic3dFrameStatsMembers for T {}
