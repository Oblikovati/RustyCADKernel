use super::*;

#[derive(Default)]
pub struct ResizeTrace {
    pub events: RefCell<String>,
    pub original_slots: RefCell<Vec<*const ResizeItem>>,
    pub active: Cell<bool>,
    pub resources: Cell<usize>,
    pub tail_attempts: Cell<usize>,
    pub completed_tail: Cell<usize>,
    pub failing_attempt: Cell<usize>,
    pub failing_drop: Cell<Option<i32>>,
}

pub struct ResizeItem {
    pub value: i32,
    payload: Option<Box<ResizePayload>>,
    trace: Rc<ResizeTrace>,
}

struct ResizePayload {
    value: i32,
    trace: Rc<ResizeTrace>,
}

impl ResizeTrace {
    fn constructed(&self, phase: char, value: i32) {
        if !self.active.get() {
            return;
        }
        self.events
            .borrow_mut()
            .push_str(&format!("{phase}{value},"));
        if phase != 'A' && phase != 'C' {
            return;
        }
        self.tail_attempts.set(self.tail_attempts.get() + 1);
        assert_ne!(
            self.tail_attempts.get(),
            self.failing_attempt.get(),
            "tail attempt {}: expected successful construction",
            self.tail_attempts.get()
        );
        self.completed_tail.set(self.completed_tail.get() + 1);
    }

    fn original_slot(&self, item: &ResizeItem) -> String {
        self.original_slots
            .borrow()
            .iter()
            .position(|&slot| std::ptr::eq(slot, item))
            .map_or_else(|| String::from("X"), |index| index.to_string())
    }
}

impl ResizePayload {
    fn new(value: i32, trace: &Rc<ResizeTrace>) -> Box<Self> {
        trace.resources.set(trace.resources.get() + 1);
        Box::new(Self {
            value,
            trace: Rc::clone(trace),
        })
    }
}

impl ResizeItem {
    pub fn new(value: i32, trace: &Rc<ResizeTrace>) -> Self {
        Self::with_payload(value, Some(ResizePayload::new(value, trace)), trace)
    }

    fn with_payload(
        value: i32,
        payload: Option<Box<ResizePayload>>,
        trace: &Rc<ResizeTrace>,
    ) -> Self {
        Self {
            value,
            payload,
            trace: Rc::clone(trace),
        }
    }

    pub fn copied(&self) -> Self {
        let payload = self
            .payload
            .as_ref()
            .map(|value| ResizePayload::new(value.value, &self.trace));
        self.trace
            .constructed(if self.value == 99 { 'C' } else { 'G' }, self.value);
        Self::with_payload(self.value, payload, &self.trace)
    }

    pub fn moved(&mut self) -> Self {
        let value = std::mem::replace(&mut self.value, -1);
        let payload = self.payload.take();
        self.trace.constructed('M', value);
        Self::with_payload(value, payload, &self.trace)
    }

    pub fn default_constructed(trace: &Rc<ResizeTrace>) -> Self {
        let payload = Some(ResizePayload::new(0, trace));
        trace.constructed('A', 0);
        Self::with_payload(0, payload, trace)
    }

    pub fn state(&self) -> String {
        format!("{}/{}", self.value, u8::from(self.payload.is_some()))
    }
}

impl Drop for ResizeItem {
    fn drop(&mut self) {
        if !self.trace.active.get() {
            return;
        }
        let slot = self.trace.original_slot(self);
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("D{}@{slot},", self.value));
        assert_ne!(
            Some(self.value),
            self.trace.failing_drop.get(),
            "destructor value {}: expected a non-failing value",
            self.value
        );
    }
}

impl Drop for ResizePayload {
    fn drop(&mut self) {
        self.trace.resources.set(self.trace.resources.get() - 1);
    }
}
