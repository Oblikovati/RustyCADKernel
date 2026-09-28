//! OSD threads probe operations (OSD_Thread, OSD_ThreadPool, OSD_Parallel); twin of
//! `tkernel_osd_threads_probe.cxx`.
use super::{bit, escape, integer};
use rustycad_tkernel::{
    osd::{Launcher, OsdThread, Parallel, ThreadPool},
    standard::Failure,
};
use std::{
    str::SplitWhitespace,
    sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicUsize, Ordering},
};

/// The output of a threads operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    Some(match op {
        "osd_pool" => pool_probe(input),
        "osd_launcher" => launcher_probe(input),
        "osd_perform" => perform_probe(input),
        "osd_perform_fail" => perform_fail_probe(input),
        "osd_for" => for_probe(input),
        "osd_foreach" => foreach_probe(input),
        "osd_nproc" => nproc_probe(),
        "osd_default_pool" => pool_sizes(ThreadPool::default_pool(-1)),
        "osd_thread" => thread_probe(input),
        "osd_thread_timed" => thread_timed_probe(input),
        "osd_thread_copy" => thread_copy_probe(input),
        _ => return None,
    })
}

fn int(input: &mut SplitWhitespace<'_>) -> i32 {
    integer(input) as i32
}

/// Visit counters of one loop (native `OsdThreadsCoverage`).
struct Coverage {
    begin: i32,
    slots: Vec<AtomicI32>,
    count: AtomicI32,
    sum: AtomicI64,
    outside: AtomicBool,
    bad_thread: AtomicBool,
}
impl Coverage {
    fn new(begin: i32, end: i32) -> Self {
        let size: usize = if end > begin {
            (end - begin) as usize
        } else {
            0
        };
        Self {
            begin,
            slots: (0..size).map(|_| AtomicI32::new(0)).collect(),
            count: AtomicI32::new(0),
            sum: AtomicI64::new(0),
            outside: AtomicBool::new(false),
            bad_thread: AtomicBool::new(false),
        }
    }
    fn visit(&self, index: i32) {
        self.count.fetch_add(1, Ordering::SeqCst);
        self.sum.fetch_add(i64::from(index), Ordering::SeqCst);
        let slot: i64 = i64::from(index) - i64::from(self.begin);
        match usize::try_from(slot)
            .ok()
            .and_then(|slot| self.slots.get(slot))
        {
            Some(counter) => {
                counter.fetch_add(1, Ordering::SeqCst);
            }
            None => self.outside.store(true, Ordering::SeqCst),
        }
    }
    fn thread(&self, thread: i32, lower: i32, upper: i32) {
        if thread < lower || thread > upper {
            self.bad_thread.store(true, Ordering::SeqCst);
        }
    }
    fn text(&self) -> String {
        let once: bool = !self.outside.load(Ordering::SeqCst)
            && self
                .slots
                .iter()
                .all(|slot| slot.load(Ordering::SeqCst) == 1);
        format!(
            "{} {} {} {}",
            self.count.load(Ordering::SeqCst),
            self.sum.load(Ordering::SeqCst),
            bit(once),
            bit(!self.bad_thread.load(Ordering::SeqCst))
        )
    }
}

fn pool_sizes(pool: &ThreadPool) -> String {
    format!(
        "{} {} {} {} {}",
        pool.nb_threads(),
        bit(pool.has_threads()),
        pool.lower_thread_index(),
        pool.upper_thread_index(),
        pool.nb_default_threads_to_launch()
    )
}

fn launcher_sizes(launcher: &Launcher<'_>) -> String {
    format!(
        "{} {} {} {}",
        launcher.nb_threads(),
        bit(launcher.has_threads()),
        launcher.lower_thread_index(),
        launcher.upper_thread_index()
    )
}

