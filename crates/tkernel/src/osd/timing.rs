//! Timing, signals and package statics: `OSD_Timer`, `OSD_Chronometer` and `OSD`.
mod chronometer;
mod clocks;
mod cpu_native;
mod osd_statics;
mod real_text;
mod signal_native;
mod signals;
mod sleep;
mod timer;
mod wall_native;

pub use chronometer::Chronometer;
pub use clocks::{CpuTimes, NativeCpuTimes, NativeWallClock, WallClock};
pub use osd_statics::Osd;
pub use real_text::{c_string_to_real, real_to_c_string};
pub use signals::{NativeSignalHost, SignalHost, SignalState};
pub use sleep::{NativeSleeper, Sleeper, milli_sec_sleep_duration, sec_sleep_duration};
pub use timer::Timer;
