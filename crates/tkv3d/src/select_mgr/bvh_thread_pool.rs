//! Background BVH construction with the native queue, events and worker controls.
use super::SelectMgrBvhThread;
use crate::select3d::Select3DSensitiveEntity;
use rustycad_tkernel::{
    message::{Message, Messenger},
    ncollection::{Array1, List},
    osd::{NativeSignalHost, OsdThread, SignalHost},
    standard::{
        Failure, Handle, NonRecursiveMutex, SharedState, StandardCondition, StandardTransient,
        exceptions::null_object,
    },
};
use std::sync::{
    Arc, RwLockWriteGuard,
    atomic::{AtomicBool, Ordering},
};

/// Protected native pool state, shared with workers; `pool.fields.started.load(Ordering::Acquire)`.
pub struct SelectMgrBvhThreadPoolFields {
    /// `myBVHToBuildList`.
    pub queued: SharedState<List<Handle<dyn Select3DSensitiveEntity>>>,
    /// `myBVHThreads`.
    pub threads: SharedState<Array1<'static, SelectMgrBvhThread>>,
    /// `myToStopBVHThread`.
    pub stop: AtomicBool,
    /// `myBVHListMutex`.
    pub list_mutex: NonRecursiveMutex,
    /// `myWakeEvent`.
    pub wake: StandardCondition,
    /// `myIdleEvent`.
    pub idle: StandardCondition,
    /// `myIsStarted`.
    pub started: AtomicBool,
    pub(super) start_control: NonRecursiveMutex,
    pub(super) messenger: Arc<Messenger>,
    pub(super) signal_host: Arc<dyn SignalHost + Send + Sync>,
}
impl SelectMgrBvhThreadPoolFields {
    fn new(
        count: i32,
        messenger: Arc<Messenger>,
        signal_host: Arc<dyn SignalHost + Send + Sync>,
    ) -> Self {
        Self {
            queued: SharedState::new(List::default()),
            threads: SharedState::new(
                Array1::with_bounds(1, count.max(1)).expect("positive worker bounds"),
            ),
            stop: AtomicBool::new(false),
            list_mutex: NonRecursiveMutex::new(),
            wake: StandardCondition::new(false),
            idle: StandardCondition::new(true),
            started: AtomicBool::new(false),
            start_control: NonRecursiveMutex::new(),
            messenger,
            signal_host,
        }
    }
    pub(super) fn next_entity(&self) -> Option<Handle<dyn Select3DSensitiveEntity>> {
        let _queue_guard = self.list_mutex.guard();
        // OCCT 3d097a0 checks stop before the queue lock. Serialize its reset
        // with shutdown's wake: a worker
        // past the outer stop check can erase the wake needed by its siblings.
        if self.stop.load(Ordering::Acquire) {
            return None;
        }
        let mut queued = self.queued.write();
        let Some(entity) = queued.first().cloned() else {
            self.wake.reset();
            self.idle.set();
            return None;
        };
        queued.remove_first().expect("first entry exists");
        Some(entity)
    }
    fn worker_handles(&self) -> Vec<Arc<OsdThread>> {
        self.threads
            .read()
            .as_slice()
            .iter()
            .map(SelectMgrBvhThread::shared_thread)
            .collect()
    }
}

