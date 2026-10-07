//! Threads, the OCCT thread pool and parallel loops: `OSD_Thread`, `OSD_ThreadPool`,
//! `OSD_ThreadPool::Launcher` and `OSD_Parallel` (the non-TBB build of this profile).
//!
//! The pool is a port of the native one on `std::thread`, so thread indices, the locking
//! of idle workers and the "busy pool runs in the caller" fallback match OCCT exactly.
mod event;
mod iterator_interface;
mod job;
mod launcher;
mod loops;
mod thread;
mod thread_pool;
mod universal_iterator;
mod worker;

pub use iterator_interface::{ParallelIteratorInterface, destroy_parallel_iterator};
pub use launcher::Launcher;
pub use loops::Parallel;
pub use thread::{OsdThread, ThreadFunction};
pub use thread_pool::ThreadPool;

pub use universal_iterator::ParallelUniversalIterator;
