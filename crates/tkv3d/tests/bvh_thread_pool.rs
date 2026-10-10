//! Native pool lifecycle and concurrency, coordinated with barriers rather than sleeps.
#[path = "support/bvh_pool_fixture.rs"]
mod fixture;
use fixture::*;
use rustycad_tkernel::standard::{Handle, RuntimeTyped, SharedState};
use rustycad_tkv3d::{select_mgr::*, select3d::Select3DSensitiveEntity};
use std::sync::{Arc, Barrier, atomic::Ordering};

#[test]
fn construction_clamps_bounds_and_rejected_entities_do_not_start_workers() {
    let (pool, signals, _) = pool_fixture(-8);
    {
        let threads = pool.threads();
        assert_eq!((threads.lower(), threads.upper()), (1, 1));
    }
    assert_eq!(
        SelectMgrBvhThreadPool::static_type().name(),
        "SelectMgr_BVHThreadPool"
    );
    let entity = Handle::new(PoolBuildJob::new(0, 0));
    entity.buildable.store(false, Ordering::Release);
    pool.add_entity(&entity.cast()).unwrap();
    assert!(!pool.fields.started.load(Ordering::Acquire));
    assert!(signals.applied.read().is_empty());
    assert!(pool.add_entity(&Handle::default()).is_err());
    pool.wait_threads();
    pool.stop_threads();
    let native = SelectMgrBvhThreadPool::new(0);
    assert_eq!(native.threads().len(), 1);
}

#[test]
fn duplicate_jobs_and_failures_preserve_fifo_and_processing_continues() {
    let (pool, signals, printer) = pool_fixture(1);
    let order = Arc::new(SharedState::default());
    let jobs: Vec<_> = (0..3)
        .map(|tag| {
            let mut job = PoolBuildJob::new(tag, tag);
            job.order = order.clone();
            Handle::new(job)
        })
        .collect();
    signals.enabled.store(false, Ordering::Release);
    for job in [&jobs[0], &jobs[1], &jobs[2], &jobs[0]] {
        pool.add_entity(&job.clone().cast()).unwrap();
    }
    pool.wait_threads();
    assert_eq!(*order.read(), vec![0, 1, 2, 0]);
    assert_eq!(jobs[0].calls.load(Ordering::Acquire), 2);
    assert_eq!(
        *printer.messages.read(),
        vec![
            "Standard_Failure: fixture failure",
            "Error: Unknown exception"
        ]
    );
    pool.stop_threads();
    assert_eq!(*signals.applied.read(), vec![true]);
    assert_eq!(pool.threads().value(1).unwrap().id(), 0);
}

fn blocking_job(entered: &Arc<Barrier>, release: &Arc<Barrier>) -> Handle<PoolBuildJob> {
    let mut job = PoolBuildJob::new(7, 0);
    job.entered = Some(entered.clone());
    job.release = Some(release.clone());
    Handle::new(job)
}

#[test]
fn workers_build_simultaneously_and_hold_their_exposed_mutexes() {
    let (pool, _, _) = pool_fixture(2);
    let entered = Arc::new(Barrier::new(3));
    let release = Arc::new(Barrier::new(3));
    let jobs = [
        blocking_job(&entered, &release),
        blocking_job(&entered, &release),
    ];
    for job in &jobs {
        pool.add_entity(&job.clone().cast()).unwrap();
    }
    entered.wait();
    for worker in pool.threads().as_slice() {
        assert!(!worker.bvh_mutex().try_lock());
    }
    release.wait();
    pool.wait_threads();
    assert!(
        jobs.iter()
            .all(|job| job.calls.load(Ordering::Acquire) == 1)
    );
}

#[test]
fn wait_observes_active_jobs_even_when_an_idle_worker_empties_the_queue() {
    let (pool, _, _) = pool_fixture(2);
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    pool.add_entity(&blocking_job(&entered, &release).cast())
        .unwrap();
    entered.wait();
    pool.fields.idle.wait();
    let waiting = pool.clone();
    let start_wait = Arc::new(Barrier::new(2));
    let ready = start_wait.clone();
    let waiter = std::thread::spawn(move || {
        ready.wait();
        waiting.wait_threads();
    });
    start_wait.wait();
    assert!(!waiter.is_finished());
    release.wait();
    waiter.join().unwrap();
}

#[test]
fn stopping_retains_unstarted_jobs_and_an_add_restarts_the_pool() {
    let (pool, _, _) = pool_fixture(1);
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let queued = Handle::new(PoolBuildJob::new(9, 0));
    pool.add_entity(&blocking_job(&entered, &release).cast())
        .unwrap();
    entered.wait();
    pool.add_entity(&queued.clone().cast()).unwrap();
    let stopping = pool.clone();
    let stopper = std::thread::spawn(move || stopping.stop_threads());
    while !pool.fields.stop.load(Ordering::Acquire) {
        std::thread::yield_now();
    }
    release.wait();
    stopper.join().unwrap();
    assert_eq!(queued.calls.load(Ordering::Acquire), 0);
    assert!(!pool.fields.started.load(Ordering::Acquire));
    pool.add_entity(&queued.clone().cast()).unwrap();
    pool.wait_threads();
    assert_eq!(queued.calls.load(Ordering::Acquire), 2);
}

