// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_TimerSentry (.hxx, .lxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MoniTool_TimerSentry`: starts a timer when created and stops it when dropped.
use super::MoniToolTimer;
use rustycad_tkernel::standard::Handle;

/// `MoniTool_TimerSentry`; the destructor (`Drop`) stops the timer unless `Stop` did.
///
/// ```
/// use rustycad_tkxsbase::moni_tool::{MoniToolTimer, MoniToolTimerSentry};
/// use rustycad_tkernel::standard::Handle;
/// let timer = Handle::new(MoniToolTimer::new());
/// {
///     let _sentry = MoniToolTimerSentry::new(&timer);
///     assert_eq!(timer.is_running(), 1);
/// }
/// assert_eq!(timer.is_running(), 0);
/// ```
pub struct MoniToolTimerSentry {
    my_timer: Handle<MoniToolTimer>,
}

impl MoniToolTimerSentry {
    /// `MoniTool_TimerSentry(cname)`: starts the timer of the dictionary named `cname`.
    pub fn from_name(cname: &str) -> Self {
        Self::new(&MoniToolTimer::timer_named(cname))
    }
    /// `MoniTool_TimerSentry(timer)`: starts `timer` (a null handle starts nothing; the
    /// native constructor dereferences it).
    pub fn new(timer: &Handle<MoniToolTimer>) -> Self {
        if let Some(live) = timer.get() {
            live.start();
        }
        Self {
            my_timer: timer.clone(),
        }
    }
    /// `Timer`.
    pub fn timer(&self) -> Handle<MoniToolTimer> {
        self.my_timer.clone()
    }
    /// `Stop`: stops the timer and forgets it.
    pub fn stop(&mut self) {
        if let Some(live) = self.my_timer.get() {
            live.stop();
        }
        self.my_timer.nullify();
    }
}

impl Drop for MoniToolTimerSentry {
    /// `~MoniTool_TimerSentry`: stops the timer unless already stopped.
    fn drop(&mut self) {
        if let Some(live) = self.my_timer.get() {
            live.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_stop_forgets_the_timer() {
        let mut sentry = MoniToolTimerSentry::from_name("_test_sentry");
        let timer = sentry.timer();
        assert_eq!(timer.is_running(), 1);
        sentry.stop();
        assert!(sentry.timer().is_null());
        drop(sentry);
        assert_eq!(timer.is_running(), 0);
        assert_eq!(timer.count(), 1);
    }
}
