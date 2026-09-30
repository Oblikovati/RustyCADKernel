//! `OSD_ThreadPool`: `NbThreads() - 1` lazily started worker threads; the thread that
//! runs a launcher is always the last participant.
use super::{Parallel, job::standard_program_error, worker::Worker};
use crate::standard::Failure;
use std::sync::{
    Arc, Mutex, MutexGuard, OnceLock,
    atomic::{AtomicI32, Ordering},
};

/// `OSD_ThreadPool`; `ThreadPool::new(4).nb_threads() == 4`.
pub struct ThreadPool {
    workers: Mutex<Vec<Arc<Worker>>>,
    nb_default_threads: AtomicI32,
}

impl ThreadPool {
    /// `DefaultPool`: the process-wide pool, sized by the first call only (a static
    /// initialised once, as natively); `ThreadPool::default_pool(-1)`.
    pub fn default_pool(nb_threads: i32) -> &'static Self {
        static THE_GLOBAL_POOL: OnceLock<ThreadPool> = OnceLock::new();
        THE_GLOBAL_POOL.get_or_init(|| Self::new(nb_threads))
    }
    /// Creates `nb_threads` participants (a value <= 0 means `NbLogicalProcessors`), all
    /// launched by default; `ThreadPool::new(2)`.
    pub fn new(nb_threads: i32) -> Self {
        let pool: Self = Self {
            workers: Mutex::new(Vec::new()),
            nb_default_threads: AtomicI32::new(0),
        };
        pool.init(nb_threads)
            .expect("a fresh pool has no locked thread to refuse reinitialization");
        pool.nb_default_threads
            .store(pool.nb_threads(), Ordering::SeqCst);
        pool
    }
    pub(super) fn workers(&self) -> MutexGuard<'_, Vec<Arc<Worker>>> {
        self.workers
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
    /// Tells whether more than one thread can take part; `pool.has_threads()`.
    pub fn has_threads(&self) -> bool {
        self.nb_threads() >= 2
    }
    /// Always 0; `pool.lower_thread_index()`.
    pub const fn lower_thread_index(&self) -> i32 {
        0
    }
    /// `LowerThreadIndex() + <worker count>`, i.e. `nb_threads() - 1`; `pool.upper_thread_index()`.
    pub fn upper_thread_index(&self) -> i32 {
        self.lower_thread_index() + self.worker_count()
    }
    /// The workers plus the calling thread; `pool.nb_threads()`.
    pub fn nb_threads(&self) -> i32 {
        self.worker_count() + 1
    }
    fn worker_count(&self) -> i32 {
        i32::try_from(self.workers().len()).expect("worker count fits the native int")
    }
    /// The launcher size used for a negative `max_threads`; `pool.nb_default_threads_to_launch()`.
    pub fn nb_default_threads_to_launch(&self) -> i32 {
        self.nb_default_threads.load(Ordering::SeqCst)
    }
    /// Stores the default launcher size unchecked, as natively; `pool.set_nb_default_threads_to_launch(2)`.
    pub fn set_nb_default_threads_to_launch(&self, nb_threads: i32) {
        self.nb_default_threads.store(nb_threads, Ordering::SeqCst);
    }
    /// Tells whether a launcher holds any worker; `pool.is_in_use()`.
    pub fn is_in_use(&self) -> bool {
        self.workers().iter().any(|worker| {
            let locked: bool = worker.lock();
            if locked {
                worker.free();
            }
            !locked
        })
    }
    /// `Init`: re-creates the workers for `nb_threads` participants (<= 0 means
    /// `NbLogicalProcessors`); `pool.init(3)?`.
    ///
    /// Keeps `nb_default_threads_to_launch` unchanged (native quirk). Fails with
    /// `Standard_ProgramError` when a launcher holds a worker.
    pub fn init(&self, nb_threads: i32) -> Result<(), Failure> {
        let requested: i32 = if nb_threads > 0 {
            nb_threads
        } else {
            Parallel::nb_logical_processors()
        };
        let count: i32 = (requested - 1).max(0);
        let mut workers = self.workers();
        if i32::try_from(workers.len()) == Ok(count) {
            return Ok(());
        }
        lock_all(&workers)?;
        release(&workers);
        *workers = (0..count).map(|index| Worker::new(index, false)).collect();
        Ok(())
    }
}

/// Locks every worker before a reinitialization, undoing the partial locks on refusal.
fn lock_all(workers: &[Arc<Worker>]) -> Result<(), Failure> {
    let busy: Option<usize> = workers.iter().position(|worker| !worker.lock());
    let Some(busy) = busy else {
        return Ok(());
    };
    workers[..busy].iter().for_each(|worker| worker.free());
    Err(standard_program_error(
        "Error: active ThreadPool is reinitialized",
        "",
    ))
}

/// `release`: stops and joins every worker thread.
fn release(workers: &[Arc<Worker>]) {
    workers.iter().for_each(|worker| worker.shut_down());
}

impl Drop for ThreadPool {
    /// `~OSD_ThreadPool`: stops and joins the worker threads.
    fn drop(&mut self) {
        release(&self.workers());
    }
}
