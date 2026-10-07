// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_Timer (.hxx, .lxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MoniTool_Timer`: an `OSD_Timer` as a handle, with a hit counter and a nesting
//! counter, and a process-wide dictionary of timers accessed by name.
//!
//! The private `AmendAccess`, `AmendStart` and `AmendStop` keep a list of the running
//! timers to subtract the cost of the timer calls; the native code never calls them
//! (their calls are commented out in `Start`, `Stop` and `Timer(name)`), so the port has
//! no such list and `Amend` stays 0 as natively.
use super::MoniToolTimerSentry;
use crate::ostream_text::ostream_real;
use rustycad_tkernel::{
    impl_transient,
    ncollection::DataMap,
    osd::Timer as OsdTimer,
    standard::{Handle, SharedState, StandardTransient},
    tcollection::AsciiString,
};
use std::io::{self, Write};
use std::sync::{OnceLock, PoisonError, RwLock, RwLockWriteGuard};

/// The fields of a timer.
#[derive(Clone, Default)]
struct TimerState {
    timer: OsdTimer,
    count: i32,
    nesting: i32,
    amend: f64,
}

/// `MoniTool_Timer`; `MoniToolTimer::timer_named("read")` gives the timer of a name.
///
/// ```
/// use rustycad_tkxsbase::moni_tool::MoniToolTimer;
/// let timer = MoniToolTimer::new();
/// timer.start();
/// timer.start();
/// assert_eq!(timer.is_running(), 2);
/// timer.stop();
/// timer.stop();
/// assert_eq!((timer.count(), timer.is_running()), (2, 0));
/// ```
#[derive(Default)]
pub struct MoniToolTimer {
    transient: StandardTransient,
    state: SharedState<TimerState>,
}
impl_transient!(
    MoniToolTimer,
    "MoniTool_Timer",
    StandardTransient,
    transient
);

/// The amendments `ComputeAmendments` measures (`GetAmendments`): access, internal,
/// external and the error of 10 nested timers.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MoniToolTimerAmendments {
    /// `amAccess`.
    pub access: f64,
    /// `amInternal`.
    pub internal: f64,
    /// `amExternal`.
    pub external: f64,
    /// `amError`.
    pub error10: f64,
}

/// The native `MoniTool_Timer::Dictionary` map; the keys are the names (natively the
/// caller's C strings, hashed by content).
type TimerDictionary = DataMap<AsciiString, Handle<MoniToolTimer>>;

fn dictionary_lock() -> &'static RwLock<TimerDictionary> {
    static DICTIONARY: OnceLock<RwLock<TimerDictionary>> = OnceLock::new();
    DICTIONARY.get_or_init(|| RwLock::new(TimerDictionary::default()))
}

fn amendments_lock() -> &'static RwLock<MoniToolTimerAmendments> {
    static AMENDMENTS: OnceLock<RwLock<MoniToolTimerAmendments>> = OnceLock::new();
    AMENDMENTS.get_or_init(|| RwLock::new(MoniToolTimerAmendments::default()))
}

