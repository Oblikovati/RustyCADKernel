//! `Graphic3d_FrameStatsData` and `Graphic3d_FrameStatsDataTmp`: one frame of rendering
//! statistics, and the frame being filled between `FrameStart` and `FrameEnd`.
use super::{
    GRAPHIC3D_FRAME_STATS_COUNTER_NB, GRAPHIC3D_FRAME_STATS_TIMER_NB, Graphic3dFrameStatsCounter,
    Graphic3dFrameStatsTimer,
};
use rustycad_tkernel::osd::Timer;
use rustycad_tkernel::standard::real::real_last;
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[cfg(test)]
#[path = "frame_stats_assignment_tests.rs"]
mod assignment_tests;

/// The native `std::max`: `a < b ? b : a`.
pub(crate) fn native_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// The native `std::min`: `b < a ? b : a`.
pub(crate) fn native_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// One frame of statistics; `Graphic3dFrameStatsData::default().frame_rate() == -1.0`.
#[derive(Clone, Debug, PartialEq)]
pub struct Graphic3dFrameStatsData {
    /// `myCounters`.
    pub(crate) counters: Vec<usize>,
    /// `myTimers`.
    pub(crate) timers: Vec<f64>,
    /// `myTimersMin`: the minimal timer values.
    pub(crate) timers_min: Vec<f64>,
    /// `myTimersMax`: the maximal timer values.
    pub(crate) timers_max: Vec<f64>,
    /// `myFps`: frames per second of elapsed time.
    pub(crate) fps: f64,
    /// `myFpsCpu`: frames per second of CPU time.
    pub(crate) fps_cpu: f64,
    /// `myFpsImmediate`.
    pub(crate) fps_immediate: f64,
    /// `myFpsCpuImmediate`.
    pub(crate) fps_cpu_immediate: f64,
}

impl Default for Graphic3dFrameStatsData {
    fn default() -> Self {
        let timer_nb = GRAPHIC3D_FRAME_STATS_TIMER_NB as usize;
        Self {
            counters: vec![0; GRAPHIC3D_FRAME_STATS_COUNTER_NB as usize],
            timers: vec![0.0; timer_nb],
            timers_min: vec![real_last(); timer_nb],
            timers_max: vec![0.0; timer_nb],
            fps: -1.0,
            fps_cpu: -1.0,
            fps_immediate: -1.0,
            fps_cpu_immediate: -1.0,
        }
    }
}

impl Graphic3dFrameStatsData {
    /// `FrameRate`: frames per second of elapsed time; `data.frame_rate()`.
    pub fn frame_rate(&self) -> f64 {
        self.fps
    }
    /// `FrameRateCpu`: frames per second of CPU time; `data.frame_rate_cpu()`.
    pub fn frame_rate_cpu(&self) -> f64 {
        self.fps_cpu
    }
    /// `ImmediateFrameRate`; `data.immediate_frame_rate()`.
    pub fn immediate_frame_rate(&self) -> f64 {
        self.fps_immediate
    }
    /// `ImmediateFrameRateCpu`; `data.immediate_frame_rate_cpu()`.
    pub fn immediate_frame_rate_cpu(&self) -> f64 {
        self.fps_cpu_immediate
    }
    /// `CounterValue`; `data.counter_value(Graphic3dFrameStatsCounter::NbLayers)`.
    pub fn counter_value(&self, counter: Graphic3dFrameStatsCounter) -> usize {
        self.counters[counter as usize]
    }
    /// `TimerValue`; `data.timer_value(Graphic3dFrameStatsTimer::CpuFrame)`.
    pub fn timer_value(&self, timer: Graphic3dFrameStatsTimer) -> f64 {
        self.timers[timer as usize]
    }
    /// `Reset`: the rates to -1, the counters and timers to 0, the minima to `RealLast`;
    /// `data.reset()`.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    /// `FillMax`: keeps the larger rates and counters, the extreme timers, and the timers
    /// set to the maxima; `max_data.fill_max(&frame)`.
    pub fn fill_max(&mut self, other: &Self) {
        self.fps = native_max(self.fps, other.fps);
        self.fps_cpu = native_max(self.fps_cpu, other.fps_cpu);
        self.fps_immediate = native_max(self.fps_immediate, other.fps_immediate);
        self.fps_cpu_immediate = native_max(self.fps_cpu_immediate, other.fps_cpu_immediate);
        for (mine, theirs) in self.counters.iter_mut().zip(&other.counters) {
            *mine = if *mine > *theirs { *mine } else { *theirs };
        }
        for index in 0..self.timers.len() {
            self.timers_max[index] = native_max(self.timers_max[index], other.timers_max[index]);
            self.timers_min[index] = native_min(self.timers_min[index], other.timers_min[index]);
            self.timers[index] = self.timers_max[index];
        }
    }
}

impl Index<Graphic3dFrameStatsCounter> for Graphic3dFrameStatsData {
    type Output = usize;
    /// `operator[](Graphic3d_FrameStatsCounter)`.
    fn index(&self, counter: Graphic3dFrameStatsCounter) -> &usize {
        &self.counters[counter as usize]
    }
}

impl Index<Graphic3dFrameStatsTimer> for Graphic3dFrameStatsData {
    type Output = f64;
    /// `operator[](Graphic3d_FrameStatsTimer)`.
    fn index(&self, timer: Graphic3dFrameStatsTimer) -> &f64 {
        &self.timers[timer as usize]
    }
}

