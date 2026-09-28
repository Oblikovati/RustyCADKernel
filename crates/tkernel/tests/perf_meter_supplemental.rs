//! Deterministic clock fixtures isolate registry state from scheduling.
use rustycad_tkernel::osd::{CpuClock, PerfMeter, PerfRegistry};
use std::{cell::Cell, rc::Rc};
#[derive(Default)]
struct FakeCpuClock {
    seconds: Cell<f64>,
}
impl CpuClock for FakeCpuClock {
    fn now(&self) -> f64 {
        self.seconds.get()
    }
}
#[test]
fn shared_names_restart_instead_of_accumulating() {
    let clock: Rc<FakeCpuClock> = Rc::default();
    let registry = PerfRegistry::new(clock.clone());
    let meter = PerfMeter::new(registry.clone(), "Shared", false);
    assert_eq!(meter.print(), "Stopwatch Shared have never been started.\n");
    clock.seconds.set(10.0);
    meter.start();
    assert_eq!(
        meter.print(),
        "Warning: Stopwatch Shared is still running.\n"
    );
    let other = PerfMeter::new(registry, "Shared", false);
    clock.seconds.set(13.0);
    assert_eq!(other.elapsed(), 3.0);
    other.stop();
    clock.seconds.set(20.0);
    meter.stop();
    assert_eq!(meter.elapsed(), 3.0);
    meter.start();
    clock.seconds.set(22.0);
    meter.stop();
    assert_eq!(other.elapsed(), 2.0);
    assert_eq!(other.print(), "Stopwatch Shared: 2 sec\n");
}
#[test]
fn kill_reset_and_uninitialized_handles_follow_registry_lookup() {
    let clock: Rc<FakeCpuClock> = Rc::default();
    let registry = PerfRegistry::new(clock.clone());
    let mut meter = PerfMeter::empty(registry.clone());
    meter.start();
    meter.stop();
    meter.kill();
    assert_eq!(meter.elapsed(), 0.0);
    assert!(meter.print().is_empty());
    meter.init("Meter");
    meter.start();
    clock.seconds.set(5.0);
    meter.stop();
    assert_eq!(registry.print_all(), "Stopwatch Meter: 5 sec\n");
    registry.reset_all();
    meter.start();
    assert_eq!(meter.elapsed(), 0.0);
    meter.init("Meter");
    assert_eq!(meter.elapsed(), 0.0);
    meter.kill();
    assert!(registry.print_all().is_empty());
}
#[test]
fn auto_start_resets_existing_watch_and_drop_retains_it() {
    let clock: Rc<FakeCpuClock> = Rc::default();
    let registry = PerfRegistry::new(clock.clone());
    let meter = PerfMeter::new(registry.clone(), "Reset", true);
    clock.seconds.set(8.0);
    let restarted = PerfMeter::new(registry.clone(), "Reset", true);
    clock.seconds.set(10.0);
    restarted.stop();
    assert_eq!(meter.elapsed(), 2.0);
    drop(restarted);
    drop(meter);
    assert_eq!(registry.print_all(), "Stopwatch Reset: 2 sec\n");
}
