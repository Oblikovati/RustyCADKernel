//! Owned callbacks preserve native copy/detach semantics without locking across joins.
use rustycad_tkernel::{
    osd::{OsdThread, OsdThreadCallback},
    standard::{NonRecursiveMutex, SharedState},
};
use std::sync::{
    Arc, Barrier, Weak,
    atomic::{AtomicUsize, Ordering},
};

struct OffsetCallback(usize);
impl OsdThreadCallback for OffsetCallback {
    fn invoke(&self, argument: usize) -> usize {
        self.0 + argument
    }
}
fn double(argument: usize) -> usize {
    2 * argument
}

#[test]
fn callback_rebinding_changes_future_runs_and_preserves_custom_functions() {
    let thread = OsdThread::new();
    let first: Arc<dyn OsdThreadCallback> = Arc::new(OffsetCallback(10));
    let second: Arc<dyn OsdThreadCallback> = Arc::new(OffsetCallback(20));
    assert!(!thread.rebind_callback(&first, second.clone()));
    thread.set_callback(first.clone());
    assert!(thread.run(2, 0));
    assert!(thread.rebind_callback(&first, second.clone()));
    assert_eq!(thread.wait_result(), Some(12));
    assert!(thread.run(3, 0));
    assert_eq!(thread.wait_result(), Some(23));
    assert!(!thread.rebind_callback(&first, second.clone()));
    thread.set_function(double);
    assert!(!thread.rebind_callback(&second, first));
    assert!(thread.run(4, 0));
    assert_eq!(thread.wait_result(), Some(8));
}

#[test]
fn owned_callbacks_copy_with_the_native_shared_thread_handle() {
    let thread = OsdThread::new();
    thread.set_callback(Arc::new(OffsetCallback(10)));
    assert!(thread.run(2, 0));
    let copy = thread.clone();
    assert_eq!(copy.wait_result(), Some(12));
    assert_eq!(thread.wait_result(), None);
    assert_ne!(thread.id(), 0);
    assert!(thread.run(3, 0));
    thread.set_callback(Arc::new(OffsetCallback(20)));
    assert_eq!(thread.id(), 0);
    assert_eq!(thread.wait_result(), None);
    assert!(thread.run(4, 0));
    assert_eq!(thread.wait_result(), Some(24));
}

struct SelfInspectingCallback {
    thread: SharedState<Weak<OsdThread>>,
    ready: Barrier,
    inspected: AtomicUsize,
}
impl OsdThreadCallback for SelfInspectingCallback {
    fn invoke(&self, _: usize) -> usize {
        self.ready.wait();
        let thread = self.thread.read().upgrade().unwrap();
        self.inspected.store(thread.id(), Ordering::Release);
        assert_eq!(thread.wait_result(), None, "native self-join is rejected");
        42
    }
}
#[test]
fn self_join_retains_the_handle_for_an_external_wait() {
    let thread = Arc::new(OsdThread::new());
    let callback = Arc::new(SelfInspectingCallback {
        thread: SharedState::new(Arc::downgrade(&thread)),
        ready: Barrier::new(2),
        inspected: AtomicUsize::new(0),
    });
    thread.set_callback(callback.clone());
    assert!(thread.run(0, 0));
    let id = thread.id();
    callback.ready.wait();
    assert_eq!(thread.wait_result(), Some(42));
    assert_eq!(callback.inspected.load(Ordering::Acquire), id);
    assert_eq!(thread.id(), 0);
}

#[test]
fn nonrecursive_mutex_rejects_recursion_and_foreign_unlock() {
    let mutex = Arc::new(NonRecursiveMutex::new());
    assert!(mutex.try_lock());
    assert!(!mutex.try_lock());
    let other = mutex.clone();
    std::thread::spawn(move || {
        assert!(!other.try_lock());
        assert!(
            other
                .unlock()
                .unwrap_err()
                .message()
                .contains("expected current thread")
        );
    })
    .join()
    .unwrap();
    mutex.unlock().unwrap();
    assert!(mutex.unlock().is_err());
    assert!(mutex.try_lock());
    mutex.unlock().unwrap();
}

#[test]
fn owned_mutex_guard_releases_for_a_waiting_thread_and_survives_its_wrapper() {
    let mutex = Arc::new(NonRecursiveMutex::new());
    let held = mutex.guard();
    let barrier = Arc::new(Barrier::new(2));
    let other = mutex.clone();
    let ready = barrier.clone();
    let waiter = std::thread::spawn(move || {
        assert!(!other.try_lock());
        ready.wait();
        let _guard = other.guard();
    });
    barrier.wait();
    drop(held);
    waiter.join().unwrap();
    let guard = mutex.guard();
    drop(mutex);
    drop(guard);
}

#[test]
fn native_self_assignment_detaches_but_retains_the_function() {
    let thread = OsdThread::with_function(double);
    assert!(thread.run(3, 0));
    thread.assign(&thread);
    assert_eq!(thread.id(), 0);
    assert!(!thread.wait());
    assert!(thread.run(4, 0));
    assert_eq!(thread.wait_result(), Some(8));
}
