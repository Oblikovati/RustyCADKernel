//! Threads, the OCCT thread pool and parallel loops: `OSD_Thread`, `OSD_ThreadPool`,
//! `OSD_ThreadPool::Launcher` and `OSD_Parallel` (the non-TBB build of this profile).
//!
//! The pool is a port of the native one on `std::thread`, so thread indices, the locking
//! of idle workers and the "busy pool runs in the caller" fallback match OCCT exactly.
mod event;
mod job;
mod launcher;
mod loops;
mod thread;
mod thread_pool;
mod worker;

pub use launcher::Launcher;
pub use loops::Parallel;
pub use thread::{OsdThread, ThreadFunction};
pub use thread_pool::ThreadPool;
