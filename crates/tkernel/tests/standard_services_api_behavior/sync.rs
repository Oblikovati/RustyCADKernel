//! Standard_Mutex (recursive, with Sentry) and Standard_Condition.
use rustycad_tkernel::standard::mutex::Sentry;
use rustycad_tkernel::standard::{StandardCondition, StandardMutex};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn other_thread_try_lock(mutex: &StandardMutex) -> bool {
    thread::scope(|scope| {
        scope
            .spawn(|| {
                let locked = mutex.try_lock();
                if locked {
                    mutex.unlock();
                }
                locked
            })
            .join()
            .unwrap()
    })
}

#[test]
fn the_owner_relocks_and_unlocks_as_many_times() {
    let mutex = StandardMutex::new();
    mutex.lock();
    assert!(mutex.try_lock());
    assert!(!other_thread_try_lock(&mutex));
    mutex.unlock();
    assert!(!other_thread_try_lock(&mutex));
    mutex.unlock();
    assert!(other_thread_try_lock(&mutex));
}

#[test]
fn only_the_owner_can_unlock() {
    let mutex = StandardMutex::new();
    mutex.unlock();
    mutex.lock();
    thread::scope(|scope| scope.spawn(|| mutex.unlock()).join().unwrap());
    assert!(!other_thread_try_lock(&mutex));
    mutex.unlock();
    assert!(other_thread_try_lock(&mutex));
}

#[test]
fn lock_waits_for_the_other_thread_to_release() {
    let mutex = StandardMutex::new();
    let (locked, wait_locked) = mpsc::channel();
    let (release, wait_release) = mpsc::channel::<()>();
    let shared = &mutex;
    thread::scope(|scope| {
        scope.spawn(move || {
            shared.lock();
            locked.send(()).unwrap();
            wait_release.recv().unwrap();
            shared.unlock();
        });
        wait_locked.recv().unwrap();
        assert!(!mutex.try_lock());
        release.send(()).unwrap();
        mutex.lock();
        mutex.unlock();
    });
}

#[test]
fn sentries_lock_for_their_scope_and_ignore_null() {
    let mutex = StandardMutex::new();
    {
        let _sentry = Sentry::new(&mutex);
        assert!(!other_thread_try_lock(&mutex));
    }
    assert!(other_thread_try_lock(&mutex));
    let _null = Sentry::from_option(None);
    assert!(other_thread_try_lock(&mutex));
}

#[test]
fn a_sentry_unlocks_while_a_panic_unwinds() {
    let mutex = StandardMutex::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _sentry = Sentry::new(&mutex);
        panic!("raised inside the critical section");
    }));
    assert!(result.is_err());
    assert!(other_thread_try_lock(&mutex));
}

#[test]
fn condition_flag_operations() {
    let condition = StandardCondition::default();
    assert!(!condition.check());
    assert!(!condition.wait_timeout(0));
    assert!(!condition.wait_timeout(-3));
    condition.set();
    assert!(condition.check() && condition.wait_timeout(0));
    assert!(condition.check_reset());
    assert!(!condition.check_reset());
    let signaled = StandardCondition::new(true);
    signaled.wait();
    signaled.reset();
    assert!(!signaled.check());
}

#[test]
fn waiters_wake_when_another_thread_sets() {
    let condition = StandardCondition::new(false);
    thread::scope(|scope| {
        scope.spawn(|| {
            thread::sleep(Duration::from_millis(10));
            condition.set();
        });
        assert!(condition.wait_timeout(10_000));
    });
    condition.reset();
    thread::scope(|scope| {
        scope.spawn(|| condition.set());
        condition.wait();
    });
    assert!(condition.check());
}
