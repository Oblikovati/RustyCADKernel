//! OSD_Timer, OSD_Chronometer and the OSD statics on named fake clocks, sleepers and
//! signal hosts, plus smoke checks of the native implementations.
// Most cases exercise the Linux native profile; other hosts compile the shared fakes only.
#![cfg_attr(not(target_os = "linux"), allow(unused_imports, dead_code))]
use rustycad_tkernel::osd::{
    Chronometer, CpuTimes, NativeCpuTimes, Osd, SignalHost, SignalMode, SignalState, Sleeper,
    Timer, WallClock, milli_sec_sleep_duration, sec_sleep_duration,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

/// A CPU clock the test advances by hand; process and thread times differ on purpose.
#[derive(Default)]
struct FakeCpuTimes {
    seconds: Mutex<f64>,
}
impl FakeCpuTimes {
    fn advance(&self, seconds: f64) {
        *self.seconds.lock().unwrap() += seconds;
    }
}
impl CpuTimes for FakeCpuTimes {
    fn process_cpu(&self) -> (f64, f64) {
        let now: f64 = *self.seconds.lock().unwrap();
        (now, now / 2.0)
    }
    fn thread_cpu(&self) -> (f64, f64) {
        (*self.seconds.lock().unwrap() / 4.0, 0.0)
    }
}

/// A wall clock that counts its reads and returns scripted times.
#[derive(Default)]
struct FakeWallClock {
    now: Mutex<f64>,
    reads: Mutex<u32>,
}
impl FakeWallClock {
    fn set(&self, seconds: f64) {
        *self.now.lock().unwrap() = seconds;
    }
}
impl WallClock for FakeWallClock {
    fn now(&self) -> f64 {
        *self.reads.lock().unwrap() += 1;
        *self.now.lock().unwrap()
    }
}

fn fake_timer() -> (Timer, Arc<FakeWallClock>, Arc<FakeCpuTimes>) {
    let wall: Arc<FakeWallClock> = Arc::default();
    let cpu: Arc<FakeCpuTimes> = Arc::default();
    let timer: Timer = Timer::with_cpu(false, cpu.clone(), wall.clone());
    (timer, wall, cpu)
}

#[test]
fn timer_accumulates_wall_laps_and_ignores_repeated_start_stop() {
    let (mut timer, wall, cpu) = fake_timer();
    wall.set(10.0);
    timer.start();
    wall.set(12.5);
    assert_eq!(timer.elapsed_time(), 2.5);
    cpu.advance(1.0);
    timer.stop();
    timer.stop();
    wall.set(100.0);
    assert_eq!(timer.elapsed_time(), 2.5);
    timer.start();
    timer.start();
    wall.set(101.0);
    assert_eq!(timer.elapsed_time(), 3.5);
    assert!(timer.is_started());
    assert_eq!(timer.user_time_cpu(), 1.0);
    assert_eq!(timer.system_time_cpu(), 0.5);
}

#[test]
fn timer_restart_reads_the_wall_clock_twice_like_the_virtual_chain() {
    let (mut timer, wall, _cpu) = fake_timer();
    wall.set(7.0);
    timer.restart();
    assert_eq!(*wall.reads.lock().unwrap(), 2);
    assert!(timer.is_started());
    wall.set(9.0);
    assert_eq!(timer.elapsed_time(), 2.0);
}

#[test]
fn timer_reset_to_splits_hours_minutes_and_prints_at_precision_12() {
    let (mut timer, _wall, _cpu) = fake_timer();
    timer.reset_to(3725.5);
    assert!(!timer.is_started());
    assert_eq!(timer.show_values(), (5.5, 2, 1, 0.0));
    assert_eq!(
        timer.show_to(),
        "Elapsed time: 1 Hours 2 Minutes 5.5 Seconds\nCPU user time: 0 seconds\nCPU system time: 0 seconds\n"
    );
    timer.reset_to(-3725.5);
    assert_eq!(timer.show_values(), (-5.5, -2, -1, 0.0));
    timer.reset();
    assert_eq!(timer.elapsed_time(), 0.0);
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[test]
fn timer_keeps_the_x86_int_min_truncation_of_out_of_range_times() {
    let (mut timer, _wall, _cpu) = fake_timer();
    timer.reset_to(f64::NAN);
    let (seconds, minutes, hours, _) = timer.show_values();
    assert_eq!((minutes, hours), (-14, -596523));
    assert!(seconds.is_nan());
}

#[test]
fn chronometer_reads_the_selected_scope_and_refuses_switching_while_running() {
    let cpu: Arc<FakeCpuTimes> = Arc::default();
    let mut chrono: Chronometer = Chronometer::with_clock(true, cpu.clone());
    assert!(chrono.is_this_thread_only());
    chrono.start();
    cpu.advance(8.0);
    assert_eq!(chrono.show_cpu(), (2.0, 0.0));
    let failure = chrono.set_this_thread_only(false).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    chrono.stop();
    assert_eq!(chrono.show_user(), 2.0);
    chrono.set_this_thread_only(false).unwrap();
    chrono.restart();
    cpu.advance(2.0);
    assert_eq!(chrono.show_cpu(), (2.0, 1.0));
    chrono.reset();
    assert_eq!(
        chrono.show_to(),
        "CPU user time: 0 seconds\nCPU system time: 0 seconds\n"
    );
}

#[test]
fn native_clocks_report_nonnegative_times() {
    let (user, sys) = Chronometer::get_process_cpu();
    assert!(user >= 0.0 && sys >= 0.0);
    let (user, sys) = Chronometer::get_thread_cpu();
    assert!(user >= 0.0 && sys >= 0.0);
    assert!(NativeCpuTimes.process_cpu().0 >= 0.0);
    let first: f64 = Timer::get_wall_clock_time();
    assert!(Timer::get_wall_clock_time() >= first);
    let mut timer: Timer = Timer::default();
    timer.start();
    timer.stop();
    assert!(timer.elapsed_time() >= 0.0);
    assert!(!Chronometer::default().is_started());
}

#[derive(Default)]
struct RecordingSleeper {
    requests: Mutex<Vec<Duration>>,
}
impl Sleeper for RecordingSleeper {
    fn sleep(&self, duration: Duration) {
        self.requests.lock().unwrap().push(duration);
    }
}

#[test]
fn sleeps_request_the_native_integer_durations() {
    let sleeper = RecordingSleeper::default();
    Osd::sec_sleep_with(&sleeper, 2);
    Osd::milli_sec_sleep_with(&sleeper, 5);
    assert_eq!(
        *sleeper.requests.lock().unwrap(),
        vec![Duration::from_secs(2), Duration::from_millis(5)]
    );
    Osd::milli_sec_sleep(0);
    Osd::sec_sleep(0);
}

#[cfg(unix)]
#[test]
fn unix_sleep_durations_wrap_like_the_int_microsecond_product() {
    assert_eq!(
        sec_sleep_duration(2148),
        Duration::from_micros(2_148_000_000)
    );
    assert_eq!(sec_sleep_duration(4295), Duration::from_micros(32_704));
    assert_eq!(
        milli_sec_sleep_duration(-1),
        Duration::from_micros(4_294_966_296)
    );
}

#[derive(Default)]
struct RecordingSignalHost {
    calls: Mutex<Vec<String>>,
    floating: Mutex<bool>,
}
impl SignalHost for RecordingSignalHost {
    fn set_floating_signal(&self, enable: bool) {
        *self.floating.lock().unwrap() = enable;
        self.calls
            .lock()
            .unwrap()
            .push(format!("floating {enable}"));
    }
    fn to_catch_floating_signals(&self) -> bool {
        *self.floating.lock().unwrap()
    }
    fn set_signal_handlers(&self, mode: SignalMode) {
        self.calls
            .lock()
            .unwrap()
            .push(format!("handlers {mode:?}"));
    }
    fn debug_mode_requested(&self) -> bool {
        true
    }
}

#[cfg(unix)]
#[test]
fn unix_set_signal_orders_floating_mode_then_handlers() {
    let state: SignalState = SignalState::new();
    let host = RecordingSignalHost::default();
    assert_eq!(state.signal_mode(), SignalMode::AsIs);
    state.set_signal(&host, SignalMode::AsIs, true);
    state.set_signal(&host, SignalMode::SetUnhandled, false);
    state.set_thread_local_signal(&host, SignalMode::Unset, true);
    assert_eq!(state.signal_mode(), SignalMode::SetUnhandled);
    assert_eq!(
        *host.calls.lock().unwrap(),
        vec![
            "floating true",
            "floating false",
            "handlers SetUnhandled",
            "floating true"
        ]
    );
    assert!(host.to_catch_floating_signals());
    assert_eq!(
        state.signal_stack_trace_length(),
        0,
        "CSF_DEBUG_MODE is Windows only"
    );
}

#[test]
fn control_break_raises_once_per_armed_interrupt() {
    let state: SignalState = SignalState::default();
    assert!(state.control_break().is_ok());
    state.arm_control_break();
    let failure = state.control_break().unwrap_err();
    assert_eq!(failure.exception_type(), "OSD_Exception_CTRL_BREAK");
    assert_eq!(failure.message(), "*** INTERRUPT ***");
    assert!(state.control_break().is_ok());
    state.set_signal_stack_trace_length(-3);
    assert_eq!(state.signal_stack_trace_length(), -3);
}

#[test]
fn global_statics_round_trip_on_the_native_host() {
    Osd::set_signal_stack_trace_length(4);
    assert_eq!(Osd::signal_stack_trace_length(), 4);
    Osd::set_signal_stack_trace_length(0);
    Osd::set_signal(SignalMode::AsIs, false);
    Osd::set_thread_local_signal(SignalMode::AsIs, false);
    Osd::set_floating_signal(false);
    assert!(!Osd::to_catch_floating_signals());
    assert_eq!(Osd::signal_mode(), SignalMode::AsIs);
    assert!(Osd::control_break().is_ok());
}

#[cfg(all(target_os = "linux", target_env = "gnu", target_arch = "x86_64"))]
#[test]
fn linux_floating_traps_toggle_for_the_calling_thread() {
    std::thread::spawn(|| {
        Osd::set_floating_signal(true);
        let catching: bool = Osd::to_catch_floating_signals();
        Osd::set_floating_signal(false);
        assert!(catching);
        assert!(!Osd::to_catch_floating_signals());
    })
    .join()
    .unwrap();
}

#[test]
fn real_text_round_trips_through_the_native_spellings() {
    assert_eq!(Osd::real_to_c_string(1e5), "1e+05");
    assert_eq!(Osd::real_to_c_string(-0.0), "-0");
    assert_eq!(Osd::real_to_c_string(2.5e-320), "2.49997216795670751e-320");
    assert_eq!(Osd::c_string_to_real("1e5"), (true, 1e5));
    assert_eq!(Osd::c_string_to_real(" 1.5"), (true, 1.5));
    assert!(!Osd::c_string_to_real("1.5 ").0);
    let text: String = Osd::real_to_c_string(0.1);
    assert_eq!(Osd::c_string_to_real(&text), (true, 0.1));
}