fn pool_probe(input: &mut SplitWhitespace<'_>) -> String {
    let (nb, init, default) = (int(input), int(input), int(input));
    let pool: ThreadPool = ThreadPool::new(nb);
    let before: String = pool_sizes(&pool);
    if default != -99 {
        pool.set_nb_default_threads_to_launch(default);
    }
    if init != 0 {
        pool.init(init).expect("an idle pool accepts Init");
    }
    format!("{before} | {} {}", pool_sizes(&pool), bit(pool.is_in_use()))
}

fn covered_perform(launcher: &Launcher<'_>, begin: i32, end: i32) -> Coverage {
    let coverage: Coverage = Coverage::new(begin, end);
    let (lower, upper) = (launcher.lower_thread_index(), launcher.upper_thread_index());
    launcher
        .perform(begin, end, |thread, index| {
            coverage.thread(thread, lower, upper);
            coverage.visit(index);
        })
        .expect("counting never fails");
    coverage
}

fn launcher_probe(input: &mut SplitWhitespace<'_>) -> String {
    let (nb, first_max, second_max) = (int(input), int(input), int(input));
    let pool: ThreadPool = ThreadPool::new(nb);
    let mut first: Launcher<'_> = Launcher::new(&pool, first_max);
    let mut out: String = format!("{} {} | ", launcher_sizes(&first), bit(pool.is_in_use()));
    let mut second: Launcher<'_> = Launcher::new(&pool, second_max);
    let coverage: Coverage = covered_perform(&second, 0, 100);
    out += &format!("{} {}", launcher_sizes(&second), coverage.text());
    first.release();
    out += &format!(" | {} {}", first.nb_threads(), bit(pool.is_in_use()));
    let empty: Coverage = covered_perform(&first, 0, 10);
    out += &format!(" {}", empty.count.load(Ordering::SeqCst));
    second.release();
    out + &format!(" {}", bit(pool.is_in_use()))
}

fn perform_probe(input: &mut SplitWhitespace<'_>) -> String {
    let (nb, max, begin, end) = (int(input), int(input), int(input), int(input));
    let pool: ThreadPool = ThreadPool::new(nb);
    let launcher: Launcher<'_> = Launcher::new(&pool, max);
    let coverage: Coverage = covered_perform(&launcher, begin, end);
    format!("{} {}", launcher_sizes(&launcher), coverage.text())
}

fn perform_once(launcher: &Launcher<'_>, begin: i32, end: i32, at: i32) -> String {
    // The failure is reported on stdout; keep the default panic report off stderr.
    let report = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result: Result<(), Failure> = launcher.perform(begin, end, |_, index| {
        if index == at {
            std::panic::panic_any(Failure::new("boom"));
        }
    });
    std::panic::set_hook(report);
    match result {
        Ok(()) => "ok".to_string(),
        Err(failure) => format!("{}{}", failure.exception_type(), escape(failure.message())),
    }
}

fn perform_fail_probe(input: &mut SplitWhitespace<'_>) -> String {
    let (nb, max, begin, end, at) = (int(input), int(input), int(input), int(input), int(input));
    let pool: ThreadPool = ThreadPool::new(nb);
    let launcher: Launcher<'_> = Launcher::new(&pool, max);
    let first: String = perform_once(&launcher, begin, end, at);
    if launcher.nb_threads() != 1 {
        return format!("{first} -");
    }
    format!("{first} {}", perform_once(&launcher, begin, end, end))
}

fn for_probe(input: &mut SplitWhitespace<'_>) -> String {
    let (begin, end, force) = (int(input), int(input), int(input));
    let coverage: Coverage = Coverage::new(begin, end);
    Parallel::for_range(begin, end, |index| coverage.visit(index), force != 0)
        .expect("counting never fails");
    coverage.text()
}

