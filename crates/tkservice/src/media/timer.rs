//! `Media_Timer`: the playback timer, an OSD timer scaled by a playback speed from an offset.
use rustycad_tkernel::osd::Timer;
use rustycad_tkernel::standard::{SharedState, StandardTransient};

/// The fields of a playback timer.
pub struct MediaTimerFields {
    /// `myTimer`.
    pub timer: Timer,
    /// `myTimerFrom`: the time at the last pause or seek.
    pub timer_from: f64,
    /// `myTimerSpeed`: the playback speed.
    pub timer_speed: f64,
}

/// A playback timer, stopped at 0 with speed 1; `MediaTimer::default().elapsed_time() == 0.0`.
pub struct MediaTimer {
    transient: StandardTransient,
    /// The fields, shared by every handle.
    pub fields: SharedState<MediaTimerFields>,
}
rustycad_tkernel::impl_transient!(MediaTimer, "Media_Timer", StandardTransient, transient);

impl Default for MediaTimer {
    fn default() -> Self {
        Self::with_timer(Timer::new(false))
    }
}

impl MediaTimer {
    /// A stopped timer over `timer` (the clocks are injectable); `MediaTimer::with_timer(Timer::new(false))`.
    pub fn with_timer(timer: Timer) -> Self {
        let fields = MediaTimerFields {
            timer,
            timer_from: 0.0,
            timer_speed: 1.0,
        };
        Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
    /// `ElapsedTime`: the offset plus the running time times the speed; `timer.elapsed_time()`.
    pub fn elapsed_time(&self) -> f64 {
        let fields = self.fields.read();
        fields.timer_from + fields.timer.elapsed_time() * fields.timer_speed
    }
    /// `PlaybackSpeed`; `timer.playback_speed()`.
    pub fn playback_speed(&self) -> f64 {
        self.fields.read().timer_speed
    }
    /// `SetPlaybackSpeed`: a running timer keeps its time and continues at the new speed;
    /// `timer.set_playback_speed(2.0)`.
    pub fn set_playback_speed(&self, speed: f64) {
        let mut fields = self.fields.write();
        if !fields.timer.is_started() {
            fields.timer_speed = speed;
            return;
        }
        fold_running_time(&mut fields);
        fields.timer_speed = speed;
        fields.timer.start();
    }
    /// `IsStarted`; `timer.is_started()`.
    pub fn is_started(&self) -> bool {
        self.fields.read().timer.is_started()
    }
    /// `Start`; `timer.start()`.
    pub fn start(&self) {
        self.fields.write().timer.start();
    }
    /// `Pause`: keeps the elapsed time and stops; `timer.pause()`.
    pub fn pause(&self) {
        fold_running_time(&mut self.fields.write());
    }
    /// `Stop`: back to 0; `timer.stop()`.
    pub fn stop(&self) {
        let mut fields = self.fields.write();
        fields.timer.stop();
        fields.timer.reset();
        fields.timer_from = 0.0;
    }
    /// `Seek`: restarts the time at `time`, running again when it was; `timer.seek(10.0)`.
    pub fn seek(&self, time: f64) {
        let mut fields = self.fields.write();
        let is_started = fields.timer.is_started();
        fields.timer.stop();
        fields.timer.reset();
        fields.timer_from = time;
        if is_started {
            fields.timer.start();
        }
    }
}

/// Stops the OSD timer, adds its scaled time to the offset and resets it.
fn fold_running_time(fields: &mut MediaTimerFields) {
    fields.timer.stop();
    fields.timer_from += fields.timer.elapsed_time() * fields.timer_speed;
    fields.timer.reset();
}
