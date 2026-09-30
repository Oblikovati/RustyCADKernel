// Rust translation of FoundationClasses/TKernel/Standard/Standard_Mutex.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Standard_Mutex` (deprecated upstream in favour of `std::mutex`) and its `Sentry`.
//!
//! The header describes a non-recursive mutex, but the implementation creates a
//! `PTHREAD_MUTEX_RECURSIVE` mutex (and a Windows critical section, also recursive), so
//! the owning thread may lock it again and must unlock it as many times. This port keeps
//! that behavior with an owner and a depth guarded by `std::sync`.
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::{self, ThreadId};

#[derive(Default)]
struct Ownership {
    owner: Option<ThreadId>,
    depth: usize,
}

/// A recursive lock with explicit `lock`/`unlock`; `mutex.lock(); mutex.unlock();`.
#[derive(Default)]
pub struct StandardMutex {
    ownership: Mutex<Ownership>,
    released: Condvar,
}

impl StandardMutex {
    /// Creates an unlocked mutex; `StandardMutex::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Waits until no other thread holds the mutex, then takes it (again); `mutex.lock()`.
    pub fn lock(&self) {
        let me = thread::current().id();
        let mut ownership = self.ownership();
        while ownership.owner.is_some_and(|owner| owner != me) {
            ownership = self
                .released
                .wait(ownership)
                .unwrap_or_else(PoisonError::into_inner);
        }
        ownership.owner = Some(me);
        ownership.depth += 1;
    }

    /// Takes the mutex unless another thread holds it; `mutex.try_lock()`.
    /// The owner re-locks successfully, as `pthread_mutex_trylock` does on a recursive mutex.
    pub fn try_lock(&self) -> bool {
        let me = thread::current().id();
        let mut ownership = self.ownership();
        if ownership.owner.is_some_and(|owner| owner != me) {
            return false;
        }
        ownership.owner = Some(me);
        ownership.depth += 1;
        true
    }

    /// Releases one level of the calling thread's hold; `mutex.unlock()`.
    /// A thread that does not own the mutex changes nothing (pthread reports EPERM).
    pub fn unlock(&self) {
        let mut ownership = self.ownership();
        if ownership.owner != Some(thread::current().id()) {
            return;
        }
        ownership.depth -= 1;
        if ownership.depth == 0 {
            ownership.owner = None;
            self.released.notify_one();
        }
    }

    fn ownership(&self) -> MutexGuard<'_, Ownership> {
        self.ownership
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// Locks a mutex for its lifetime; `let _sentry = Sentry::new(&mutex);`.
///
/// Dropping the sentry unlocks, also while a panic unwinds, which is what the native
/// error-handler callback (`RegisterCallback`/`DestroyCallback`) provides on OCCT signals.
pub struct Sentry<'a> {
    mutex: Option<&'a StandardMutex>,
}

impl<'a> Sentry<'a> {
    /// Locks `mutex` at once; `Sentry::new(&mutex)`.
    pub fn new(mutex: &'a StandardMutex) -> Self {
        Self::from_option(Some(mutex))
    }

    /// The pointer overload: locks only a non-null mutex; `Sentry::from_option(None)`.
    pub fn from_option(mutex: Option<&'a StandardMutex>) -> Self {
        if let Some(mutex) = mutex {
            mutex.lock();
        }
        Self { mutex }
    }
}

impl Drop for Sentry<'_> {
    fn drop(&mut self) {
        if let Some(mutex) = self.mutex {
            mutex.unlock();
        }
    }
}