/// Lazily started pool for sensitive-entity BVHs; `pool.add_entity(&entity)?; pool.wait_threads()`.
pub struct SelectMgrBvhThreadPool {
    transient: StandardTransient,
    /// `Protected native state`.
    pub fields: Arc<SelectMgrBvhThreadPoolFields>,
}
rustycad_tkernel::impl_transient!(
    SelectMgrBvhThreadPool,
    "SelectMgr_BVHThreadPool",
    StandardTransient,
    transient
);
impl SelectMgrBvhThreadPool {
    /// Captures the current floating-signal setting; `SelectMgrBvhThreadPool::new(2)`.
    pub fn new(count: i32) -> Self {
        Self::with_services(
            count,
            Message::default_messenger().clone(),
            Arc::new(NativeSignalHost),
        )
    }
    /// Supplies message output and signal operations; `SelectMgrBvhThreadPool::with_services(2, messenger, host)`.
    pub fn with_services(
        count: i32,
        messenger: Arc<Messenger>,
        signal_host: Arc<dyn SignalHost + Send + Sync>,
    ) -> Self {
        let catch_fpe = signal_host.to_catch_floating_signals();
        let fields = Arc::new(SelectMgrBvhThreadPoolFields::new(
            count,
            messenger,
            signal_host,
        ));
        for worker in fields.threads.write().as_mut_slice() {
            *worker = SelectMgrBvhThread::for_pool(Arc::downgrade(&fields), catch_fpe);
        }
        Self {
            transient: StandardTransient::default(),
            fields,
        }
    }
    /// Exposes the actual mutable workers; release the guard before pool operations.
    /// Example: `pool.threads().change_value(1)?.set_priority(2)`.
    pub fn threads(&self) -> RwLockWriteGuard<'_, Array1<'static, SelectMgrBvhThread>> {
        self.fields.threads.write()
    }
    /// Queues buildable entities without deduplication; `pool.add_entity(&entity)?`.
    pub fn add_entity(&self, entity: &Handle<dyn Select3DSensitiveEntity>) -> Result<(), Failure> {
        let sensitive = entity
            .get()
            .ok_or_else(|| null_object("entity is null; expected sensitive entity"))?;
        if !sensitive.to_build_bvh() {
            return Ok(());
        }
        {
            let _queue_guard = self.fields.list_mutex.guard();
            self.fields.queued.write().append(entity.clone());
            self.fields.wake.set();
            self.fields.idle.reset();
        }
        self.start_workers();
        Ok(())
    }
    fn start_workers(&self) {
        let _start_guard = self.fields.start_control.guard();
        if self.fields.started.swap(true, Ordering::AcqRel) {
            return;
        }
        for worker in self.fields.threads.read().as_slice() {
            worker.run(worker, 0);
        }
    }
    /// Waits for queue exhaustion and all worker mutexes; `pool.wait_threads()`.
    pub fn wait_threads(&self) {
        self.fields.idle.wait();
        let mutexes = self.worker_mutexes();
        let _guards: Vec<_> = mutexes.iter().map(|mutex| mutex.guard()).collect();
    }
    pub(super) fn worker_mutexes(&self) -> Vec<Arc<NonRecursiveMutex>> {
        self.fields
            .threads
            .read()
            .as_slice()
            .iter()
            .map(SelectMgrBvhThread::shared_mutex)
            .collect()
    }
    /// Stops workers, preserving any queued entities for restart; `pool.stop_threads()`.
    pub fn stop_threads(&self) {
        let Some(workers) = self.request_stop() else {
            return;
        };
        for worker in workers {
            worker.wait();
        }
        let _start_guard = self.fields.start_control.guard();
        self.fields.stop.store(false, Ordering::Release);
        self.fields.started.store(false, Ordering::Release);
    }
    fn request_stop(&self) -> Option<Vec<Arc<OsdThread>>> {
        let _start_guard = self.fields.start_control.guard();
        if !self.fields.started.load(Ordering::Acquire) {
            return None;
        }
        {
            let _queue_guard = self.fields.list_mutex.guard();
            self.fields.stop.store(true, Ordering::Release);
            self.fields.wake.set();
        }
        Some(self.fields.worker_handles())
    }
}
impl Drop for SelectMgrBvhThreadPool {
    fn drop(&mut self) {
        self.stop_threads();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn an_empty_worker_cannot_erase_the_shutdown_wake() {
        let pool = SelectMgrBvhThreadPool::new(2);
        pool.fields.stop.store(true, Ordering::Release);
        pool.fields.wake.set();
        // Represents a worker that passed its outer stop check before shutdown.
        assert!(pool.fields.next_entity().is_none());
        assert!(pool.fields.wake.check());
    }
}
