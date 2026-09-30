//! `OSD_ThreadPool::EnumeratedThread`: one pool thread (or the launcher's self-thread),
//! locked by at most one launcher, started lazily by its first job.
use super::{
    event::ManualResetEvent,
    job::{JobRef, standard_program_error},
};
use crate::standard::Failure;
use std::{
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
    thread::JoinHandle,
};

/// A pool slot; shared between the pool, the launcher that locked it and its thread.
pub(super) struct Worker {
    usage: AtomicI32,
    index: AtomicI32,
    wake: ManualResetEvent,
    idle: ManualResetEvent,
    job: Mutex<Option<JobRef>>,
    failure: Mutex<Option<Failure>>,
    shutdown: AtomicBool,
    handle: Mutex<Option<JoinHandle<()>>>,
    is_self_thread: bool,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // Jobs run under catch_unwind outside these locks, so poisoning never hides a torn state.
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

impl Worker {
    /// A pool thread (`is_self_thread` false) or a launcher's self-thread, with its index.
    pub(super) fn new(index: i32, is_self_thread: bool) -> Arc<Self> {
        Arc::new(Self {
            usage: AtomicI32::new(0),
            index: AtomicI32::new(index),
            wake: ManualResetEvent::default(),
            idle: ManualResetEvent::default(),
            job: Mutex::new(None),
            failure: Mutex::new(None),
            shutdown: AtomicBool::new(false),
            handle: Mutex::new(None),
            is_self_thread,
        })
    }
    /// `Lock`: claims an unused slot; false when a launcher holds it already.
    pub(super) fn lock(&self) -> bool {
        self.usage.swap(1, Ordering::SeqCst) == 0
    }
    /// `Free`: returns the slot to the pool.
    pub(super) fn free(&self) {
        self.usage.store(0, Ordering::SeqCst);
    }
    /// Renumbers the slot for the launcher that locked it (native `myThreadIndex`).
    pub(super) fn set_index(&self, index: i32) {
        self.index.store(index, Ordering::SeqCst);
    }
    /// The recorded failure of the last job, if any.
    pub(super) fn failure(&self) -> Option<Failure> {
        lock(&self.failure).clone()
    }
    /// `WakeUp`: the self-thread runs the job at once; a pool thread gets it through the
    /// wake event and is started by its first job.
    pub(super) fn wake_up(self: &Arc<Self>, job: Option<JobRef>) {
        *lock(&self.job) = job;
        if self.is_self_thread {
            // Native quirk: performJob only ever sets the self-thread's failure, so a
            // failure stays recorded for every later job of the same launcher.
            if let Some(failure) =
                job.and_then(|job| job.perform(self.index.load(Ordering::SeqCst)))
            {
                *lock(&self.failure) = Some(failure);
            }
            return;
        }
        self.wake.set();
        if job.is_some() && lock(&self.handle).is_none() {
            self.start();
        }
    }
    /// `WaitIdle`: waits for a pool thread to finish its job; the self-thread never waits.
    pub(super) fn wait_idle(&self) {
        if !self.is_self_thread {
            self.idle.wait();
            self.idle.reset();
        }
    }
    /// Pool `release` for this slot: asks the thread to exit and joins it (`Wait`, which
    /// natively returns false for a never-started thread).
    pub(super) fn shut_down(self: &Arc<Self>) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.wake_up(None);
        if let Some(handle) = lock(&self.handle).take() {
            // The thread body never panics (jobs are caught), so join cannot fail.
            let _ = handle.join();
        }
    }
    fn start(self: &Arc<Self>) {
        let worker: Arc<Self> = Arc::clone(self);
        match std::thread::Builder::new().spawn(move || worker.perform_thread()) {
            Ok(handle) => *lock(&self.handle) = Some(handle),
            Err(error) => self.refuse_job(&error.to_string()),
        }
    }
    /// Rust deviation: natively a thread that cannot start leaves the launcher waiting
    /// forever; here the job is dropped, recorded as a failure and the slot reports idle.
    fn refuse_job(&self, reason: &str) {
        *lock(&self.job) = None;
        let message: String = format!("Error: unable to start a pool thread ({reason})");
        *lock(&self.failure) = Some(standard_program_error(&message, ""));
        self.idle.set();
    }
    /// `performThread`: waits for work until the pool shuts down.
    fn perform_thread(&self) {
        loop {
            self.wake.wait();
            self.wake.reset();
            if self.shutdown.load(Ordering::SeqCst) {
                return;
            }
            let job: Option<JobRef> = lock(&self.job).take();
            let failure: Option<Failure> =
                job.and_then(|job| job.perform(self.index.load(Ordering::SeqCst)));
            *lock(&self.failure) = failure;
            self.idle.set();
        }
    }
}
