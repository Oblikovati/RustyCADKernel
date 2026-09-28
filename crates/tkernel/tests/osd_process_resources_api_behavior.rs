//! OSD_Environment, OSD_SharedLibrary and OSD_MemInfo: every public member, through named
//! fake sources where the answer depends on the host, plus native smoke checks.
// Most cases exercise the Linux native profile; other hosts compile the shared fakes only.
#![cfg_attr(not(target_os = "linux"), allow(unused_imports, dead_code))]
use rustycad_tkernel::osd::{
    Environment, EnvironmentSource, LoadMode, MemCounter, MemInfo, MemInfoSource, OsdErrorAccess,
    SharedLibrary, WhoAmI,
};
use std::{cell::RefCell, collections::BTreeMap};

/// An in-memory environment; `refuse` makes every definition fail with that errno.
#[derive(Default)]
struct FakeEnvironment {
    variables: RefCell<BTreeMap<Vec<u8>, Vec<u8>>>,
    refuse: Option<i32>,
}
impl EnvironmentSource for FakeEnvironment {
    fn get(&self, name: &[u8]) -> Option<Vec<u8>> {
        self.variables.borrow().get(name).cloned()
    }
    fn put(&self, name: &[u8], value: &[u8]) -> Result<(), i32> {
        if let Some(errno) = self.refuse {
            return Err(errno);
        }
        self.variables
            .borrow_mut()
            .insert(name.to_vec(), value.to_vec());
        Ok(())
    }
}

#[cfg(unix)]
#[test]
fn environment_builds_reads_and_removes_to_empty_on_unix() {
    let mut env =
        Environment::with_value_in(FakeEnvironment::default(), "CAD_HOME", "/opt/$v").unwrap();
    assert_eq!(env.name().as_bytes(), b"CAD_HOME");
    env.build();
    assert_eq!(env.value().as_bytes(), b"/opt/$v");
    env.set_value("/srv").unwrap();
    env.build();
    assert_eq!(env.value().as_bytes(), b"/srv");
    env.remove();
    assert_eq!(
        env.value().as_bytes(),
        b"",
        "Unix Remove defines the variable as empty"
    );
    env.set_name("UNSET").unwrap();
    assert!(env.value().is_empty());
    assert!(!env.failed());
}

#[cfg(unix)]
#[test]
fn environment_refuses_bad_spellings_with_construction_errors() {
    let failure = Environment::named_in(FakeEnvironment::default(), "A$B")
        .err()
        .unwrap();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    assert_eq!(
        failure.message(),
        "OSD_Environment::OSD_Environment: bad argument"
    );
    assert!(Environment::with_value_in(FakeEnvironment::default(), "A", "tab\there").is_err());
    let mut env = Environment::bare(FakeEnvironment::default());
    assert!(env.name().is_empty());
    let failure = env.set_value("$x").unwrap_err();
    assert_eq!(failure.message(), "OSD_Environment::Change: bad argument");
    let failure = env.set_name("é").unwrap_err();
    assert_eq!(failure.message(), "OSD_Environment::SetName: bad argument");
}

#[cfg(unix)]
#[test]
fn environment_build_failure_records_set_environment_and_set_name_resets_it() {
    let source = FakeEnvironment {
        refuse: Some(libc::ENOMEM),
        ..FakeEnvironment::default()
    };
    let mut env = Environment::with_value_in(source, "X", "1").unwrap();
    env.build();
    assert!(env.failed());
    assert_eq!(env.osd_error().message().as_bytes(), b"Set Environment");
    assert_eq!(env.osd_error().who(), Some(WhoAmI::Environment));
    let report = env.perror().unwrap_err();
    assert_eq!(report.message(), " :\n Not enough memory.\n\n");
    assert_eq!(env.error(), 10);
    env.set_name("Y").unwrap();
    assert!(!env.failed(), "Unix SetName resets the error first");
}

