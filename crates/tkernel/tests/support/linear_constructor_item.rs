//! Shared owning payloads for native copy/fill construction phase observations.
use std::cell::{Cell, RefCell};

#[derive(Default)]
pub struct CopyTrace {
    pub attempts: Cell<usize>,
    pub completed: Cell<usize>,
    pub failing: Cell<usize>,
    pub resources: Cell<usize>,
    pub active: Cell<bool>,
    pub events: RefCell<String>,
}

struct CopyPayload<'a> {
    value: i32,
    trace: &'a CopyTrace,
}
impl<'a> CopyPayload<'a> {
    fn new(value: i32, trace: &'a CopyTrace) -> Box<Self> {
        trace.resources.set(trace.resources.get() + 1);
        Box::new(Self { value, trace })
    }
}
impl Drop for CopyPayload<'_> {
    fn drop(&mut self) {
        self.trace.resources.set(self.trace.resources.get() - 1);
    }
}

pub struct CopyItem<'a> {
    pub value: i32,
    payload: Box<CopyPayload<'a>>,
}
impl<'a> CopyItem<'a> {
    pub fn new(value: i32, trace: &'a CopyTrace) -> Self {
        Self {
            value,
            payload: CopyPayload::new(value, trace),
        }
    }
}
impl Clone for CopyItem<'_> {
    fn clone(&self) -> Self {
        let trace = self.payload.trace;
        let payload = CopyPayload::new(self.payload.value, trace);
        trace
            .events
            .borrow_mut()
            .push_str(&format!("C{},", self.value));
        let attempt = trace.attempts.get() + 1;
        trace.attempts.set(attempt);
        if attempt == trace.failing.get() {
            std::panic::panic_any(attempt);
        }
        trace.completed.set(trace.completed.get() + 1);
        Self {
            value: self.value,
            payload,
        }
    }
}
impl Drop for CopyItem<'_> {
    fn drop(&mut self) {
        if self.payload.trace.active.get() {
            self.payload
                .trace
                .events
                .borrow_mut()
                .push_str(&format!("D{},", self.value));
        }
    }
}
