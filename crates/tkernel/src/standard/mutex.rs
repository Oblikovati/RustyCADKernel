// Rust translation of FoundationClasses/TKernel/Standard/Standard_Mutex.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Standard_Mutex` (deprecated upstream in favour of `std::mutex`) and its `Sentry`.
//!
//! The header describes a non-recursive mutex, but the implementation creates a
//! `PTHREAD_MUTEX_RECURSIVE` mutex (and a Windows critical section, also recursive), so
//! the owning thread may lock it again and must unlock it as many times. This port keeps
//! that behavior with an owner and a depth guarded by `std::sync`. A sentry also registers
//! an error-handler callback, matching the native unlock when a signal handler is destroyed.
use super::error_handler::ErrorHandlerCallback;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::{self, ThreadId};

#[derive(Default)]
struct Ownership {
    owner: Option<ThreadId>,
    depth: usize,
}

#[derive(Default)]
struct MutexState {
    ownership: Mutex<Ownership>,
    released: Condvar,
}

impl MutexState {
    fn lock(&self) {
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

    fn try_lock(&self) -> bool {
        let me = thread::current().id();
        let mut ownership = self.ownership();
        if ownership.owner.is_some_and(|owner| owner != me) {
            return false;
        }
        ownership.owner = Some(me);
        ownership.depth += 1;
        true
    }

    fn unlock(&self) {
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

/// A recursive lock with explicit `lock`/`unlock`; `mutex.lock(); mutex.unlock();`.
#[derive(Default)]
pub struct StandardMutex {
    state: Arc<MutexState>,
}

impl StandardMutex {
    /// Creates an unlocked mutex; `StandardMutex::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Waits until no other thread holds the mutex, then takes it (again); `mutex.lock()`.
    pub fn lock(&self) {
        self.state.lock();
    }

    /// Takes the mutex unless another thread holds it; `mutex.try_lock()`.
    /// The owner re-locks successfully, as `pthread_mutex_trylock` does on a recursive mutex.
    pub fn try_lock(&self) -> bool {
        self.state.try_lock()
    }

    /// Releases one level of the calling thread's hold; `mutex.unlock()`.
    /// A thread that does not own the mutex changes nothing (pthread reports EPERM).
    pub fn unlock(&self) {
        self.state.unlock();
    }
}

/// Locks a mutex for its lifetime; `let _sentry = Sentry::new(&mutex);`.
///
/// The native sentry also registers `DestroyCallback` with the active error handler. If that
/// handler is destroyed by OCCT signal conversion, the callback releases this lock before
/// ordinary stack cleanup can run.
pub struct Sentry<'a> {
    mutex: Option<&'a StandardMutex>,
    callback: Option<ErrorHandlerCallback>,
    lock_held: Option<Arc<AtomicBool>>,
}

impl<'a> Sentry<'a> {
    /// Locks `mutex` at once; `Sentry::new(&mutex)`.
    pub fn new(mutex: &'a StandardMutex) -> Self {
        Self::from_option(Some(mutex))
    }

    /// The pointer overload: locks only a non-null mutex; `Sentry::from_option(None)`.
    pub fn from_option(mutex: Option<&'a StandardMutex>) -> Self {
        let Some(mutex) = mutex else {
            return Self {
                mutex: None,
                callback: None,
                lock_held: None,
            };
        };
        mutex.lock();
        Self::locked(mutex)
    }

    fn locked(mutex: &'a StandardMutex) -> Self {
        let lock_held = Arc::new(AtomicBool::new(true));
        let callback = sentry_unlock_callback(Arc::clone(&mutex.state), Arc::clone(&lock_held));
        callback.register_callback();
        Self {
            mutex: Some(mutex),
            callback: Some(callback),
            lock_held: Some(lock_held),
        }
    }
}

fn sentry_unlock_callback(
    state: Arc<MutexState>,
    lock_held: Arc<AtomicBool>,
) -> ErrorHandlerCallback {
    ErrorHandlerCallback::new(move || release_sentry_lock(&state, &lock_held))
}

fn release_sentry_lock(state: &MutexState, lock_held: &AtomicBool) {
    if lock_held.swap(false, Ordering::AcqRel) {
        state.unlock();
    }
}

impl Drop for Sentry<'_> {
    fn drop(&mut self) {
        if let Some(callback) = &self.callback {
            callback.unregister_callback();
        }
        if let (Some(mutex), Some(lock_held)) = (self.mutex, &self.lock_held) {
            release_sentry_lock(&mutex.state, lock_held);
        }
    }
}
