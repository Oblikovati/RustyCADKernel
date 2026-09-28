//! OSD_Thread, OSD_ThreadPool, OSD_ThreadPool::Launcher and OSD_Parallel: every public
//! member, with the native quirks each keeps.
use rustycad_tkernel::{
    osd::{Launcher, OsdThread, Parallel, ThreadPool},
    standard::Failure,
};
use std::{
    collections::BTreeSet,
    sync::{
        Barrier, Mutex,
        atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering},
    },
    thread::ThreadId,
};

#[test]
fn pool_sizes_follow_the_requested_participants() {
    let pool: ThreadPool = ThreadPool::new(3);
    assert_eq!(pool.nb_threads(), 3);
    assert!(pool.has_threads());
    assert_eq!(
        (pool.lower_thread_index(), pool.upper_thread_index()),
        (0, 2)
    );
    assert_eq!(pool.nb_default_threads_to_launch(), 3);
    let single: ThreadPool = ThreadPool::new(1);
    assert!(!single.has_threads());
    assert_eq!(single.upper_thread_index(), 0);
    assert_eq!(
        ThreadPool::new(0).nb_threads(),
        Parallel::nb_logical_processors().max(1)
    );
}

#[test]
fn init_resizes_but_keeps_the_default_launch_size() {
    let pool: ThreadPool = ThreadPool::new(4);
    pool.set_nb_default_threads_to_launch(7);
    pool.init(2).unwrap();
    assert_eq!(pool.nb_threads(), 2);
    assert_eq!(
        pool.nb_default_threads_to_launch(),
        7,
        "native Init keeps it"
    );
    pool.init(2).unwrap();
    assert_eq!(pool.nb_threads(), 2);
}

