use std::cell::RefCell;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
type SharedTrace = Arc<Mutex<FillTrace>>;

// Default has no dependency parameter; a scoped per-thread fixture observes its required order.
thread_local! {
    static ACTIVE_FILL_TRACE: RefCell<Option<SharedTrace>> = const { RefCell::new(None) };
}
#[derive(Default)]
struct FillTrace {
    events: String,
    defaults: usize,
    fail: usize,
}
pub struct FillScope {
    trace: SharedTrace,
    previous: Option<SharedTrace>,
}
impl FillScope {
    /// Installs an isolated constructor journal; `FillScope::new(0)`.
    pub fn new(fail: usize) -> Self {
        let trace = Arc::new(Mutex::new(FillTrace {
            fail,
            ..FillTrace::default()
        }));
        let previous = ACTIVE_FILL_TRACE.with(|slot| slot.replace(Some(Arc::clone(&trace))));
        Self { trace, previous }
    }
    /// Retains the caller's donor outside constructor cleanup; `scope.donor(9)`.
    pub fn donor(&self, number: i32) -> FillProbe {
        FillProbe {
            id: 0,
            number,
            reads: AtomicUsize::new(0),
            trace: Arc::clone(&self.trace),
        }
    }
    /// Takes default/assignment/destructor events; `scope.take_events()`.
    pub fn take_events(&self) -> String {
        let events = std::mem::take(&mut self.trace.lock().unwrap().events);
        if events.is_empty() {
            "-".to_owned()
        } else {
            events
        }
    }
}
impl Drop for FillScope {
    fn drop(&mut self) {
        ACTIVE_FILL_TRACE.with(|slot| {
            slot.replace(self.previous.take());
        });
    }
}
pub struct FillProbe {
    id: usize,
    pub number: i32,
    reads: AtomicUsize,
    trace: SharedTrace,
}
impl Default for FillProbe {
    fn default() -> Self {
        let trace = ACTIVE_FILL_TRACE.with(|slot| Arc::clone(slot.borrow().as_ref().unwrap()));
        let id = {
            let mut log = trace.lock().unwrap();
            log.defaults += 1;
            let id = log.defaults;
            log.events += &format!("d{id};");
            id
        };
        Self {
            id,
            number: -(id as i32),
            reads: AtomicUsize::new(0),
            trace,
        }
    }
}
impl FillProbe {
    /// Preserves native borrowed assignment and failing-slot mutation; `FillProbe::assign(slot, donor)`.
    pub fn assign(&mut self, donor: &Self) {
        donor.reads.fetch_add(1, Ordering::Relaxed);
        self.number = donor.number;
        let fail = {
            let mut log = self.trace.lock().unwrap();
            log.events += &format!("a{}={};", self.id, self.number);
            log.fail
        };
        assert_ne!(
            self.id, fail,
            "assignment slot={}; expected a non-failing slot",
            self.id
        );
    }
    /// Reads the source-side native mutable observation; `donor.reads()`.
    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::Relaxed)
    }
}
impl Drop for FillProbe {
    fn drop(&mut self) {
        self.trace.lock().unwrap().events += &format!("x{}={};", self.id, self.number);
    }
}
#[derive(Default)]
pub struct ClonePolicy(pub FillProbe);
impl Clone for ClonePolicy {
    fn clone(&self) -> Self {
        panic!("copy construction requested; expected assignment-only constructor path")
    }
    fn clone_from(&mut self, source: &Self) {
        self.0.assign(&source.0);
    }
}
