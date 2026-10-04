// Rust translation of FoundationClasses/TKernel/Standard/Standard_Condition.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Standard_Condition`: a manual-reset event shared between threads.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

/// A boolean flag threads set, reset and wait on, like a WinAPI event; `condition.set()`.
pub struct StandardCondition {
    flag: AtomicBool,
    waiter_mutex: Mutex<()>,
    signal: Condvar,
}

impl StandardCondition {
    /// Creates the event in the given state; `StandardCondition::new(false)`.
    pub fn new(is_set: bool) -> Self {
        Self {
            flag: AtomicBool::new(is_set),
            waiter_mutex: Mutex::new(()),
            signal: Condvar::new(),
        }
    }

    /// Signals the event and wakes every waiter; `condition.set()`.
    pub fn set(&self) {
        {
            let _guard = self.waiter_guard();
            self.flag.store(true, Ordering::SeqCst);
        }
        self.signal.notify_all();
    }

    /// Clears the signal; `condition.reset()`.
    pub fn reset(&self) {
        let _guard = self.waiter_guard();
        self.flag.store(false, Ordering::SeqCst);
    }

    /// Blocks until the event is signaled; `condition.wait()`.
    pub fn wait(&self) {
        let guard = self.waiter_guard();
        let _signaled = self
            .signal
            .wait_while(guard, |()| !self.check())
            .unwrap_or_else(PoisonError::into_inner);
    }

    /// Waits at most `milliseconds` for the signal and reports it; `condition.wait_timeout(10)`.
    /// A zero or negative limit only tests the flag, as `wait_for` does with such durations.
    pub fn wait_timeout(&self, milliseconds: i32) -> bool {
        let limit = Duration::from_millis(u64::try_from(milliseconds).unwrap_or(0));
        self.wait_for(limit)
    }

    /// Shares the event timeout with the OSD pool; `condition.wait_for(Duration::ZERO)`.
    pub(crate) fn wait_for(&self, limit: Duration) -> bool {
        let guard = self.waiter_guard();
        let (_guard, _) = self
            .signal
            .wait_timeout_while(guard, limit, |()| !self.check())
            .unwrap_or_else(PoisonError::into_inner);
        self.check()
    }

    /// Reports the flag without waiting; `condition.check()`.
    pub fn check(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    /// Clears the flag and reports whether it was signaled; `condition.check_reset()`.
    pub fn check_reset(&self) -> bool {
        let _guard = self.waiter_guard();
        self.flag.swap(false, Ordering::SeqCst)
    }

    fn waiter_guard(&self) -> MutexGuard<'_, ()> {
        self.waiter_mutex
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for StandardCondition {
    /// The header's default argument: not signaled; `StandardCondition::default().check() == false`.
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests;
