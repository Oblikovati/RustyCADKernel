//! The `OSD` class: package statics on the native hosts and the global signal state.
use super::{
    NativeSignalHost, NativeSleeper, SignalState, Sleeper, real_text, sec_sleep_duration,
    sleep_milliseconds_with_platform,
};
use crate::{osd::SignalMode, standard::Failure};

/// `OSD`: operating-system dependent statics; `Osd::real_to_c_string(1.5) == "1.5"`.
///
/// The `_with` variants of the host-dependent members take injected hosts for tests.
pub struct Osd;
impl Osd {
    /// `SetSignal(OSD_SignalMode, bool)` on the native host (see [`SignalState::set_signal`]
    /// for what is ported); `Osd::set_signal(SignalMode::Set, false)`.
    pub fn set_signal(mode: SignalMode, floating: bool) {
        SignalState::global().set_signal(&NativeSignalHost, mode, floating);
    }
    /// `SetSignal(bool)`: `SetSignal(OSD_SignalMode_Set, floating)`; `Osd::set_signal_default(true)`.
    pub fn set_signal_default(floating: bool) {
        Self::set_signal(SignalMode::Set, floating);
    }
    /// `SetThreadLocalSignal(OSD_SignalMode, bool)`; `Osd::set_thread_local_signal(SignalMode::Set, false)`.
    pub fn set_thread_local_signal(mode: SignalMode, floating: bool) {
        SignalState::global().set_thread_local_signal(&NativeSignalHost, mode, floating);
    }
    /// `SetFloatingSignal(bool)`: the calling thread's FP traps; `Osd::set_floating_signal(false)`.
    pub fn set_floating_signal(floating: bool) {
        use super::SignalHost;
        NativeSignalHost.set_floating_signal(floating);
    }
    /// `SignalMode()`: the mode of the last `set_signal`, `AsIs` by default; `Osd::signal_mode()`.
    pub fn signal_mode() -> SignalMode {
        SignalState::global().signal_mode()
    }
    /// `ToCatchFloatingSignals()`; `Osd::to_catch_floating_signals()`.
    pub fn to_catch_floating_signals() -> bool {
        use super::SignalHost;
        NativeSignalHost.to_catch_floating_signals()
    }
    /// `SecSleep(int)` (see [`sec_sleep_duration`]); `Osd::sec_sleep(1)`.
    pub fn sec_sleep(seconds: i32) {
        Self::sec_sleep_with(&NativeSleeper, seconds);
    }
    /// `SecSleep` on an injected sleeper; `Osd::sec_sleep_with(&sleeper, 1)`.
    pub fn sec_sleep_with(sleeper: &dyn Sleeper, seconds: i32) {
        sleeper.sleep(sec_sleep_duration(seconds));
    }
    /// `MilliSecSleep(int)` (see [`milli_sec_sleep_duration`]); `Osd::milli_sec_sleep(5)`.
    pub fn milli_sec_sleep(milliseconds: i32) {
        Self::milli_sec_sleep_with(&NativeSleeper, milliseconds);
    }
    /// `MilliSecSleep` on an injected sleeper; `Osd::milli_sec_sleep_with(&sleeper, 5)`.
    pub fn milli_sec_sleep_with(sleeper: &dyn Sleeper, milliseconds: i32) {
        sleep_milliseconds_with_platform(sleeper, milliseconds, cfg!(windows));
    }
    /// `RealToCString(double, Standard_PCharacter&)`: the text written into the caller's
    /// buffer (the native `true` result is implied); `Osd::real_to_c_string(1e5) == "1e+05"`.
    pub fn real_to_c_string(value: f64) -> String {
        real_text::real_to_c_string(value)
    }
    /// `CStringToReal(const char*, double&)`: (whole text consumed, value);
    /// `Osd::c_string_to_real("1.5") == (true, 1.5)`.
    pub fn c_string_to_real(text: &str) -> (bool, f64) {
        real_text::c_string_to_real(text)
    }
    /// `ControlBreak()`: raises `OSD_Exception_CTRL_BREAK` once after a Ctrl-C/Ctrl-Break
    /// caught by the handler `set_signal` installed; `Osd::control_break()?`.
    pub fn control_break() -> Result<(), Failure> {
        SignalState::global().control_break()
    }
    /// `SignalStackTraceLength()`: 0 by default; `Osd::signal_stack_trace_length()`.
    pub fn signal_stack_trace_length() -> i32 {
        SignalState::global().signal_stack_trace_length()
    }
    /// `SetSignalStackTraceLength(int)`; `Osd::set_signal_stack_trace_length(10)`.
    pub fn set_signal_stack_trace_length(length: i32) {
        SignalState::global().set_signal_stack_trace_length(length);
    }
}
