//! `OSD::SecSleep` and `OSD::MilliSecSleep`, with the native integer arithmetic.
use std::time::Duration;

/// Blocks the calling thread; implement a recording fake for tests.
pub trait Sleeper {
    /// Sleeps for `duration`; `sleeper.sleep(Duration::from_millis(5))`.
    fn sleep(&self, duration: Duration);
}

/// `std::thread::sleep` (usleep on Unix and Sleep on Windows natively); `NativeSleeper`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeSleeper;
impl Sleeper for NativeSleeper {
    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// The duration `OSD::SecSleep(seconds)` requests from the host;
/// `sec_sleep_duration(2) == Duration::from_secs(2)`.
///
/// Unix computes `seconds * 1000 * 1000` in `int` (wrapping past 2147 s, the
/// two's-complement result of the undefined overflow) and passes it to `usleep` as an
/// unsigned `useconds_t`, so negative values sleep ~71 minutes. Windows computes
/// `seconds * 1000` milliseconds as a `DWORD`.
pub fn sec_sleep_duration(seconds: i32) -> Duration {
    if cfg!(windows) {
        return milli_sec_sleep_duration(seconds.wrapping_mul(1000));
    }
    Duration::from_micros(u64::from(
        seconds.wrapping_mul(1000).wrapping_mul(1000) as u32
    ))
}

/// The duration `OSD::MilliSecSleep(milliseconds)` requests; `milli_sec_sleep_duration(5)`.
///
/// Unix: `milliseconds * 1000` microseconds as unsigned `useconds_t` (same wrapping).
/// Windows: the milliseconds as a `DWORD`; `INFINITE` (-1) is kept as its 49-day value
/// because Rust has no infinite sleep.
pub fn milli_sec_sleep_duration(milliseconds: i32) -> Duration {
    if cfg!(windows) {
        return Duration::from_millis(u64::from(milliseconds as u32));
    }
    Duration::from_micros(u64::from(milliseconds.wrapping_mul(1000) as u32))
}
