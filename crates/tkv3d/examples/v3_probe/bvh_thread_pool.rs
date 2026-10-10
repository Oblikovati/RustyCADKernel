//! Same completed-round observations as the native background-pool probe.
#[path = "../../tests/support/bvh_pool_fixture.rs"]
mod fixture;
use super::common::{Reader, end, flag, int, text};
use fixture::{PoolBuildJob, pool_fixture};
use rustycad_tkernel::standard::Handle;
use rustycad_tkv3d::select_mgr::{SelectMgrBvhThreadPool, SelectMgrBvhThreadSentry};
use std::sync::{Arc, Barrier, atomic::Ordering};
fn pool_state(pool: &SelectMgrBvhThreadPool, jobs: &[Handle<PoolBuildJob>]) {
    let workers = pool.threads();
    int(workers.lower());
    int(workers.upper());
    for job in jobs {
        int(job.calls.load(Ordering::Acquire) as i32);
    }
    for worker in workers.as_slice() {
        flag(worker.id() != 0);
    }
}
fn pool_round(pool: &Handle<SelectMgrBvhThreadPool>, jobs: &[Handle<PoolBuildJob>], round: usize) {
    {
        let _sentry = SelectMgrBvhThreadSentry::new(pool.clone());
        let observed = pool.clone();
        let acquired = std::thread::spawn(move || {
            observed
                .threads()
                .as_slice()
                .iter()
                .map(|worker| {
                    let ok = worker.bvh_mutex().try_lock();
                    if ok {
                        worker.bvh_mutex().unlock().unwrap();
                    }
                    ok
                })
                .collect::<Vec<bool>>()
        })
        .join()
        .unwrap();
        for ok in acquired {
            flag(ok);
        }
    }
    for index in 0..jobs.len() * 2 {
        pool.add_entity(&jobs[(index + round) % jobs.len()].clone().cast())
            .unwrap();
    }
    pool.wait_threads();
    stop_with_parked_workers(pool, jobs);
    pool_state(pool, jobs);
}
fn stop_with_parked_workers(pool: &Handle<SelectMgrBvhThreadPool>, jobs: &[Handle<PoolBuildJob>]) {
    if !pool.fields.started.load(Ordering::Acquire) {
        pool_state(pool, jobs);
        pool.stop_threads();
        return;
    }
    let count = pool.threads().len();
    let entered = Arc::new(Barrier::new(count + 1));
    let release = Arc::new(Barrier::new(count + 1));
    let mut parked = PoolBuildJob::new(-1, 0);
    parked.entered = Some(entered.clone());
    parked.release = Some(release.clone());
    let parked = Handle::new(parked).cast();
    for _ in 0..count {
        pool.add_entity(&parked).unwrap();
    }
    entered.wait();
    pool_state(pool, jobs);
    pool.fields.wake.reset();
    let stopping = pool.clone();
    let stopper = std::thread::spawn(move || stopping.stop_threads());
    pool.fields.wake.wait();
    release.wait();
    stopper.join().unwrap();
}
pub fn bvh_thread_pool_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let (threads, count, rounds, seed) = (
        reader.integer(),
        reader.integer(),
        reader.integer(),
        reader.integer(),
    );
    let (pool, _, printer) = pool_fixture(threads);
    let jobs: Vec<_> = (0..count)
        .map(|index| {
            let job = Handle::new(PoolBuildJob::new(index, (index + seed) % 3));
            job.buildable
                .store((index + seed) % 4 != 0, Ordering::Release);
            job
        })
        .collect();
    pool_state(&pool, &jobs);
    for round in 0..rounds {
        pool_round(&pool, &jobs, round as usize);
    }
    let mut messages = printer.messages.snapshot();
    messages.sort();
    int(messages.len() as i32);
    for message in messages {
        text(message.as_bytes());
    }
    end();
}