fn foreach_probe(input: &mut SplitWhitespace<'_>) -> String {
    let (count, nb_items, force) = (int(input), int(input), int(input));
    let values: Vec<i32> = (0..count).collect();
    let coverage: Coverage = Coverage::new(0, count);
    Parallel::for_each(
        values.iter(),
        |&value| coverage.visit(value),
        force != 0,
        nb_items,
    )
    .expect("counting never fails");
    coverage.text()
}

fn nproc_probe() -> String {
    let before: bool = Parallel::to_use_occt_threads();
    Parallel::set_use_occt_threads(false);
    let after: bool = Parallel::to_use_occt_threads();
    Parallel::set_use_occt_threads(true);
    format!(
        "{} {} {}",
        Parallel::nb_logical_processors(),
        bit(before),
        bit(after)
    )
}

fn double(data: usize) -> usize {
    data * 2
}

static SPIN_RELEASED: AtomicBool = AtomicBool::new(false);
static SPIN_CURRENT: AtomicUsize = AtomicUsize::new(0);
fn spin(data: usize) -> usize {
    SPIN_CURRENT.store(OsdThread::current(), Ordering::SeqCst);
    while !SPIN_RELEASED.load(Ordering::SeqCst) {
        std::hint::spin_loop();
    }
    data + 1
}

/// Native `Wait(void*&)`: the flag and the result, 0 when nothing was joined.
fn wait_text(thread: &mut OsdThread) -> String {
    let result: Option<usize> = thread.wait_result();
    format!(" {} {}", bit(result.is_some()), result.unwrap_or(0))
}

fn thread_probe(input: &mut SplitWhitespace<'_>) -> String {
    let value: usize = integer(input) as usize;
    let mut empty: OsdThread = OsdThread::new();
    let mut out: String = format!("{} {}", bit(empty.run(value, 0)), bit(empty.wait()));
    let timed: Option<usize> = empty.wait_for(5);
    out += &format!(" {} {}", bit(timed.is_some()), timed.unwrap_or(0));
    let mut thread: OsdThread = OsdThread::with_function(double);
    out += &format!(" {} {}", bit(thread.id() == 0), bit(thread.run(value, 0)));
    out += &format!(" {}", bit(thread.id() != 0));
    out += &wait_text(&mut thread);
    out += &format!(" {}", bit(thread.id() == 0));
    out += &wait_text(&mut thread);
    out + &format!(" {}", bit(OsdThread::current() != 0))
}

fn thread_timed_probe(input: &mut SplitWhitespace<'_>) -> String {
    let time: i32 = int(input);
    SPIN_RELEASED.store(false, Ordering::SeqCst);
    let mut thread: OsdThread = OsdThread::with_function(spin);
    let mut out: String = bit(thread.run(41, 0)).to_string();
    let id: usize = thread.id();
    let early: Option<usize> = thread.wait_for(time);
    out += &format!(" {} {}", bit(early.is_some()), early.unwrap_or(0));
    SPIN_RELEASED.store(true, Ordering::SeqCst);
    let late: Option<usize> = thread.wait_for(10000);
    out += &format!(" {} {}", bit(late.is_some()), late.unwrap_or(0));
    let same: bool = SPIN_CURRENT.load(Ordering::SeqCst) == id;
    out + &format!(" {} {}", bit(same), bit(thread.id() == 0))
}

fn thread_copy_probe(input: &mut SplitWhitespace<'_>) -> String {
    let value: usize = integer(input) as usize;
    let original: OsdThread = OsdThread::with_function(double);
    let mut copy: OsdThread = original.clone();
    let mut out: String = bit(copy.run(value, 0)).to_string();
    out += &wait_text(&mut copy);
    let mut assigned: OsdThread = OsdThread::new();
    assigned.clone_from(&original);
    out += &format!(" {}", bit(assigned.run(value + 1, 0)));
    out += &wait_text(&mut assigned);
    assigned.set_function(double);
    assigned.set_priority(0);
    out += &format!(" {}", bit(assigned.run(value + 2, 65536)));
    out + &wait_text(&mut assigned)
}
