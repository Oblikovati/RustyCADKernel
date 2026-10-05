//! `OSD::SecSleep` and `OSD::MilliSecSleep`, with the native integer arithmetic.
mod native;

use std::time::Duration;

/// A finite request or the native Windows `INFINITE` sentinel; `SleepRequest::Infinite`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SleepRequest {
    /// The requested finite delay.
    Finite(Duration),
    /// Do not resume from the sleep call.
    Infinite,
}

/// Blocks the calling thread; implement a recording fake for tests.
pub trait Sleeper {
    /// Sleeps for `duration`; `sleeper.sleep(Duration::from_millis(5))`.
    fn sleep(&self, duration: Duration);

    /// Preserves native seconds for specialized sleepers; `sleeper.sleep_seconds(2)`.
    fn sleep_seconds(&self, seconds: i32) {
        self.sleep(sec_sleep_duration(seconds));
    }

    /// Preserves native milliseconds for finite-only sleepers; `sleeper.sleep_milliseconds(5)`.
    fn sleep_milliseconds(&self, milliseconds: i32) {
        sleep_milliseconds_with_platform(self, milliseconds, cfg!(windows));
    }

    /// Accepts the native sentinel while retaining existing finite-only implementations;
    /// `sleeper.sleep_request(SleepRequest::Finite(Duration::from_millis(5)))`.
    fn sleep_request(&self, request: SleepRequest) {
        match request {
            SleepRequest::Finite(duration) => self.sleep(duration),
            SleepRequest::Infinite => loop {
                self.sleep(Duration::MAX);
            },
        }
    }
}

/// Generic duration requests use `std::thread::sleep`; OSD seconds requests call the host API.
/// `Infinite` parks and re-parks, `NativeSleeper`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeSleeper;
impl Sleeper for NativeSleeper {
    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }

    fn sleep_request(&self, request: SleepRequest) {
        match request {
            SleepRequest::Finite(duration) => self.sleep(duration),
            SleepRequest::Infinite => sleep_forever(),
        }
    }
    fn sleep_seconds(&self, seconds: i32) {
        native::sec_sleep(seconds);
    }
    fn sleep_milliseconds(&self, milliseconds: i32) {
        native::milli_sec_sleep(milliseconds);
    }
}

fn sleep_forever() {
    loop {
        std::thread::park();
    }
}

/// Converts seconds for an injected finite-only sleeper; `sec_sleep_duration(2) == Duration::from_secs(2)`.
///
/// Injected sleepers receive the native integer conversion. The default `Osd::sec_sleep`
/// calls `usleep`/`Sleep` directly so native EINTR and invalid-delay behavior is retained;
/// `Osd::sec_sleep_with(&fake, 2)` records this duration without blocking.
pub fn sec_sleep_duration(seconds: i32) -> Duration {
    if cfg!(windows) {
        return milli_sec_sleep_duration(seconds.wrapping_mul(1000));
    }
    Duration::from_micros(u64::from(
        seconds.wrapping_mul(1000).wrapping_mul(1000) as u32
    ))
}

/// Converts a millisecond value to a finite duration; `milli_sec_sleep_duration(5)`.
///
/// Unix: `milliseconds * 1000` microseconds as unsigned `useconds_t` (same wrapping).
/// Windows: the millisecond value as a `DWORD`. The Windows `INFINITE` sentinel is a
/// separate [`SleepRequest::Infinite`] in `Osd::milli_sec_sleep_with`.
pub fn milli_sec_sleep_duration(milliseconds: i32) -> Duration {
    milli_sec_sleep_duration_for(milliseconds, cfg!(windows))
}

/// Converts a platform's millisecond argument without losing Windows `INFINITE`;
/// `milli_sec_sleep_request(-1, true) == SleepRequest::Infinite`.
pub(crate) fn milli_sec_sleep_request(milliseconds: i32, is_windows: bool) -> SleepRequest {
    // OSD.cxx:91 passes -1 as DWORD INFINITE; a Duration would turn it into a finite 49-day delay.
    if is_windows && milliseconds == -1 {
        return SleepRequest::Infinite;
    }
    SleepRequest::Finite(milli_sec_sleep_duration_for(milliseconds, is_windows))
}

/// Routes milliseconds through an explicit platform policy so sentinel handling is testable.
pub(crate) fn sleep_milliseconds_with_platform(
    sleeper: &(impl Sleeper + ?Sized),
    milliseconds: i32,
    is_windows: bool,
) {
    sleeper.sleep_request(milli_sec_sleep_request(milliseconds, is_windows));
}

fn milli_sec_sleep_duration_for(milliseconds: i32, is_windows: bool) -> Duration {
    if is_windows {
        return Duration::from_millis(u64::from(milliseconds as u32));
    }
    Duration::from_micros(u64::from(milliseconds.wrapping_mul(1000) as u32))
}

#[cfg(test)]
mod tests {
    use super::{SleepRequest, Sleeper, sleep_milliseconds_with_platform};
    use std::{sync::Mutex, time::Duration};

    #[derive(Default)]
    struct RecordingSleeper(Mutex<Vec<SleepRequest>>);

    #[derive(Default)]
    struct FiniteOnlySleeper(Mutex<Vec<Duration>>);

    impl Sleeper for FiniteOnlySleeper {
        fn sleep(&self, duration: Duration) {
            self.0.lock().unwrap().push(duration);
        }
    }

    impl Sleeper for RecordingSleeper {
        fn sleep(&self, duration: Duration) {
            self.0.lock().unwrap().push(SleepRequest::Finite(duration));
        }

        fn sleep_request(&self, request: SleepRequest) {
            self.0.lock().unwrap().push(request);
        }
    }

    #[test]
    fn windows_infinite_milliseconds_stay_a_sentinel() {
        let sleeper = RecordingSleeper::default();
        sleep_milliseconds_with_platform(&sleeper, -1, true);
        assert_eq!(*sleeper.0.lock().unwrap(), [SleepRequest::Infinite]);
    }

    #[test]
    fn finite_platform_arithmetic_is_unchanged() {
        assert_eq!(
            super::milli_sec_sleep_request(-1, false),
            SleepRequest::Finite(Duration::from_micros(4_294_966_296))
        );
        assert_eq!(
            super::milli_sec_sleep_request(17, true),
            SleepRequest::Finite(Duration::from_millis(17))
        );
        assert_eq!(
            super::milli_sec_sleep_duration_for(-1, true),
            Duration::from_millis(u64::from(u32::MAX))
        );
    }

    #[test]
    fn finite_dispatch_uses_the_legacy_sleeper_duration_method() {
        let sleeper = FiniteOnlySleeper::default();
        sleep_milliseconds_with_platform(&sleeper, 17, true);
        assert_eq!(*sleeper.0.lock().unwrap(), [Duration::from_millis(17)]);
    }
}
