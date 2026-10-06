use super::*;

#[derive(Default)]
pub struct LastTrace {
    pub slots: RefCell<Vec<*const LastItem>>,
    pub events: RefCell<String>,
    pub resources: Cell<usize>,
    pub active: Cell<bool>,
    pub failing_value: Cell<i32>,
}

pub struct LastItem {
    pub value: i32,
    payload: LastPayload,
}

struct LastPayload {
    value: i32,
    trace: Rc<LastTrace>,
}

impl LastItem {
    pub fn new(value: i32, trace: &Rc<LastTrace>) -> Self {
        trace.resources.set(trace.resources.get() + 1);
        Self {
            value,
            payload: LastPayload {
                value,
                trace: Rc::clone(trace),
            },
        }
    }
}

impl LastTrace {
    fn original_slot(&self, item: &LastItem) -> String {
        self.slots
            .borrow()
            .iter()
            .position(|&p| std::ptr::eq(p, item))
            .map_or_else(|| String::from("X"), |index| index.to_string())
    }
}

impl Drop for LastItem {
    fn drop(&mut self) {
        let trace = &self.payload.trace;
        if !trace.active.get() {
            return;
        }
        let location = trace.original_slot(self);
        trace
            .events
            .borrow_mut()
            .push_str(&format!("D{}@{location},", self.value));
        assert_ne!(
            self.value,
            trace.failing_value.get(),
            "destructor value {}: expected a non-failing value",
            self.value
        );
    }
}

impl Drop for LastPayload {
    fn drop(&mut self) {
        if self.trace.active.get() {
            self.trace
                .events
                .borrow_mut()
                .push_str(&format!("R{},", self.value));
        }
        self.trace.resources.set(self.trace.resources.get() - 1);
    }
}