#[test]
fn launchers_lock_idle_workers_and_fall_back_to_the_caller() {
    let pool: ThreadPool = ThreadPool::new(4);
    assert!(!pool.is_in_use());
    let mut first: Launcher<'_> = Launcher::new(&pool, -1);
    assert_eq!((first.nb_threads(), first.upper_thread_index()), (4, 3));
    assert!(first.has_threads());
    assert!(pool.is_in_use());
    let busy: Launcher<'_> = Launcher::new(&pool, -1);
    assert_eq!((busy.nb_threads(), busy.lower_thread_index()), (1, 0));
    let seen: Mutex<BTreeSet<i32>> = Mutex::new(BTreeSet::new());
    busy.perform(0, 50, |thread, _| {
        seen.lock().unwrap().insert(thread);
    })
    .unwrap();
    assert_eq!(*seen.lock().unwrap(), BTreeSet::from([0]));
    let refused: Failure = pool.init(2).unwrap_err();
    assert_eq!(refused.exception_type(), "Standard_ProgramError");
    assert_eq!(
        refused.message(),
        "Error: active ThreadPool is reinitialized"
    );
    first.release();
    assert_eq!(first.nb_threads(), 0);
    assert!(!pool.is_in_use());
    let calls: AtomicI32 = AtomicI32::new(0);
    first
        .perform(0, 10, |_, _| {
            calls.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "a released launcher runs nothing"
    );
}

#[test]
fn launcher_size_arguments_follow_the_native_rules() {
    let pool: ThreadPool = ThreadPool::new(4);
    assert_eq!(Launcher::new(&pool, 0).nb_threads(), 1);
    assert_eq!(Launcher::new(&pool, 2).nb_threads(), 2);
    assert_eq!(Launcher::new(&pool, 10).nb_threads(), 4);
    pool.set_nb_default_threads_to_launch(0);
    assert_eq!(Launcher::new(&pool, -1).nb_threads(), 1);
}

#[test]
fn perform_visits_each_index_once_with_the_caller_last() {
    let pool: ThreadPool = ThreadPool::new(3);
    let launcher: Launcher<'_> = Launcher::new(&pool, -1);
    let caller: ThreadId = std::thread::current().id();
    let visits: Vec<AtomicI32> = (0..1000).map(|_| AtomicI32::new(0)).collect();
    let caller_indices: Mutex<BTreeSet<i32>> = Mutex::new(BTreeSet::new());
    launcher
        .perform(-500, 500, |thread, index| {
            assert!((0..=2).contains(&thread));
            if std::thread::current().id() == caller {
                caller_indices.lock().unwrap().insert(thread);
            }
            visits[(index + 500) as usize].fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();
    assert!(visits.iter().all(|count| count.load(Ordering::SeqCst) == 1));
    assert!(
        caller_indices
            .lock()
            .unwrap()
            .iter()
            .all(|&thread| thread == 2)
    );
    launcher
        .perform(5, 3, |_, _| panic!("empty range"))
        .unwrap();
}

#[test]
fn a_failing_functor_becomes_a_program_error() {
    let pool: ThreadPool = ThreadPool::new(1);
    let launcher: Launcher<'_> = Launcher::new(&pool, -1);
    let failure: Failure = launcher
        .perform(0, 4, |_, index| {
            if index == 2 {
                std::panic::panic_any(Failure::new("boom"));
            }
        })
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert_eq!(failure.message(), "Standard_Failure: boom");
    let again: Failure = launcher.perform(0, 4, |_, _| {}).unwrap_err();
    assert_eq!(
        again.message(),
        "Standard_Failure: boom",
        "self-thread keeps it"
    );
    let fresh: Launcher<'_> = Launcher::new(&pool, -1);
    let text: Failure = fresh.perform(0, 1, |_, _| panic!("text")).unwrap_err();
    assert_eq!(text.message(), "panic: text");
    let other: Failure = Launcher::new(&pool, -1)
        .perform(0, 1, |_, _| std::panic::panic_any(7_u8))
        .unwrap_err();
    assert_eq!(other.message(), "Error: Unknown exception");
}

#[test]
fn failures_on_several_threads_are_joined() {
    let pool: ThreadPool = ThreadPool::new(3);
    let launcher: Launcher<'_> = Launcher::new(&pool, -1);
    assert_eq!(launcher.nb_threads(), 3);
    // Each thread blocks on the barrier after taking one index, so all three fail.
    let barrier: Barrier = Barrier::new(3);
    let failure: Failure = launcher
        .perform(0, 3, |_, _| {
            barrier.wait();
            std::panic::panic_any(Failure::new("each"));
        })
        .unwrap_err();
    let each: &str = "Standard_Failure: each";
    assert_eq!(
        failure.message(),
        format!("Multiple exceptions:\n{each}\n{each}\n{each}")
    );
}

#[test]
fn parallel_loops_cover_their_range_and_items() {
    assert!(Parallel::to_use_occt_threads());
    Parallel::set_use_occt_threads(false);
    assert!(Parallel::to_use_occt_threads(), "ignored without TBB");
    assert!(Parallel::nb_logical_processors() >= 1);
    let total: AtomicI32 = AtomicI32::new(0);
    Parallel::for_range(
        -3,
        97,
        |index| {
            total.fetch_add(index, Ordering::SeqCst);
        },
        false,
    )
    .unwrap();
    assert_eq!(total.load(Ordering::SeqCst), (-3..97).sum::<i32>());
    let order: Mutex<Vec<i32>> = Mutex::new(Vec::new());
    Parallel::for_range(0, 5, |index| order.lock().unwrap().push(index), true).unwrap();
    assert_eq!(*order.lock().unwrap(), vec![0, 1, 2, 3, 4]);
    let mut values: Vec<i32> = (0..300).collect();
    Parallel::for_each(values.iter_mut(), |value| *value *= 2, false, -1).unwrap();
    assert_eq!(values, (0..300).map(|v| v * 2).collect::<Vec<i32>>());
    let order: Mutex<Vec<i32>> = Mutex::new(Vec::new());
    Parallel::for_each(0..4, |item| order.lock().unwrap().push(item), false, 1).unwrap();
    assert_eq!(*order.lock().unwrap(), vec![0, 1, 2, 3]);
}

#[test]
fn parallel_for_single_index_runs_in_the_caller() {
    let caller: ThreadId = std::thread::current().id();
    Parallel::for_range(
        8,
        9,
        |index| {
            assert_eq!(index, 8);
            assert_eq!(std::thread::current().id(), caller);
        },
        false,
    )
    .unwrap();
}

fn double(data: usize) -> usize {
    data * 2
}

static SPIN_RELEASED: AtomicBool = AtomicBool::new(false);
static SPIN_CURRENT: AtomicUsize = AtomicUsize::new(0);
fn spin_until_released(data: usize) -> usize {
    SPIN_CURRENT.store(OsdThread::current(), Ordering::SeqCst);
    while !SPIN_RELEASED.load(Ordering::SeqCst) {
        std::thread::yield_now();
    }
    data + 1
}

#[test]
fn thread_runs_waits_and_reports_its_identifier() {
    let mut idle: OsdThread = OsdThread::new();
    assert!(!idle.run(1, 0));
    assert!(!idle.wait());
    assert_eq!(idle.wait_for(5), None);
    let mut thread: OsdThread = OsdThread::with_function(double);
    assert_eq!(thread.id(), 0);
    assert!(thread.run(21, 0));
    assert_ne!(thread.id(), 0);
    assert_eq!(thread.wait_result(), Some(42));
    assert_eq!(thread.id(), 0);
    assert_eq!(thread.wait_result(), None);
    thread.set_priority(3);
    assert!(thread.run(1, 4096));
    assert!(thread.wait());
    assert_ne!(OsdThread::current(), 0);
}

// macOS natively ignores the timeout and joins, which would block on the spinning thread.
#[cfg(not(target_vendor = "apple"))]
#[test]
fn timed_wait_times_out_then_joins() {
    let mut thread: OsdThread = OsdThread::with_function(spin_until_released);
    assert!(thread.run(9, 0));
    let id: usize = thread.id();
    assert_eq!(thread.wait_for(10), None);
    assert_eq!(thread.wait_for(-5), None);
    SPIN_RELEASED.store(true, Ordering::SeqCst);
    assert_eq!(thread.wait_for(10_000), Some(10));
    assert_eq!(
        SPIN_CURRENT.load(Ordering::SeqCst),
        id,
        "Current() inside == GetId()"
    );
}

#[test]
fn copies_share_one_thread_and_detach_it_for_all() {
    let mut thread: OsdThread = OsdThread::with_function(double);
    assert!(thread.run(5, 0));
    let mut copy: OsdThread = thread.clone();
    assert_eq!(copy.id(), thread.id());
    assert_eq!(copy.wait_result(), Some(10));
    assert_eq!(thread.wait_result(), None, "joined through the copy");
    assert!(thread.run(6, 0));
    drop(thread.clone());
    assert!(
        !thread.wait(),
        "destroying a copy detached the thread (native quirk)"
    );
    let mut assigned: OsdThread = OsdThread::new();
    assigned.clone_from(&thread);
    assert!(assigned.run(4, 0), "the function was copied");
    assert_eq!(assigned.wait_result(), Some(8));
    assigned.set_function(spin_until_released);
    assert_eq!(assigned.id(), 0);
}

fn panicking(_data: usize) -> usize {
    panic!("thread body")
}

#[test]
fn a_panicking_thread_function_makes_wait_fail() {
    let mut thread: OsdThread = OsdThread::with_function(panicking);
    assert!(thread.run(0, 0));
    assert_eq!(thread.wait_for(10_000), None);
    assert!(!thread.wait(), "the failed join consumed the handle");
}