impl MoniToolTimer {
    /// `MoniTool_Timer()`: stopped, no hit.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Timer() const`: a copy of the embedded `OSD_Timer`.
    pub fn timer(&self) -> OsdTimer {
        self.state.read().timer.clone()
    }
    /// `Timer()`: edits the embedded `OSD_Timer` (natively a reference to it).
    pub fn change_timer<R>(&self, edit: impl FnOnce(&mut OsdTimer) -> R) -> R {
        edit(&mut self.state.write().timer)
    }
    /// `Start`: starts the `OSD_Timer` when not nested, counts a hit.
    pub fn start(&self) {
        let mut state = self.state.write();
        if state.nesting == 0 {
            state.timer.start();
        }
        state.count += 1;
        state.nesting += 1;
    }
    /// `Stop`: stops the `OSD_Timer` when the outermost start is closed.
    pub fn stop(&self) {
        let mut state = self.state.write();
        state.nesting -= 1;
        if state.nesting == 0 {
            state.timer.stop();
        }
    }
    /// `Reset`: resets the `OSD_Timer` and the counters.
    pub fn reset(&self) {
        let mut state = self.state.write();
        state.timer.reset();
        state.count = 0;
        state.nesting = 0;
        state.amend = 0.0;
    }
    /// `Count`: the number of starts.
    pub fn count(&self) -> i32 {
        self.state.read().count
    }
    /// `IsRunning`: the nesting counter.
    pub fn is_running(&self) -> i32 {
        self.state.read().nesting
    }
    /// `CPU`: the user CPU time minus the amendment.
    pub fn cpu(&self) -> f64 {
        let state = self.state.read();
        let (_, _, _, cpu) = state.timer.show_values();
        cpu - state.amend
    }
    /// `Amend`: the accumulated amendment.
    pub fn amend(&self) -> f64 {
        self.state.read().amend
    }
    /// `Dump`: one line with the elapsed, user and system times and the hits.
    pub fn dump(&self, ostr: &mut dyn Write) -> io::Result<()> {
        let state = self.state.read();
        let (seconds, minutes, hours, _) = state.timer.show_values();
        let (user, system) = state.timer.show_cpu();
        let elapsed = seconds + f64::from(minutes * 60) + f64::from(hours * 3600);
        writeln!(
            ostr,
            "Elapsed:{elapsed:6.1} sec, CPU User:{user:9.4} sec, CPU Sys:{system:9.4} sec, hits: {}",
            state.count
        )
    }
    /// `Timer(name)`: the timer of `name` in the dictionary, created (reset) when absent.
    pub fn timer_named(name: &str) -> Handle<MoniToolTimer> {
        let key = AsciiString::new(name);
        let mut dic = Self::dictionary();
        if let Some(found) = dic.seek(&key) {
            return found.clone();
        }
        let created = Handle::new(MoniToolTimer::new());
        created.change_timer(OsdTimer::reset);
        dic.bind(key, created.clone());
        created
    }
    /// `Start(name)`: starts the timer of `name`.
    pub fn start_named(name: &str) {
        Self::timer_named(name).start();
    }
    /// `Stop(name)`: stops the timer of `name`.
    pub fn stop_named(name: &str) {
        Self::timer_named(name).stop();
    }
    /// `Dictionary`: the process-wide map of timers (a write guard to drop before any
    /// other call by name).
    pub fn dictionary() -> RwLockWriteGuard<'static, TimerDictionary> {
        dictionary_lock()
            .write()
            .unwrap_or_else(PoisonError::into_inner)
    }
    /// `ClearTimers`.
    pub fn clear_timers() {
        Self::dictionary().clear(false);
    }
    /// `DumpTimers`: the timers in name order; a running timer is reported on stderr.
    pub fn dump_timers(ostr: &mut dyn Write) -> io::Result<()> {
        let mut names: Vec<AsciiString> =
            Self::dictionary().items().map(|(k, _)| k.clone()).collect();
        names.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        writeln!(ostr, "DUMP OF TIMERS:")?;
        for name in names {
            let text = String::from_utf8_lossy(name.as_bytes()).into_owned();
            write!(ostr, "TIMER: {text:<20}\t")?;
            let timer = Self::timer_named(&text);
            timer.dump(ostr)?;
            if timer.is_running() != 0 {
                eprintln!("Warning: timer {text} is running");
            }
        }
        Ok(())
    }
    /// `ComputeAmendments`: measures the cost of the timer calls over 100000 tests and
    /// prints the CPU times on standard output.
    pub fn compute_amendments() {
        let mut out = io::stdout();
        // A failed write on stdout is ignored, as a native `std::cout` failure is.
        let _ = compute_amendments_with(100_000, &mut out);
    }
    /// `GetAmendments`: the amendments the last `ComputeAmendments` accumulated.
    pub fn get_amendments() -> MoniToolTimerAmendments {
        *amendments_lock()
            .read()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// The reference loop of `ComputeAmendments`: 100 square roots (kept from the
/// optimiser, as the native `[[maybe_unused]]` values may not be).
fn amendment_work(i: i32) {
    for k in 1..=100 {
        std::hint::black_box(f64::from(i + k).sqrt());
    }
}

/// The CPU times of the four amendment tests: reference, direct, sentry and by name.
fn amendment_tests(nbtests: i32) -> [Handle<MoniToolTimer>; 4] {
    let timers = [
        "_mt_amend_0_",
        "_mt_amend_1_",
        "_mt_amend_2_",
        "_mt_amend_3_",
    ]
    .map(MoniToolTimer::timer_named);
    timers.iter().for_each(|t| (**t).reset());
    for name in ["_mt_amend_t1_", "_mt_amend_t2_", "_mt_amend_t3_"] {
        (*MoniToolTimer::timer_named(name)).reset();
    }
    run_timed(&timers[0], nbtests, amendment_work);
    let direct = MoniToolTimer::timer_named("_mt_amend_t1_");
    run_timed(&timers[1], nbtests, |i| {
        direct.start();
        amendment_work(i);
        direct.stop();
    });
    run_timed(&timers[2], nbtests, |i| {
        let _sentry = MoniToolTimerSentry::from_name("_mt_amend_t2_");
        amendment_work(i);
    });
    run_timed(&timers[3], nbtests, |i| {
        MoniToolTimer::start_named("_mt_amend_t3_");
        amendment_work(i);
        MoniToolTimer::stop_named("_mt_amend_t3_");
    });
    timers
}

fn run_timed(timer: &Handle<MoniToolTimer>, nbtests: i32, mut work: impl FnMut(i32)) {
    timer.start();
    (1..=nbtests).for_each(&mut work);
    timer.stop();
}

/// `ComputeAmendments` over `nbtests` tests (100000 natively).
fn compute_amendments_with(nbtests: i32, out: &mut dyn Write) -> io::Result<()> {
    let timers = amendment_tests(nbtests);
    let cpu = timers.each_ref().map(|t| t.cpu());
    let inner = ["_mt_amend_t1_", "_mt_amend_t2_", "_mt_amend_t3_"]
        .map(|n| MoniToolTimer::timer_named(n).cpu());
    let tests = f64::from(nbtests);
    let report = {
        let mut am = amendments_lock()
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        am.external += (cpu[1] - cpu[0]) / tests;
        am.internal += (inner[0] - cpu[0]) / tests;
        am.access += (0.5 * (cpu[3] - cpu[1])) / tests;
        am.error10 = (cpu[1] + cpu[3] - 2.0 * cpu[2]).abs() / tests;
        *am
    };
    let [c0, c1, c2, c3] = cpu.map(ostream_real);
    let [t1, t2, t3] = inner.map(ostream_real);
    writeln!(out, "CPU 0: {c0}\nCPU 1: {c1} INTERNAL: {t1}")?;
    writeln!(
        out,
        "CPU 2: {c2} INTERNAL: {t2}\nCPU 3: {c3} INTERNAL: {t3}"
    )?;
    writeln!(
        out,
        "Access: {}, External: {}, Internal: {}, Error: {}",
        ostream_real(report.access),
        ostream_real(report.external),
        ostream_real(report.internal),
        ostream_real(report.error10)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nesting_counts_and_reset() {
        let timer = MoniToolTimer::new();
        timer.start();
        assert_eq!(timer.is_running(), 1);
        timer.stop();
        assert_eq!(timer.count(), 1);
        timer.reset();
        assert_eq!(
            (timer.count(), timer.is_running(), timer.amend()),
            (0, 0, 0.0)
        );
        assert!(timer.cpu() >= 0.0);
        assert!(!timer.timer().is_started());
    }

    #[test]
    fn dictionary_and_dump() {
        let first = MoniToolTimer::timer_named("_test_dump_b");
        let again = MoniToolTimer::timer_named("_test_dump_b");
        assert!(first == again);
        MoniToolTimer::start_named("_test_dump_a");
        MoniToolTimer::stop_named("_test_dump_a");
        let mut text = Vec::new();
        MoniToolTimer::dump_timers(&mut text).unwrap();
        let text = String::from_utf8(text).unwrap();
        assert!(text.starts_with("DUMP OF TIMERS:\n"));
        let a = text.find("TIMER: _test_dump_a").unwrap();
        let b = text.find("TIMER: _test_dump_b").unwrap();
        assert!(a < b);
        assert!(text.contains("hits: 1\n"));
    }

    #[test]
    fn dump_line_format() {
        let mut line = Vec::new();
        MoniToolTimer::new().dump(&mut line).unwrap();
        let line = String::from_utf8(line).unwrap();
        assert_eq!(
            line,
            "Elapsed:   0.0 sec, CPU User:   0.0000 sec, CPU Sys:   0.0000 sec, hits: 0\n"
        );
    }

    #[test]
    fn amendments_accumulate() {
        let mut out = Vec::new();
        compute_amendments_with(10, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("CPU 0: "));
        assert!(text.contains("Access: "));
        assert_eq!(MoniToolTimer::timer_named("_mt_amend_t3_").count(), 10);
        assert!(MoniToolTimer::get_amendments().error10 >= 0.0);
    }
}
