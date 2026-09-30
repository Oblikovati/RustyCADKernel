//! Named stopwatches with injected clock and explicitly shared registry lifetime.
use crate::{ncollection::DataMap, tcollection::AsciiString};
use std::{cell::RefCell, rc::Rc};
mod clock;
mod meter;
pub use clock::NativeCpuClock;
/// Clock dependency matching the native GetThreadCPU user-seconds output.
pub trait CpuClock {
    /// Read absolute CPU seconds; `clock.now()`; native failures return zero.
    fn now(&self) -> f64;
}
#[derive(Clone)]
struct Stopwatch {
    start: f64,
    end: f64,
    running: bool,
}
impl Stopwatch {
    fn new(now: f64) -> Self {
        Self {
            start: now,
            end: now,
            running: false,
        }
    }
    fn elapsed(&self, clock: &dyn CpuClock) -> f64 {
        (if self.running { clock.now() } else { self.end }) - self.start
    }
    fn print(&self, name: &AsciiString) -> String {
        let name: String = String::from_utf8_lossy(name.as_bytes()).into_owned();
        if self.running {
            return format!("Warning: Stopwatch {name} is still running.\n");
        }
        if self.end - self.start <= 0.0 {
            return format!("Stopwatch {name} have never been started.\n");
        }
        let elapsed: AsciiString = AsciiString::from_real(self.end - self.start);
        format!(
            "Stopwatch {name}: {} sec\n",
            String::from_utf8_lossy(elapsed.as_bytes())
        )
    }
}
struct RegistryState {
    watches: DataMap<AsciiString, Stopwatch>,
    clock: Rc<dyn CpuClock>,
}
/// Shared name registry; clones refer to the same watches, `PerfRegistry::new(clock)`.
#[derive(Clone)]
pub struct PerfRegistry {
    state: Rc<RefCell<RegistryState>>,
}
impl PerfRegistry {
    /// Inject CPU time and isolate a registry; `PerfRegistry::new(Rc::new(NativeCpuClock))`.
    pub fn new(clock: Rc<dyn CpuClock>) -> Self {
        Self {
            state: Rc::new(RefCell::new(RegistryState {
                watches: DataMap::default(),
                clock,
            })),
        }
    }
    /// Remove all names, invalidating live meter lookups until reinitialized; `registry.reset_all()`.
    pub fn reset_all(&self) {
        self.state.borrow_mut().watches.clear(false);
    }
    /// Render all watches in native hash order, including inactive watches; `registry.print_all()`.
    pub fn print_all(&self) -> String {
        self.state
            .borrow()
            .watches
            .items()
            .map(|(name, watch)| watch.print(name))
            .collect()
    }
}
/// Name-based meter handle; destroying a handle retains its watch in the registry.
pub struct PerfMeter {
    registry: PerfRegistry,
    name: AsciiString,
}
