//! `OSD_ThreadPool::JobInterface` and `performJob`: a borrowed job handed to pool threads,
//! and the conversion of a failing job into the `Standard_ProgramError` the launcher raises.
//!
//! Like the native launcher, which passes a pointer to a job on its own stack, the pool
//! hands borrowed jobs to long-lived threads. The borrow is erased here, and only here:
//! `Launcher::perform` waits for every thread it woke before the job goes out of scope,
//! and every job runs under `catch_unwind`, so that wait cannot be skipped.
#![allow(unsafe_code)]
use crate::standard::Failure;
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
};

/// The job body: called once per participating thread with that thread's index.
pub(super) type JobBody<'a> = dyn Fn(i32) + Sync + 'a;

/// A job pointer whose borrow was erased; valid until the launcher's wait returns.
#[derive(Clone, Copy)]
pub(super) struct JobRef {
    body: *const JobBody<'static>,
}
// SAFETY: the body is `Sync`, so calling it through a shared pointer from any thread is
// sound; the pointer itself is only dereferenced while the launcher keeps the job alive.
unsafe impl Send for JobRef {}

impl JobRef {
    /// Erases the borrow of `body`; the caller must not let `body` end before every
    /// thread that received the reference reported idle.
    pub(super) fn erase(body: &JobBody<'_>) -> Self {
        let pointer: *const JobBody<'_> = body;
        // SAFETY: only the lifetime bound changes (identical fat-pointer layout); the
        // launcher's run/wait protocol keeps the referent alive for every dereference.
        let body: *const JobBody<'static> = unsafe { std::mem::transmute(pointer) };
        Self { body }
    }
    /// `OSD_ThreadPool::performJob`: runs the job for `thread_index` and records the first
    /// failure, returning it; `job.perform(0)`.
    pub(super) fn perform(self, thread_index: i32) -> Option<Failure> {
        // SAFETY: see `erase`; the launcher waits for this call to return.
        let body: &JobBody<'_> = unsafe { &*self.body };
        catch_unwind(AssertUnwindSafe(|| body(thread_index)))
            .err()
            .map(|payload| program_error(payload.as_ref()))
    }
}

/// The native catch clauses: a `Failure` payload keeps "Type: message" and its stack, a
/// text panic reads "panic: <text>" (the `std::exception` clause prints the type name),
/// any other payload is "Error: Unknown exception".
fn program_error(payload: &(dyn Any + Send)) -> Failure {
    if let Some(failure) = payload.downcast_ref::<Failure>() {
        let message: String = format!("{}: {}", failure.exception_type(), failure.message());
        return standard_program_error(&message, failure.stack_string());
    }
    let text: Option<&str> = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str));
    match text {
        Some(text) => standard_program_error(&format!("panic: {text}"), ""),
        None => standard_program_error("Error: Unknown exception", ""),
    }
}

/// `Standard_ProgramError(message, stack)`; `standard_program_error("bad", "")`.
pub(super) fn standard_program_error(message: &str, stack: &str) -> Failure {
    Failure::with_type("Standard_ProgramError", message, stack)
}