#[test]
fn sentry_locks_all_workers_and_releases_on_unlock_and_drop() {
    let (pool, _, _) = pool_fixture(2);
    let mut sentry = SelectMgrBvhThreadSentry::new(pool.clone());
    for worker in pool.threads().as_slice() {
        assert!(!worker.bvh_mutex().try_lock());
    }
    assert!(sentry.lock().is_err());
    sentry.unlock();
    for worker in pool.threads().as_slice() {
        assert!(worker.bvh_mutex().try_lock());
        worker.bvh_mutex().unlock().unwrap();
    }
    sentry.lock().unwrap();
    drop(sentry);
    for worker in pool.threads().as_slice() {
        assert!(worker.bvh_mutex().try_lock());
        worker.bvh_mutex().unlock().unwrap();
    }
    let mut null = SelectMgrBvhThreadSentry::new(Handle::default());
    null.lock().unwrap();
    null.unlock();
}

fn return_argument(argument: usize) -> usize {
    argument
}
#[test]
fn worker_copies_share_osd_runs_but_have_independent_mutexes() {
    let worker = SelectMgrBvhThread::new();
    assert!(!worker.run(&worker, 0));
    worker.set_function(return_argument);
    worker.bvh_mutex().lock();
    let copy = SelectMgrBvhThread::from_thread(&worker);
    assert!(copy.bvh_mutex().try_lock());
    copy.bvh_mutex().unlock().unwrap();
    assert!(worker.run(&copy, 0));
    let running = worker.clone();
    assert_eq!(
        running.wait_result(),
        Some(std::ptr::from_ref(&copy) as usize)
    );
    assert!(!worker.wait());
    copy.bvh_mutex().lock();
    copy.assign(&worker);
    assert!(!copy.bvh_mutex().try_lock());
    copy.bvh_mutex().unlock().unwrap();
    worker.bvh_mutex().unlock().unwrap();
}

#[test]
fn default_worker_replacement_and_custom_function_are_the_actual_public_threads() {
    let (pool, _, _) = pool_fixture(1);
    pool.threads()
        .change_value(1)
        .unwrap()
        .set_function(return_argument);
    let entity: Handle<dyn Select3DSensitiveEntity> = Handle::new(PoolBuildJob::new(0, 0)).cast();
    pool.add_entity(&entity).unwrap();
    pool.stop_threads();
    assert_eq!(pool.fields.queued.read().len(), 1);
    let (source, _, _) = pool_fixture(1);
    pool.threads()
        .change_value(1)
        .unwrap()
        .assign(source.threads().value(1).unwrap());
    // Assignment really copied the source pool binding, so its worker consumes that pool's queue.
    source.fields.queued.write().append(entity);
    source.fields.idle.reset();
    source.fields.wake.set();
    source.fields.started.store(true, Ordering::Release);
    {
        let workers = pool.threads();
        let worker = workers.value(1).unwrap();
        assert!(worker.run(worker, 0));
    }
    source.wait_threads();
    source.fields.stop.store(true, Ordering::Release);
    source.fields.wake.set();
    assert!(pool.threads().value(1).unwrap().wait());
}

#[test]
fn callbacks_can_inspect_workers_and_enqueue_more_work() {
    let (pool, _, _) = pool_fixture(1);
    let followup = Handle::new(PoolBuildJob::new(8, 0));
    let mut first = PoolBuildJob::new(7, 0);
    first.inspect_pool = pool.downgrade();
    first.followup = followup.clone().cast();
    pool.add_entity(&Handle::new(first).cast()).unwrap();
    pool.wait_threads();
    assert_eq!(followup.calls.load(Ordering::Acquire), 1);
}

#[test]
fn queued_entities_survive_caller_handles_and_pool_destruction_joins_workers() {
    let (pool, signals, _) = pool_fixture(1);
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let job = blocking_job(&entered, &release);
    let weak_job = job.downgrade();
    pool.add_entity(&job.clone().cast()).unwrap();
    drop(job);
    entered.wait();
    assert!(weak_job.upgrade().is_some());
    let weak_pool = pool.downgrade();
    let dropping = std::thread::spawn(move || drop(pool));
    release.wait();
    dropping.join().unwrap();
    assert!(weak_pool.upgrade().is_null());
    assert!(weak_job.upgrade().is_null());
    assert_eq!(*signals.applied.read(), vec![true]);
}

#[test]
fn shutdown_and_empty_queue_workers_keep_the_wake_across_restarts() {
    let (pool, _, _) = pool_fixture(4);
    let entity = Handle::new(PoolBuildJob::new(0, 0));
    for _ in 0..100 {
        pool.add_entity(&entity.clone().cast()).unwrap();
        pool.wait_threads();
        pool.stop_threads();
    }
    assert_eq!(entity.calls.load(Ordering::Acquire), 100);
}
