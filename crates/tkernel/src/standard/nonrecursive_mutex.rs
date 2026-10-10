//! Owned guards for the nonrecursive `std::mutex` members of native classes.
use super::{Failure, exceptions::program_error};
use std::{
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError},
    thread::{self, ThreadId},
};

#[derive(Default)]
struct MutexOwnership {
    owner: Mutex<Option<ThreadId>>,
    released: Condvar,
}
impl MutexOwnership {
    fn owner(&self) -> MutexGuard<'_, Option<ThreadId>> {
        self.owner.lock().unwrap_or_else(PoisonError::into_inner)
    }
    fn unlock(&self) -> Result<(), Failure> {
        let mut owner = self.owner();
        if *owner != Some(thread::current().id()) {
            return Err(program_error(&format!(
                "mutex owner {owner:?}; expected current thread {:?}",
                thread::current().id()
            )));
        }
        *owner = None;
        self.released.notify_one();
        Ok(())
    }
}

/// Nonrecursive native lock with explicit release; `mutex.lock(); mutex.unlock()?`.
#[derive(Default)]
pub struct NonRecursiveMutex {
    ownership: Arc<MutexOwnership>,
}
impl NonRecursiveMutex {
    /// Creates an unlocked mutex; `NonRecursiveMutex::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Acquires exclusive ownership; `mutex.lock()`.
    pub fn lock(&self) {
        let mut owner = self.ownership.owner();
        while owner.is_some() {
            owner = self
                .ownership
                .released
                .wait(owner)
                .unwrap_or_else(PoisonError::into_inner);
        }
        *owner = Some(thread::current().id());
    }
    /// Attempts acquisition, including rejection of recursion; `assert!(!mutex.try_lock())`.
    pub fn try_lock(&self) -> bool {
        let mut owner = self.ownership.owner();
        if owner.is_some() {
            return false;
        }
        *owner = Some(thread::current().id());
        true
    }
    /// Releases only the calling thread's ownership; `mutex.unlock()?`.
    pub fn unlock(&self) -> Result<(), Failure> {
        self.ownership.unlock()
    }
    /// Acquires a guard that can outlive a borrowed wrapper; `let guard = mutex.guard()`.
    pub fn guard(&self) -> NonRecursiveMutexGuard {
        self.lock();
        NonRecursiveMutexGuard {
            ownership: Arc::clone(&self.ownership),
            same_thread: PhantomData,
        }
    }
}

/// Owned lock, released on its acquiring thread; `drop(mutex.guard())`.
pub struct NonRecursiveMutexGuard {
    ownership: Arc<MutexOwnership>,
    // Native std::mutex may only be unlocked by its owning thread.
    same_thread: PhantomData<Rc<()>>,
}
impl Drop for NonRecursiveMutexGuard {
    fn drop(&mut self) {
        let _ = self.ownership.unlock();
    }
}
