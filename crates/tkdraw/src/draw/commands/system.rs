//! The process commands of `Draw_BasicCommands.cxx`: `cpulimit`, `dversion`,
//! `dlocale`, `meminfo`, `dparallel`, `dperf`, `dsetsignal`.
use crate::draw::os_limits;
use crate::draw::{DrawInterpretor, DrawSession};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::osd::{
    Environment, MemCounter, MemInfo, Osd, Parallel, SignalMode, ThreadPool, Timer,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::version_info::occt_version_string_extended;
use std::fmt::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// The elapsed-time clock of `cpulimit` (`static OSD_Timer aTimer`).
static ELAPSED: Mutex<Option<Timer>> = Mutex::new(None);
/// `static int aFirst`: the watchdog thread is started by the first call only.
static WATCHDOG_STARTED: AtomicBool = AtomicBool::new(false);

/// `GetCpuLimit`: the seconds of `text`, times `CSF_CPULIMIT_FACTOR` when set.
fn cpu_limit_seconds(session: &DrawSession, text: &str) -> i64 {
    let mut seconds = i64::from(session.atoi(text));
    let factor = Environment::named("CSF_CPULIMIT_FACTOR")
        .map(|mut e| e.value().as_bytes().to_vec())
        .unwrap_or_default();
    if !factor.is_empty() {
        seconds *= i64::from(session.atoi(&String::from_utf8_lossy(&factor)));
    }
    seconds
}

/// Restarts the elapsed-time clock the watchdog reads.
fn restart_elapsed_clock() {
    let mut clock = ELAPSED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let timer = clock.get_or_insert_with(|| Timer::new(false));
    timer.reset();
    timer.start();
}

/// The watchdog: every 5 s, ends the process (status 2) once the elapsed time reached
/// a positive limit.
fn watch_elapsed_time() {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let elapsed = ELAPSED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .map_or(0.0, Timer::elapsed_time);
        let limit = os_limits::reported_cpu_limit();
        if limit > 0 && elapsed as i64 >= limit {
            println!("Process killed by elapsed limit  ({limit} sec)");
            std::process::exit(2);
        }
    }
}

/// `cpulimit [seconds]`: limits CPU time (`setrlimit`) and elapsed time (a watchdog
/// thread); without an argument removes the CPU limit.
pub(super) fn cpulimit(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let limit = a.get(1).map(|text| cpu_limit_seconds(session, text));
    // `(clock_t) RLIM_INFINITY` is -1.
    os_limits::set_reported_cpu_limit(limit.unwrap_or(-1));
    #[cfg(unix)]
    {
        let status = os_limits::set_cpu_rlimit(limit.map(|seconds| seconds as u64));
        if status != 0 {
            di.append(format!("status cpulimit setrlimit : {status}\n"))?;
        }
        os_limits::install_cpu_limit_handler();
    }
    restart_elapsed_clock();
    if !WATCHDOG_STARTED.swap(true, Ordering::SeqCst) {
        std::thread::spawn(watch_elapsed_time);
    }
    di.append("CPU and elapsed time limit set to ")?
        .append(os_limits::reported_cpu_limit() as f64)?
        .append(" seconds")?;
    Ok(0)
}

/// `dversion`: the version and the configuration of this build. The C++ reports its
/// compile-time macros; the Rust build reports the same items for itself.
pub(super) fn dversion(
    _session: &DrawSession,
    di: &DrawInterpretor,
    _a: &[&str],
) -> Result<i32, Failure> {
    let mut text = format!(
        "Open CASCADE Technology {}\n",
        occt_version_string_extended()
    );
    if cfg!(debug_assertions) {
        text.push_str("Debug mode\n");
    }
    text.push_str("Tk enabled (HAVE_TK)\nXlib disabled\nTBB disabled\nFreeType disabled\nFreeImage disabled\n");
    text.push_str(
        "FFmpeg disabled\nOpenVR disabled\nRapidJSON disabled\nDraco disabled\nVTK disabled\n",
    );
    text.push_str("Exceptions enabled\n");
    text.push_str(&format!(
        "Compiler: rustc {}\n",
        env!("CARGO_PKG_RUST_VERSION")
    ));
    text.push_str(&format!("Architecture: {}\n", architecture_name()));
    text.push_str(&format!("OS: {}\n", os_name()));
    di.append(text)?;
    Ok(0)
}

