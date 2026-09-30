use super::{PerfMeter, PerfRegistry, Stopwatch};
use crate::tcollection::AsciiString;
impl PerfMeter {
    /// Create an unnamed handle without creating a watch; `PerfMeter::empty(registry)`.
    pub fn empty(registry: PerfRegistry) -> Self {
        Self {
            registry,
            name: AsciiString::default(),
        }
    }
    /// Attach to a name and optionally restart it; `PerfMeter::new(registry, "Work", true)`.
    pub fn new(registry: PerfRegistry, name: &str, auto_start: bool) -> Self {
        let mut meter: Self = Self::empty(registry);
        meter.init(name);
        if auto_start {
            meter.start();
        }
        meter
    }
    /// Reuse an existing watch or create an inactive one; `meter.init("Work")`.
    pub fn init(&mut self, name: &str) {
        self.name = AsciiString::new(name);
        let mut state = self.registry.state.borrow_mut();
        if !state.watches.is_bound(&self.name) {
            let now: f64 = state.clock.now();
            state.watches.bind(self.name.clone(), Stopwatch::new(now));
        }
    }
    /// Restart a registered watch; removed or uninitialized names are ignored, `meter.start()`.
    pub fn start(&self) {
        let mut state = self.registry.state.borrow_mut();
        if !state.watches.is_bound(&self.name) {
            return;
        }
        let now: f64 = state.clock.now();
        let watch: &mut Stopwatch = state.watches.change_find(&self.name).unwrap();
        watch.start = now;
        watch.running = true;
    }
    /// Freeze a running watch; repeated calls do nothing, `meter.stop()`.
    pub fn stop(&self) {
        let mut state = self.registry.state.borrow_mut();
        if !state
            .watches
            .find(&self.name)
            .is_ok_and(|watch| watch.running)
        {
            return;
        }
        let now: f64 = state.clock.now();
        let watch: &mut Stopwatch = state.watches.change_find(&self.name).unwrap();
        watch.end = now;
        watch.running = false;
    }
    /// Read CPU seconds since the most recent start, or zero if absent; `meter.elapsed()`.
    pub fn elapsed(&self) -> f64 {
        let state = self.registry.state.borrow();
        state
            .watches
            .find(&self.name)
            .map_or(0.0, |watch| watch.elapsed(state.clock.as_ref()))
    }
    /// Remove the watch shared by every handle of this name; `meter.kill()`.
    pub fn kill(&self) {
        self.registry.state.borrow_mut().watches.unbind(&self.name);
    }
    /// Render the native state message or empty text for an absent name; `meter.print()`.
    pub fn print(&self) -> String {
        self.registry
            .state
            .borrow()
            .watches
            .find(&self.name)
            .map_or_else(|_| String::new(), |watch| watch.print(&self.name))
    }
}
