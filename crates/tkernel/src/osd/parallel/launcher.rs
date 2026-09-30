//! `OSD_ThreadPool::Launcher`: locks idle pool workers for one parallel job; the caller
//! joins as the last thread index, and alone when the pool is busy.
use super::{
    ThreadPool,
    job::{JobBody, JobRef, standard_program_error},
    worker::Worker,
};
use crate::standard::Failure;
use std::{
    marker::PhantomData,
    sync::{
        Arc, Mutex,
        atomic::{AtomicI32, Ordering},
    },
};

/// `OSD_ThreadPool::Launcher`; `Launcher::new(&pool, -1).perform(0, 10, |thread, index| {})?`.
pub struct Launcher<'a> {
    // Locked pool workers followed by the self-thread (native `myThreads`).
    threads: Vec<Arc<Worker>>,
    self_thread: Arc<Worker>,
    nb_threads: i32,
    pool: PhantomData<&'a ThreadPool>,
}

/// The native launcher size: `max_threads > 0` caps it by the pool, a negative value asks
/// for the pool default (at least 1), and 0 means the caller alone.
fn requested_threads(pool: &ThreadPool, max_threads: i32) -> i32 {
    if max_threads > 0 {
        max_threads.min(pool.nb_threads())
    } else if max_threads < 0 {
        pool.nb_default_threads_to_launch().max(1)
    } else {
        1
    }
}

impl<'a> Launcher<'a> {
    /// Locks up to `requested - 1` idle workers, numbered from 0 in pool order, then adds
    /// the calling thread with the next index; `Launcher::new(&pool, -1)`.
    pub fn new(pool: &'a ThreadPool, max_threads: i32) -> Self {
        let requested: i32 = requested_threads(pool, max_threads);
        let mut threads: Vec<Arc<Worker>> = Vec::new();
        if requested > 1 {
            for worker in pool.workers().iter() {
                if !worker.lock() {
                    continue;
                }
                worker.set_index(threads.len() as i32);
                threads.push(Arc::clone(worker));
                if threads.len() as i32 == requested - 1 {
                    break;
                }
            }
        }
        let self_thread: Arc<Worker> = Worker::new(threads.len() as i32, true);
        threads.push(Arc::clone(&self_thread));
        let nb_threads: i32 = threads.len() as i32;
        Self {
            threads,
            self_thread,
            nb_threads,
            pool: PhantomData,
        }
    }
    /// Tells whether more than one thread was locked; `launcher.has_threads()`.
    pub const fn has_threads(&self) -> bool {
        self.nb_threads >= 2
    }
    /// The locked workers plus the caller; `launcher.nb_threads()`.
    pub const fn nb_threads(&self) -> i32 {
        self.nb_threads
    }
    /// Always 0; `launcher.lower_thread_index()`.
    pub const fn lower_thread_index(&self) -> i32 {
        0
    }
    /// `nb_threads() - 1`, the caller's index; `launcher.upper_thread_index()`.
    pub const fn upper_thread_index(&self) -> i32 {
        self.lower_thread_index() + self.nb_threads - 1
    }
    /// `Perform`: calls `functor(thread_index, index)` once for each index in
    /// `begin..end`, handed out through a shared atomic counter; `launcher.perform(0, 8, f)?`.
    ///
    /// A panicking functor stops its thread's share; one failure is returned as
    /// `Standard_ProgramError("<type>: <message>")`, several as "Multiple exceptions:\n…".
    /// After `release` nothing runs (native quirk).
    pub fn perform(
        &self,
        begin: i32,
        end: i32,
        functor: impl Fn(i32, i32) + Sync,
    ) -> Result<(), Failure> {
        let next: AtomicI32 = AtomicI32::new(begin);
        self.perform_job(&|thread_index: i32| {
            // Native `for (it = It(); it < End(); it = It())` with a wrapping fetch_add.
            let mut index: i32 = next.fetch_add(1, Ordering::SeqCst);
            while index < end {
                functor(thread_index, index);
                index = next.fetch_add(1, Ordering::SeqCst);
            }
        })
    }
    /// `OSD_Parallel_Threads::UniversalLauncher::Perform`: hands out the items of `items`
    /// one at a time under a mutex and calls `functor(thread_index, item)`; the way Rust
    /// shares mutable elements (`values.iter_mut()`); `launcher.perform_iter(v.iter_mut(), f)?`.
    pub fn perform_iter<I>(
        &self,
        items: I,
        functor: impl Fn(i32, I::Item) + Sync,
    ) -> Result<(), Failure>
    where
        I: Iterator + Send,
        I::Item: Send,
    {
        let range: Mutex<I> = Mutex::new(items);
        self.perform_job(&|thread_index: i32| {
            while let Some(item) = next_item(&range) {
                functor(thread_index, item);
            }
        })
    }
    /// `perform` = `run` then `wait`; the job must outlive both, which this frame ensures.
    fn perform_job(&self, body: &JobBody<'_>) -> Result<(), Failure> {
        let job: JobRef = JobRef::erase(body);
        self.threads
            .iter()
            .for_each(|thread| thread.wake_up(Some(job)));
        self.wait()
    }
    /// Waits for every woken thread, then raises the collected failures.
    fn wait(&self) -> Result<(), Failure> {
        self.threads.iter().for_each(|thread| thread.wait_idle());
        let failures: Vec<Failure> = self.threads.iter().filter_map(|t| t.failure()).collect();
        match failures.as_slice() {
            [] => Ok(()),
            [single] => Err(single.clone()),
            several => {
                let texts: Vec<&str> = several.iter().map(Failure::message).collect();
                let message: String = format!("Multiple exceptions:\n{}", texts.join("\n"));
                Err(standard_program_error(&message, ""))
            }
        }
    }
    /// `Release`: frees the locked workers; later `perform` calls do nothing; `launcher.release()`.
    pub fn release(&mut self) {
        self.threads
            .iter()
            .filter(|thread| !Arc::ptr_eq(thread, &self.self_thread))
            .for_each(|thread| thread.free());
        self.threads.clear();
        self.nb_threads = 0;
    }
}

fn next_item<I: Iterator>(range: &Mutex<I>) -> Option<I::Item> {
    range
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .next()
}

impl Drop for Launcher<'_> {
    /// `~Launcher`: releases the locked workers.
    fn drop(&mut self) {
        self.release();
    }
}