/// The architecture line of `dversion`.
fn architecture_name() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "AMD64",
        "x86" => "Intel x86",
        "aarch64" => "ARM 64-bit",
        "arm" => "ARM 32-bit",
        "sparc64" => "SPARC",
        "wasm32" => "WASM 32-bit",
        _ => "unrecognized",
    }
}

/// The OS line of `dversion`.
fn os_name() -> &'static str {
    match std::env::consts::OS {
        "windows" => "Windows",
        "macos" => "Mac OS X",
        "linux" => "Linux",
        "android" => "Android",
        "freebsd" | "openbsd" | "netbsd" | "dragonfly" => "BSD",
        _ => "unrecognized",
    }
}

/// `dlocale [category [locale]]`: queries or sets the C locale.
pub(super) fn dlocale(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let category = match a.get(1) {
        None => os_limits::locale_category("LC_ALL"),
        Some(name) => os_limits::locale_category(name),
    };
    let Some(category) = category else {
        let _ = write!(
            Message::send_fail(),
            "Error: cannot recognize argument {} as one of LC_ macros",
            a[1]
        );
        return Ok(1);
    };
    let locale = a.get(2).copied();
    match os_limits::set_locale(category, locale) {
        Some(result) => drop(di.append(result)?),
        None => session.console().write_out(&format!(
            "Error: unsupported locale specification: {}\n",
            locale.unwrap_or("(null)")
        )),
    }
    Ok(0)
}

/// The counter named by a `meminfo` argument.
fn mem_counter(name: &str) -> Option<MemCounter> {
    match name {
        "virt" | "v" => Some(MemCounter::Virtual),
        "heap" | "h" => Some(MemCounter::HeapUsage),
        "wset" | "w" => Some(MemCounter::WorkingSet),
        "wsetpeak" => Some(MemCounter::WorkingSetPeak),
        "swap" => Some(MemCounter::SwapUsage),
        "swappeak" => Some(MemCounter::SwapUsagePeak),
        "private" => Some(MemCounter::Private),
        _ => None,
    }
}

/// `meminfo [counters]`: the memory report, or the values of the named counters (each
/// once, in counter order, as the C++ map iterates).
pub(super) fn dmeminfo(
    _session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() <= 1 {
        di.append(String::from_utf8_lossy(MemInfo::new(true).to_text().as_bytes()).into_owned())?;
        return Ok(0);
    }
    let mut counters: Vec<MemCounter> = Vec::new();
    for argument in &a[1..] {
        let Some(counter) = mem_counter(&argument.to_ascii_lowercase()) else {
            di.append(format!("Syntax error at '{argument}'!\n"))?;
            return Ok(1);
        };
        counters.push(counter);
    }
    let mut info = MemInfo::new(false);
    info.set_active(false);
    let selected: Vec<MemCounter> = MemCounter::ALL
        .into_iter()
        .filter(|c| counters.contains(c))
        .collect();
    selected
        .iter()
        .for_each(|counter| info.set_counter_active(*counter, true));
    info.update();
    for counter in selected {
        di.append(info.value(counter) as f64)?.append(" ")?;
    }
    di.append("\n")?;
    Ok(0)
}

/// `dparallel [options]`: the default thread pool and the threading backend.
pub(super) fn dparallel(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let pool = ThreadPool::default_pool(-1);
    if a.len() <= 1 {
        let text = format!(
            "NbLogicalProcessors: {}\nNbThreads:           {}\nNbDefThreads:        {}\nUseOcct:             {}",
            Parallel::nb_logical_processors(),
            pool.nb_threads(),
            pool.nb_default_threads_to_launch(),
            i32::from(Parallel::to_use_occt_threads())
        );
        di.append(text)?;
        return Ok(0);
    }
    let mut index = 1;
    while index < a.len() {
        if !apply_parallel_option(session, pool, a, &mut index)? {
            return Ok(1);
        }
        index += 1;
    }
    Ok(0)
}

