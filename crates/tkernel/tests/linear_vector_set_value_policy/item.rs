use super::*;

#[derive(Default)]
pub struct SetTrace {
    pub events: RefCell<String>,
    pub original_slots: RefCell<Vec<*const SetItem>>,
    pub active: Cell<bool>,
    pub resources: Cell<usize>,
    pub tail_attempts: Cell<usize>,
    pub completed_tail: Cell<usize>,
    pub failing_attempt: Cell<usize>,
    pub failing_assignment: Cell<bool>,
}

pub struct SetItem {
    pub value: i32,
    payload: Option<Box<SetPayload>>,
    trace: Rc<SetTrace>,
}

struct SetPayload {
    value: i32,
    trace: Rc<SetTrace>,
}

impl SetTrace {
    fn constructed(&self, phase: char, value: i32) {
        if !self.active.get() {
            return;
        }
        self.events
            .borrow_mut()
            .push_str(&format!("{phase}{value},"));
        if phase != 'A' {
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

    fn assigned(&self, phase: char, before: i32, after: i32) {
        self.events
            .borrow_mut()
            .push_str(&format!("{phase}{before}>{after},"));
        assert!(
            !self.failing_assignment.get(),
            "assignment value {after}: expected success"
        );
    }

    fn original_slot(&self, item: &SetItem) -> String {
        self.original_slots
            .borrow()
            .iter()
            .position(|&slot| std::ptr::eq(slot, item))
            .map_or_else(|| String::from("X"), |index| index.to_string())
    }
}

impl SetPayload {
    fn new(value: i32, trace: &Rc<SetTrace>) -> Box<Self> {
        trace.resources.set(trace.resources.get() + 1);
        Box::new(Self {
            value,
            trace: Rc::clone(trace),
        })
    }
}

impl SetItem {
    pub fn new(value: i32, trace: &Rc<SetTrace>) -> Self {
        Self::with_payload(value, Some(SetPayload::new(value, trace)), trace)
    }

    fn with_payload(value: i32, payload: Option<Box<SetPayload>>, trace: &Rc<SetTrace>) -> Self {
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
            .map(|value| SetPayload::new(value.value, &self.trace));
        self.trace.constructed('G', self.value);
        Self::with_payload(self.value, payload, &self.trace)
    }

    pub fn moved(&mut self) -> Self {
        let value = std::mem::replace(&mut self.value, -1);
        let payload = self.payload.take();
        self.trace.constructed('M', value);
        Self::with_payload(value, payload, &self.trace)
    }

    pub fn default_constructed(trace: &Rc<SetTrace>) -> Self {
        let payload = Some(SetPayload::new(0, trace));
        trace.constructed('A', 0);
        Self::with_payload(0, payload, trace)
    }

    pub fn assign_copy(&mut self, source: &Self) {
        let before = self.value;
        self.value = source.value;
        self.payload = source
            .payload
            .as_ref()
            .map(|value| SetPayload::new(value.value, &self.trace));
        self.trace.assigned('C', before, self.value);
    }

    pub fn assign_move(&mut self, source: &mut Self) {
        let before = self.value;
        self.value = std::mem::replace(&mut source.value, -1);
        self.payload = source.payload.take();
        self.trace.assigned('M', before, self.value);
    }

    pub fn payload_pointer(&self) -> *const i32 {
        self.payload.as_deref().map_or(std::ptr::null(), |payload| {
            std::ptr::from_ref(&payload.value)
        })
    }

    pub fn state(&self) -> String {
        format!("{}/{}", self.value, u8::from(self.payload.is_some()))
    }
}

impl Drop for SetItem {
    fn drop(&mut self) {
        if !self.trace.active.get() {
            return;
        }
        let slot = self.trace.original_slot(self);
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("D{}@{slot},", self.value));
    }
}

impl Drop for SetPayload {
    fn drop(&mut self) {
        self.trace.resources.set(self.trace.resources.get() - 1);
    }
}
