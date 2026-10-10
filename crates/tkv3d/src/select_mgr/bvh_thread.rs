//! Native worker copies share OSD handles while retaining distinct BVH mutexes.
use super::SelectMgrBvhThreadPoolFields;
use crate::select3d::Select3DSensitiveEntity;
use rustycad_tkernel::{
    osd::{OsdThread, OsdThreadCallback, SignalState},
    standard::{Handle, NonRecursiveMutex, SharedState},
    tcollection::AsciiString,
};
use std::{
    ops::Deref,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Weak, atomic::Ordering},
};

#[derive(Clone, Default)]
struct BvhWorkerBinding {
    pool: Weak<SelectMgrBvhThreadPoolFields>,
    catch_fpe: bool,
}
#[derive(Default)]
struct BvhWorkerContext {
    binding: SharedState<BvhWorkerBinding>,
    mutex: Arc<NonRecursiveMutex>,
}
impl OsdThreadCallback for BvhWorkerContext {
    fn invoke(&self, _argument: usize) -> usize {
        let binding = self.binding.snapshot();
        let Some(pool) = binding.pool.upgrade() else {
            return 0;
        };
        let signals = SignalState::global();
        signals.set_thread_local_signal(
            pool.signal_host.as_ref(),
            signals.signal_mode(),
            binding.catch_fpe,
        );
        self.perform(&pool)
    }
}
impl BvhWorkerContext {
    fn perform(&self, pool: &SelectMgrBvhThreadPoolFields) -> usize {
        loop {
            pool.wake.wait();
            if pool.stop.load(Ordering::Acquire) {
                return 0;
            }
            if let Some(entity) = pool.next_entity() {
                let _build_guard = self.mutex.guard();
                build_sensitive(pool, &entity);
            }
        }
    }
}
fn build_sensitive(
    pool: &SelectMgrBvhThreadPoolFields,
    entity: &Handle<dyn Select3DSensitiveEntity>,
) {
    let Some(sensitive) = entity.get() else {
        return;
    };
    let failure = match catch_unwind(AssertUnwindSafe(|| sensitive.bvh())) {
        Ok(Ok(())) => return,
        Ok(Err(failure)) => format!("{}: {}", failure.exception_type(), failure.message()),
        Err(_) => "Error: Unknown exception".to_owned(),
    };
    pool.messenger.send_fail_text(&AsciiString::new(&failure));
}

/// A pool worker inheriting OSD thread controls; `worker.bvh_mutex().try_lock()`.
pub struct SelectMgrBvhThread {
    thread: Arc<OsdThread>,
    context: Arc<BvhWorkerContext>,
}
impl SelectMgrBvhThread {
    /// Creates a worker with no function or pool; `SelectMgrBvhThread::new()`.
    pub fn new() -> Self {
        Self {
            thread: Arc::new(OsdThread::new()),
            context: Arc::new(BvhWorkerContext::default()),
        }
    }
    pub(super) fn for_pool(pool: Weak<SelectMgrBvhThreadPoolFields>, catch_fpe: bool) -> Self {
        let worker = Self::new();
        *worker.context.binding.write() = BvhWorkerBinding { pool, catch_fpe };
        worker.thread.set_callback(worker.callback());
        worker
    }
    /// Copies the native thread handle and binding, with a fresh mutex; `SelectMgrBvhThread::from_thread(&worker)`.
    pub fn from_thread(other: &Self) -> Self {
        let copy = Self::new();
        copy.assign(other);
        copy
    }
    /// Copies the thread and pool binding while retaining this mutex; `worker.assign(&other)`.
    pub fn assign(&self, other: &Self) {
        self.thread.assign(&other.thread);
        *self.context.binding.write() = other.context.binding.snapshot();
        self.thread
            .rebind_callback(&other.callback(), self.callback());
    }
    /// Returns the nonrecursive build mutex; `let guard = worker.bvh_mutex().guard()`.
    pub fn bvh_mutex(&self) -> &NonRecursiveMutex {
        &self.context.mutex
    }
    /// Supplies a safe worker argument for the native callback; `worker.run(&worker, 0)`.
    /// Custom OSD functions still receive the target's opaque address.
    pub fn run(&self, target: &Self, stack_size: i32) -> bool {
        let own = self.callback();
        let argument = target.callback();
        let rebound = self.thread.rebind_callback(&own, argument.clone());
        let started = self
            .thread
            .run(std::ptr::from_ref(target) as usize, stack_size);
        if rebound {
            self.thread.rebind_callback(&argument, own);
        }
        started
    }
    fn callback(&self) -> Arc<dyn OsdThreadCallback> {
        self.context.clone()
    }
    pub(super) fn shared_thread(&self) -> Arc<OsdThread> {
        self.thread.clone()
    }
    pub(super) fn shared_mutex(&self) -> Arc<NonRecursiveMutex> {
        self.context.mutex.clone()
    }
}
impl Default for SelectMgrBvhThread {
    fn default() -> Self {
        Self::new()
    }
}
impl Clone for SelectMgrBvhThread {
    fn clone(&self) -> Self {
        Self::from_thread(self)
    }
    fn clone_from(&mut self, source: &Self) {
        self.assign(source);
    }
}
impl Deref for SelectMgrBvhThread {
    type Target = OsdThread;
    fn deref(&self) -> &Self::Target {
        &self.thread
    }
}
