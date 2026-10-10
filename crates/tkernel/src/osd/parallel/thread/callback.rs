//! Owned thread arguments avoid reconstructing pointers when porting native callbacks.
use super::ThreadFunction;
use std::sync::Arc;

/// Safe context for a native thread entry point; `thread.set_callback(Arc::new(job))`.
pub trait OsdThreadCallback: Send + Sync {
    /// Executes with the caller's opaque argument; `job.invoke(42)`.
    fn invoke(&self, argument: usize) -> usize;
}

#[derive(Clone)]
pub(super) enum ThreadEntry {
    Opaque(ThreadFunction),
    Owned(Arc<dyn OsdThreadCallback>),
}
impl ThreadEntry {
    pub(super) fn invoke(&self, argument: usize) -> usize {
        match self {
            Self::Opaque(function) => function(argument),
            Self::Owned(callback) => callback.invoke(argument),
        }
    }
}