#[cfg(unix)]
#[test]
fn native_environment_round_trips_through_putenv() {
    let mut env = Environment::with_value("OSD_TEST_PROCESS_ROUND_TRIP", "one").unwrap();
    env.build();
    assert_eq!(std::env::var("OSD_TEST_PROCESS_ROUND_TRIP").unwrap(), "one");
    env.set_value("two").unwrap();
    env.build();
    assert_eq!(env.value().as_bytes(), b"two");
    env.remove();
    assert_eq!(std::env::var("OSD_TEST_PROCESS_ROUND_TRIP").unwrap(), "");
    assert!(
        Environment::named("OSD_TEST_PROCESS_NEVER_SET")
            .unwrap()
            .value()
            .is_empty()
    );
    assert!(Environment::new().name().is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn native_shared_library_opens_resolves_and_reports_errors() {
    let mut library = SharedLibrary::named(Some("libm.so.6"));
    assert_eq!(library.name(), Some("libm.so.6"));
    assert!(library.dl_open(LoadMode::RtldNow));
    let cosine = library.dl_symb("cos").expect("libm exports cos");
    // SAFETY: libm's cos has the C signature double(double).
    #[allow(unsafe_code)]
    let cos: extern "C" fn(f64) -> f64 = unsafe { std::mem::transmute(cosine.as_ptr()) };
    assert_eq!(cos(0.0), 1.0);
    assert!(library.dl_symb("no_such_symbol_osd").is_none());
    assert!(library.dl_error().unwrap().contains("no_such_symbol_osd"));
    assert!(library.dl_error().is_none(), "dlerror clears after a read");
    library.dl_close();
    assert!(library.dl_symb("cos").is_none());
    library.destroy();
    assert_eq!(library.name(), None);
}

#[cfg(target_os = "linux")]
#[test]
fn native_shared_library_failures_and_the_main_program() {
    let mut missing = SharedLibrary::new();
    missing.set_name(Some("libosd_missing_library.so"));
    missing.set_name(None);
    assert_eq!(missing.name(), Some("libosd_missing_library.so"));
    assert!(!missing.dl_open(LoadMode::RtldLazy));
    assert!(
        missing
            .dl_error()
            .unwrap()
            .contains("libosd_missing_library.so")
    );
    let mut main = SharedLibrary::default();
    assert!(main.dl_open(LoadMode::RtldLazy));
    assert!(main.dl_symb("malloc").is_some());
    main.dl_close();
}

/// Fixed memory figures standing for mallinfo2, procfs and task_info.
struct FakeMemInfo {
    status: Option<&'static str>,
}
impl MemInfoSource for FakeMemInfo {
    fn heap_usage(&self) -> usize {
        3 * 1024 * 1024 + 5
    }
    fn proc_status(&self) -> Option<String> {
        self.status.map(str::to_string)
    }
    fn task_counters(&self) -> Option<(usize, usize, usize)> {
        Some((64 << 20, 16 << 20, 3 << 20))
    }
}

const STATUS: &str = "Name:\tcad\nVmPeak:\t  999 kB\nVmSize:\t  204800 kB\n\nVmHWM:\t  \
                      40960 kB\nVmRSS:\t   20480 kB\nVmData:\t    8192 kB\nVmStk:\t     \
                      1024 kB\nVmExe:\t 12 kB\n";

#[test]
fn mem_info_starts_active_and_unknown_without_update() {
    let mut info = MemInfo::with_source(FakeMemInfo { status: None }, false);
    for counter in MemCounter::ALL {
        assert!(info.is_active(counter));
        assert_eq!(info.value(counter), usize::MAX);
        assert_eq!(info.value_mib(counter), usize::MAX);
        assert_eq!(info.value_precise_mib(counter), -1.0);
    }
    assert!(info.to_text().is_empty());
    info.set_active(false);
    assert!(!info.is_active(MemCounter::HeapUsage));
    info.set_counter_active(MemCounter::HeapUsage, true);
    assert!(info.is_active(MemCounter::HeapUsage));
    info.clear();
}

#[cfg(target_os = "linux")]
#[test]
fn mem_info_reads_linux_status_lines_in_native_order() {
    let mut info = MemInfo::with_source(
        FakeMemInfo {
            status: Some(STATUS),
        },
        true,
    );
    assert_eq!(info.value(MemCounter::Virtual), 204_800 * 1024);
    assert_eq!(info.value(MemCounter::WorkingSet), 20_480 * 1024);
    assert_eq!(info.value(MemCounter::WorkingSetPeak), 40_960 * 1024);
    assert_eq!(info.value(MemCounter::Private), (8192 + 1024) * 1024);
    assert_eq!(info.value(MemCounter::SwapUsage), usize::MAX);
    assert_eq!(info.value_mib(MemCounter::HeapUsage), 3);
    assert_eq!(
        info.value_precise_mib(MemCounter::HeapUsage),
        3.0 + 5.0 / 1_048_576.0
    );
    let expected = "  Private memory:     9 MiB\n  Working Set:        20 MiB (peak: 40 MiB)\n  \
                    Virtual memory:     200 MiB\n  Heap memory:     3 MiB\n";
    assert_eq!(info.to_text().as_bytes(), expected.as_bytes());
    info.set_counter_active(MemCounter::WorkingSetPeak, false);
    info.set_counter_active(MemCounter::Private, false);
    info.update();
    assert_eq!(info.value(MemCounter::Private), usize::MAX);
    assert!(
        info.to_text()
            .as_bytes()
            .starts_with(b"  Working Set:        20 MiB\n")
    );
}

#[cfg(target_os = "linux")]
#[test]
fn mem_info_skips_procfs_when_only_heap_and_swap_are_active() {
    let mut info = MemInfo::with_source(
        FakeMemInfo {
            status: Some(STATUS),
        },
        false,
    );
    info.set_active(false);
    info.set_counter_active(MemCounter::HeapUsage, true);
    info.set_counter_active(MemCounter::SwapUsage, true);
    info.update();
    assert_eq!(info.value(MemCounter::HeapUsage), 3 * 1024 * 1024 + 5);
    assert_eq!(info.to_text().as_bytes(), b"  Heap memory:     3 MiB\n");
}

#[cfg(target_os = "linux")]
#[test]
fn native_mem_info_fills_the_linux_counters() {
    let info = MemInfo::new(true);
    for counter in [
        MemCounter::Private,
        MemCounter::Virtual,
        MemCounter::WorkingSet,
        MemCounter::WorkingSetPeak,
    ] {
        assert_ne!(info.value(counter), usize::MAX, "{counter:?}");
    }
    assert_eq!(info.value(MemCounter::SwapUsage), usize::MAX);
    assert!(
        MemInfo::print_info()
            .as_bytes()
            .starts_with(b"  Private memory:")
    );
}