/// The frame being filled; `Graphic3dFrameStatsDataTmp::default().change_frame_rate()`.
#[derive(Clone)]
pub struct Graphic3dFrameStatsDataTmp {
    data: Graphic3dFrameStatsData,
    /// `myOsdTimers`: precise timers for time measurements.
    osd_timers: Vec<Timer>,
    /// `myTimersPrev`: the previous timer values.
    timers_prev: Vec<f64>,
}

impl Default for Graphic3dFrameStatsDataTmp {
    fn default() -> Self {
        let timer_nb = GRAPHIC3D_FRAME_STATS_TIMER_NB as usize;
        Self {
            data: Graphic3dFrameStatsData::default(),
            osd_timers: (0..timer_nb).map(|_| Timer::new(true)).collect(),
            timers_prev: vec![0.0; timer_nb],
        }
    }
}

impl Graphic3dFrameStatsDataTmp {
    /// `FlushTimers`: folds the time of the last frame into the extremes and, at the end of
    /// a measurement, averages the timers over `nb_frames`; `tmp.flush_timers(10, true)`.
    pub fn flush_timers(&mut self, nb_frames: usize, is_final: bool) {
        let data = &mut self.data;
        for index in 0..data.timers.len() {
            let frame_time = data.timers[index] - self.timers_prev[index];
            data.timers_max[index] = native_max(data.timers_max[index], frame_time);
            data.timers_min[index] = native_min(data.timers_min[index], frame_time);
            self.timers_prev[index] = data.timers[index];
        }
        if is_final {
            let nb_frames = nb_frames as f64;
            data.timers.iter_mut().for_each(|timer| *timer /= nb_frames);
        }
    }
    /// `Reset`: the frame data, the previous timers and the OSD timers; `tmp.reset()`.
    pub fn reset(&mut self) {
        self.data.reset();
        self.timers_prev.iter_mut().for_each(|timer| *timer = 0.0);
        self.osd_timers.iter_mut().for_each(Timer::reset);
    }
    /// `operator=(const Graphic3d_FrameStatsData&)`: copies the frame data only, not the
    /// OSD or previous timers; `tmp.assign_data(&frame)`.
    pub fn assign_data(&mut self, other: &Graphic3dFrameStatsData) {
        self.data = other.clone();
    }
    /// `ChangeFrameRate`; `*tmp.change_frame_rate() = 60.0`.
    pub fn change_frame_rate(&mut self) -> &mut f64 {
        &mut self.data.fps
    }
    /// `ChangeFrameRateCpu`; `*tmp.change_frame_rate_cpu() = 90.0`.
    pub fn change_frame_rate_cpu(&mut self) -> &mut f64 {
        &mut self.data.fps_cpu
    }
    /// `ChangeImmediateFrameRate`; `*tmp.change_immediate_frame_rate() = -1.0`.
    pub fn change_immediate_frame_rate(&mut self) -> &mut f64 {
        &mut self.data.fps_immediate
    }
    /// `ChangeImmediateFrameRateCpu`; `*tmp.change_immediate_frame_rate_cpu() = -1.0`.
    pub fn change_immediate_frame_rate_cpu(&mut self) -> &mut f64 {
        &mut self.data.fps_cpu_immediate
    }
    /// `ChangeTimer`: the OSD timer of `timer`; `tmp.change_timer(Graphic3dFrameStatsTimer::CpuCulling).start()`.
    pub fn change_timer(&mut self, timer: Graphic3dFrameStatsTimer) -> &mut Timer {
        &mut self.osd_timers[timer as usize]
    }
    /// `ChangeCounterValue`; `*tmp.change_counter_value(Graphic3dFrameStatsCounter::NbStructs) += 1`.
    pub fn change_counter_value(&mut self, counter: Graphic3dFrameStatsCounter) -> &mut usize {
        &mut self.data.counters[counter as usize]
    }
    /// `ChangeTimerValue`; `*tmp.change_timer_value(Graphic3dFrameStatsTimer::CpuFrame) += 0.01`.
    pub fn change_timer_value(&mut self, timer: Graphic3dFrameStatsTimer) -> &mut f64 {
        &mut self.data.timers[timer as usize]
    }
}

impl Deref for Graphic3dFrameStatsDataTmp {
    type Target = Graphic3dFrameStatsData;
    /// The inherited `Graphic3d_FrameStatsData` members.
    fn deref(&self) -> &Graphic3dFrameStatsData {
        &self.data
    }
}

impl DerefMut for Graphic3dFrameStatsDataTmp {
    fn deref_mut(&mut self) -> &mut Graphic3dFrameStatsData {
        &mut self.data
    }
}

impl Index<Graphic3dFrameStatsCounter> for Graphic3dFrameStatsDataTmp {
    type Output = usize;
    fn index(&self, counter: Graphic3dFrameStatsCounter) -> &usize {
        &self.data[counter]
    }
}

impl IndexMut<Graphic3dFrameStatsCounter> for Graphic3dFrameStatsDataTmp {
    /// `operator[](Graphic3d_FrameStatsCounter)`: modifies a counter.
    fn index_mut(&mut self, counter: Graphic3dFrameStatsCounter) -> &mut usize {
        self.change_counter_value(counter)
    }
}

impl Index<Graphic3dFrameStatsTimer> for Graphic3dFrameStatsDataTmp {
    type Output = f64;
    fn index(&self, timer: Graphic3dFrameStatsTimer) -> &f64 {
        &self.data[timer]
    }
}

impl IndexMut<Graphic3dFrameStatsTimer> for Graphic3dFrameStatsDataTmp {
    /// `operator[](Graphic3d_FrameStatsTimer)`: modifies a timer.
    fn index_mut(&mut self, timer: Graphic3dFrameStatsTimer) -> &mut f64 {
        self.change_timer_value(timer)
    }
}