/// One `dparallel` option; false after a syntax error (already reported).
fn apply_parallel_option(
    session: &DrawSession,
    pool: &ThreadPool,
    a: &[&str],
    index: &mut usize,
) -> Result<bool, Failure> {
    let option = a[*index].to_ascii_lowercase();
    let has_value = *index + 1 < a.len();
    let value = || session.atoi(a[*index + 1]);
    match option.as_str() {
        "-nbthreads" | "-threads" if has_value => pool.init(value())?,
        "-nbdefthreads" | "-defthreads" | "-nbmaxdefthreads" | "-maxdefthreads" if has_value => {
            let count = value();
            if count <= 0 || count > pool.nb_threads() {
                let _ = write!(
                    Message::send_fail(),
                    "Syntax error: maximum number of threads to use should be <= of threads in the pool"
                );
                return Ok(false);
            }
            pool.set_nb_default_threads_to_launch(count);
        }
        "-useocct" | "-touseocct" | "-occt" if has_value => switch_threads(session, value() == 1),
        "-usetbb" | "-tousetbb" | "-tbb" if has_value => switch_threads(session, value() == 0),
        _ => {
            let _ = write!(
                Message::send_fail(),
                "Syntax error: unknown argument '{option}'"
            );
            return Ok(false);
        }
    }
    *index += 1;
    Ok(true)
}

/// `SetUseOcctThreads`, with the warning when the backend cannot be switched.
fn switch_threads(session: &DrawSession, use_occt: bool) {
    Parallel::set_use_occt_threads(use_occt);
    if Parallel::to_use_occt_threads() != use_occt {
        session
            .console()
            .write_out("Warning: unable to switch threads library - no options available\n");
    }
}

/// `dperf [reset]`: the performance meters of the session registry; any argument other
/// than `0` (or empty) resets them afterwards.
pub(super) fn dperf(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let reset = a
        .get(1)
        .is_some_and(|text| !text.is_empty() && !text.starts_with('0'));
    di.append(session.perf_registry().print_all())?;
    if reset {
        session.perf_registry().reset_all();
    }
    Ok(0)
}

/// The signal mode named by a `dsetsignal` argument.
fn signal_mode(name: &str) -> Option<SignalMode> {
    match name {
        "asis" => Some(SignalMode::AsIs),
        "set" => Some(SignalMode::Set),
        "unhandled" => Some(SignalMode::SetUnhandled),
        "unset" => Some(SignalMode::Unset),
        _ => None,
    }
}

/// `dsetsignal [mode] [0|1|default] [-strackTraceLength n]`: installs the OSD signal
/// handlers and reports the resulting state.
pub(super) fn dsetsignal(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let mut mode = SignalMode::Set;
    let fpe_default = Environment::named("CSF_FPE")
        .map(|mut e| e.value().as_bytes().to_vec())
        .unwrap_or_default();
    let mut catch_fpe = fpe_default
        .first()
        .map_or(Osd::to_catch_floating_signals(), |first| *first != b'0');
    let mut stack_length = Osd::signal_stack_trace_length();
    let mut index = 1;
    while index < a.len() {
        let option = a[index].to_ascii_lowercase();
        let has_value = index + 1 < a.len();
        match option.as_str() {
            "1" | "on" => catch_fpe = true,
            "0" | "off" => catch_fpe = false,
            "default" => {}
            "-stracktracelength" | "-stracktracelen" | "-stracklength" | "-stracklen"
                if has_value =>
            {
                index += 1;
                stack_length = session.atoi(a[index]);
            }
            _ => match signal_mode(&option) {
                Some(selected) => mode = selected,
                None => {
                    let _ = write!(
                        Message::send_fail(),
                        "Syntax error: unknown argument '{option}'"
                    );
                    return Ok(1);
                }
            },
        }
        index += 1;
    }
    Osd::set_signal(mode, catch_fpe);
    Osd::set_signal_stack_trace_length(stack_length);
    report_signals(di, stack_length)
}

/// The final report of `dsetsignal`.
fn report_signals(di: &DrawInterpretor, stack_length: i32) -> Result<i32, Failure> {
    let mode = match Osd::signal_mode() {
        SignalMode::Set => "set",
        SignalMode::SetUnhandled => "unhandled",
        SignalMode::Unset => "unset",
        SignalMode::AsIs => "asis",
    };
    let fpe = if Osd::to_catch_floating_signals() {
        "1"
    } else {
        "0"
    };
    di.append(format!(
        "Signal mode: {mode}\nCatch FPE: {fpe}\nStack Trace Length: {stack_length}\n"
    ))?;
    Ok(0)
}
