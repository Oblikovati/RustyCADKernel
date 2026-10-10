//! `OSD_Thread`: a function run on its own thread, with the pthread handle semantics of
//! the native class: copies share one thread, and detaching any copy (including by
//! destroying it) makes the thread unjoinable for all of them.
use super::event::ManualResetEvent;
use crate::standard::SharedState;
use std::{
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicI32, AtomicUsize, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

mod callback;
pub(super) mod native;
pub use callback::OsdThreadCallback;
use callback::ThreadEntry;

/// `OSD_ThreadFunction` (`void* (*)(void*)`): the opaque pointer-sized argument and
/// result travel as `usize`; `fn double(data: usize) -> usize { data * 2 }`.
pub type ThreadFunction = fn(usize) -> usize;

/// One started thread, shared by the `OsdThread` copies that refer to it.
struct RunningThread {
    handle: Mutex<Option<JoinHandle<usize>>>,
    finished: ManualResetEvent,
}
impl RunningThread {
    fn spawn(
        function: ThreadEntry,
        argument: usize,
        stack_size: i32,
    ) -> Option<(Arc<Self>, JoinHandle<usize>)> {
        let running: Arc<RunningThread> = Arc::new(RunningThread {
            handle: Mutex::new(None),
            finished: ManualResetEvent::default(),
        });
        let signal: FinishSignal = FinishSignal(Arc::clone(&running));
        let spawned = thread_builder(stack_size).spawn(move || {
            let _signal: FinishSignal = signal;
            function.invoke(argument)
        });
        let Ok(handle) = spawned else {
            return None;
        };
        Some((running, handle))
    }

    fn take_joinable(&self) -> Option<JoinHandle<usize>> {
        let mut handle = self.handle();
        // pthread_join rejects self-join without consuming the native handle.
        if handle.as_ref()?.thread().id() == std::thread::current().id() {
            return None;
        }
        handle.take()
    }
    fn handle(&self) -> MutexGuard<'_, Option<JoinHandle<usize>>> {
        self.handle
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

/// Signals the end of the thread function even when it panics.
struct FinishSignal(Arc<RunningThread>);
impl Drop for FinishSignal {
    fn drop(&mut self) {
        self.0.finished.set();
    }
}

/// `OSD_Thread`; `let mut thread = OsdThread::with_function(double); thread.run(21, 0);`.
pub struct OsdThread {
    function: SharedState<Option<ThreadEntry>>,
    thread: SharedState<Option<Arc<RunningThread>>>,
    thread_id: AtomicUsize,
    priority: AtomicI32,
}

impl Default for OsdThread {
    fn default() -> Self {
        Self::new()
    }
}

impl OsdThread {
    /// A thread without function; `OsdThread::new().run(0, 0) == false`.
    pub const fn new() -> Self {
        Self {
            function: SharedState::new(None),
            thread: SharedState::new(None),
            thread_id: AtomicUsize::new(0),
            priority: AtomicI32::new(0),
        }
    }
    /// A thread ready to run `function`; `OsdThread::with_function(double)`.
    pub const fn with_function(function: ThreadFunction) -> Self {
        Self {
            function: SharedState::new(Some(ThreadEntry::Opaque(function))),
            thread: SharedState::new(None),
            thread_id: AtomicUsize::new(0),
            priority: AtomicI32::new(0),
        }
    }
    /// `Assign`: copies the function and priority, detaches this thread, then shares the
    /// other's thread and identifier; `thread.assign(&other)`.
    pub fn assign(&self, other: &Self) {
        *self.function.write() = other.function.snapshot();
        self.priority
            .store(other.priority.load(Ordering::Acquire), Ordering::Release);
        self.detach();
        *self.thread.write() = other.thread.snapshot();
        self.thread_id.store(other.id(), Ordering::Release);
    }
    /// Stores the priority; applied with `SetThreadPriority` on Windows only (the pthread
    /// build keeps the value without effect); `thread.set_priority(1)`.
    pub fn set_priority(&self, priority: i32) {
        self.priority.store(priority, Ordering::Release);
        if let Some(thread) = self.thread.snapshot()
            && let Some(handle) = thread.handle().as_ref()
        {
            native::apply_priority(handle, priority);
        }
    }
    /// Detaches the current thread, then stores the function; `thread.set_function(double)`.
    pub fn set_function(&self, function: ThreadFunction) {
        self.detach();
        *self.function.write() = Some(ThreadEntry::Opaque(function));
    }
    /// `Run`: starts `function(data)` on a new thread after detaching the previous one;
    /// false without function or when the system refuses the thread; `thread.run(21, 0)`.
    ///
    /// `stack_size` is honoured on Windows only when positive, as natively (the pthread
    /// build ignores it).
    pub fn run(&self, data: usize, stack_size: i32) -> bool {
        let Some(function) = self.function.snapshot() else {
            return false;
        };
        self.detach();
        let Some((running, handle)) = RunningThread::spawn(function, data, stack_size) else {
            return false;
        };
        self.thread_id
            .store(native::thread_id(&handle), Ordering::Release);
        native::apply_priority(&handle, self.priority.load(Ordering::Acquire));
        *running.handle() = Some(handle);
        *self.thread.write() = Some(running);
        true
    }
    /// Retains safe callback context until execution ends; `thread.set_callback(callback)`.
    pub fn set_callback(&self, callback: Arc<dyn OsdThreadCallback>) {
        self.detach();
        *self.function.write() = Some(ThreadEntry::Owned(callback));
    }
    /// Rebinds a derived wrapper's future callback without detaching a shared run;
    /// `thread.rebind_callback(&previous, replacement)` preserves custom functions.
    pub fn rebind_callback(
        &self,
        previous: &Arc<dyn OsdThreadCallback>,
        replacement: Arc<dyn OsdThreadCallback>,
    ) -> bool {
        let mut function = self.function.write();
        let Some(ThreadEntry::Owned(callback)) = function.as_ref() else {
            return false;
        };
        if !Arc::ptr_eq(callback, previous) {
            return false;
        }
        *function = Some(ThreadEntry::Owned(replacement));
        true
    }
    /// `Detach`: lets the thread run on unjoined, for every copy; `thread.detach()`.
    pub fn detach(&self) {
        if let Some(thread) = self.thread.write().take() {
            // Dropping the JoinHandle is pthread_detach.
            drop(thread.handle().take());
        }
        self.thread_id.store(0, Ordering::Release);
    }
    /// `Wait()`: joins the thread; `thread.wait()`.
    pub fn wait(&self) -> bool {
        self.wait_result().is_some()
    }
    /// `Wait(void*&)`: joins the thread and returns its result; `None` (native false with
    /// a null result) when nothing runs, when a copy already joined or detached it, or when
    /// the thread function panicked (natively a throwing thread terminates the process).
    pub fn wait_result(&self) -> Option<usize> {
        let thread: Arc<RunningThread> = self.thread.snapshot()?;
        self.join(&thread)
    }
    /// `Wait(int, void*&)`: joins the thread if it ends within `milliseconds`;
    /// `thread.wait_for(100)`.
    ///
    /// macOS has no `pthread_timedjoin_np`, so there the native call ignores the timeout
    /// and joins; the port does the same.
    pub fn wait_for(&self, milliseconds: i32) -> Option<usize> {
        if cfg!(target_vendor = "apple") {
            return self.wait_result();
        }
        let thread: Arc<RunningThread> = self.thread.snapshot()?;
        if !thread.finished.wait_for(join_timeout(milliseconds)) {
            return None;
        }
        self.join(&thread)
    }
    fn join(&self, thread: &Arc<RunningThread>) -> Option<usize> {
        // A failed join leaves the handle set, as natively.
        let handle: JoinHandle<usize> = thread.take_joinable()?;
        let result: usize = handle.join().ok()?;
        let mut current = self.thread.write();
        if current
            .as_ref()
            .is_some_and(|running| Arc::ptr_eq(running, thread))
        {
            *current = None;
            self.thread_id.store(0, Ordering::Release);
        }
        Some(result)
    }
    /// The identifier recorded by `run`, 0 when nothing runs; `thread.id()`.
    pub fn id(&self) -> usize {
        self.thread_id.load(Ordering::Acquire)
    }
    /// `Current`: the identifier of the calling thread (`pthread_self`,
    /// `GetCurrentThreadId` on Windows); `OsdThread::current()`.
    pub fn current() -> usize {
        native::current_thread_id()
    }
}

/// The native timeout: a negative value is a past deadline for pthread_timedjoin_np,
/// and a DWORD cast (almost 50 days) for WaitForSingleObject on Windows.
fn join_timeout(milliseconds: i32) -> Duration {
    let milliseconds: u64 = if cfg!(windows) {
        u64::from(milliseconds as u32)
    } else {
        u64::try_from(milliseconds).unwrap_or(0)
    };
    Duration::from_millis(milliseconds)
}

#[cfg(windows)]
fn thread_builder(stack_size: i32) -> std::thread::Builder {
    let builder: std::thread::Builder = std::thread::Builder::new();
    match usize::try_from(stack_size) {
        Ok(size) if size > 0 => builder.stack_size(size),
        _ => builder,
    }
}

#[cfg(not(windows))]
fn thread_builder(_stack_size: i32) -> std::thread::Builder {
    std::thread::Builder::new()
}

impl Clone for OsdThread {
    /// The copy constructor: shares the thread (see [`OsdThread::assign`]).
    fn clone(&self) -> Self {
        let copy: Self = Self::new();
        copy.assign(self);
        copy
    }
    /// `operator=`: [`OsdThread::assign`].
    fn clone_from(&mut self, source: &Self) {
        self.assign(source);
    }
}

impl Drop for OsdThread {
    /// `~OSD_Thread`: detaches the thread, for every copy (native quirk).
    fn drop(&mut self) {
        self.detach();
    }
}
