//! OCCT parallel loops and launchers backed by an isolated Rayon pool.
use crate::standard::Failure;
use rayon::prelude::*;
use std::{ops::Range, sync::OnceLock};

/// An explicit reusable worker pool; `ThreadPool::new(4)?` creates four workers.
pub struct ThreadPool {
    workers: rayon::ThreadPool,
}
/// A borrowed pool launcher; `pool.launcher().perform(0..10, |worker,index| {})`.
pub struct Launcher<'a> {
    pool: &'a ThreadPool,
}
/// Convenience loops using the default pool; `Parallel::for_indices(0..10, |index| {})`.
pub struct Parallel;
impl ThreadPool {
    /// Creates a pool with a positive worker count; `ThreadPool::new(4)?`.
    pub fn new(threads: usize) -> Result<Self, Failure> {
        if threads == 0 {
            return Err(Failure::out_of_range("thread count 0: expected >= 1"));
        }
        let workers = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .map_err(|error| Failure::new(&error.to_string()))?;
        Ok(Self { workers })
    }
    /// Returns a lazily initialized process pool; `ThreadPool::default_pool().launcher()`.
    pub fn default_pool() -> &'static Self {
        static POOL: OnceLock<ThreadPool> = OnceLock::new();
        POOL.get_or_init(|| {
            Self::new(std::thread::available_parallelism().map_or(1, usize::from))
                .expect("could not create default worker pool")
        })
    }
    /// Borrows the pool for synchronous work; `pool.launcher()`.
    pub const fn launcher(&self) -> Launcher<'_> {
        Launcher { pool: self }
    }
    /// Reports worker count; `ThreadPool::new(4)?.thread_count() == 4`.
    pub fn thread_count(&self) -> usize {
        self.workers.current_num_threads()
    }
}
impl Launcher<'_> {
    /// Runs every index once, supplying a pool-local worker ID; `launcher.perform(0..10, |worker,index| {})`.
    pub fn perform(&self, range: Range<usize>, operation: impl Fn(usize, usize) + Sync + Send) {
        self.pool.workers.install(|| {
            range
                .into_par_iter()
                .for_each(|index| operation(worker_index(), index))
        });
    }
    /// Runs with disjoint mutable slots; `launcher.perform_mut(&mut values, |worker,index,value| *value=0)`.
    pub fn perform_mut<T: Send>(
        &self,
        values: &mut [T],
        operation: impl Fn(usize, usize, &mut T) + Sync + Send,
    ) {
        self.pool.workers.install(|| {
            values
                .par_iter_mut()
                .enumerate()
                .for_each(|(index, value)| operation(worker_index(), index, value))
        });
    }
    /// Runs with disjoint mutable chunks; `launcher.perform_chunks_mut(&mut values,10, |worker,index,chunk| chunk.fill(0))`.
    pub fn perform_chunks_mut<T: Send>(
        &self,
        values: &mut [T],
        size: usize,
        operation: impl Fn(usize, usize, &mut [T]) + Sync + Send,
    ) {
        assert!(size > 0, "chunk size {size}: expected >= 1");
        self.pool.workers.install(|| {
            values
                .par_chunks_mut(size)
                .enumerate()
                .for_each(|(index, chunk)| operation(worker_index(), index, chunk))
        });
    }
}
impl Parallel {
    /// Applies a shared callback to each index; `Parallel::for_indices(0..10, |index| {})`.
    pub fn for_indices(range: Range<usize>, operation: impl Fn(usize) + Sync + Send) {
        ThreadPool::default_pool()
            .launcher()
            .perform(range, |_, index| operation(index));
    }
    /// Applies a callback to each exclusive slot; `Parallel::for_each_mut(&mut values, |index,value| *value=0)`.
    pub fn for_each_mut<T: Send>(
        values: &mut [T],
        operation: impl Fn(usize, &mut T) + Sync + Send,
    ) {
        ThreadPool::default_pool()
            .launcher()
            .perform_mut(values, |_, index, value| operation(index, value));
    }
    /// Applies a callback to exclusive chunks; `Parallel::for_chunks_mut(&mut values,10, |index,chunk| chunk.fill(0))`.
    pub fn for_chunks_mut<T: Send>(
        values: &mut [T],
        size: usize,
        operation: impl Fn(usize, &mut [T]) + Sync + Send,
    ) {
        ThreadPool::default_pool().launcher().perform_chunks_mut(
            values,
            size,
            |_, index, chunk| operation(index, chunk),
        );
    }
}
fn worker_index() -> usize {
    rayon::current_thread_index().expect("callback runs inside the selected pool")
}
