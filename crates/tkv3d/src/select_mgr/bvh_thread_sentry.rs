//! Owning sentry for all pool workers, preserving array lock order.
use super::SelectMgrBvhThreadPool;
use rustycad_tkernel::standard::{
    Failure, Handle, NonRecursiveMutexGuard, exceptions::program_error,
};

/// Locks all BVH workers while retaining their pool; `let sentry = SelectMgrBvhThreadSentry::new(pool)`.
pub struct SelectMgrBvhThreadSentry {
    pool: Handle<SelectMgrBvhThreadPool>,
    guards: Vec<NonRecursiveMutexGuard>,
}
impl SelectMgrBvhThreadSentry {
    /// Locks immediately; a null pool is permitted; `SelectMgrBvhThreadSentry::new(Handle::null())`.
    pub fn new(pool: Handle<SelectMgrBvhThreadPool>) -> Self {
        let mut sentry = Self {
            pool,
            guards: Vec::new(),
        };
        sentry.lock().expect("new sentry is unlocked");
        sentry
    }
    /// Acquires all mutexes in array order; `sentry.unlock(); sentry.lock()?`.
    pub fn lock(&mut self) -> Result<(), Failure> {
        if !self.guards.is_empty() {
            return Err(program_error(&format!(
                "sentry holds {} mutexes; expected unlocked sentry",
                self.guards.len()
            )));
        }
        if let Some(pool) = self.pool.get() {
            self.guards = pool
                .worker_mutexes()
                .iter()
                .map(|mutex| mutex.guard())
                .collect();
        }
        Ok(())
    }
    /// Releases the acquired mutexes in array order; `sentry.unlock()`.
    pub fn unlock(&mut self) {
        self.guards.clear();
    }
}
impl Drop for SelectMgrBvhThreadSentry {
    fn drop(&mut self) {
        self.unlock();
    }
}
