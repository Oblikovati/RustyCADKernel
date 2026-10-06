use super::*;

#[derive(Default)]
pub struct AppendTrace {
    pub events: RefCell<String>,
    pub active: Cell<bool>,
    pub failing: Cell<bool>,
    pub construction: Cell<char>,
}

pub struct AppendItem {
    pub value: i32,
    pub payload: Option<Box<i32>>,
    pub trace: Rc<AppendTrace>,
}

impl AppendTrace {
    fn constructed(&self, phase: char, value: i32) {
        if !self.active.get() {
            return;
        }
        self.events
            .borrow_mut()
            .push_str(&format!("{phase}{value},"));
        let failed =
            self.failing.get() && phase == self.construction.get() && matches!(value, 99 | 0 | 42);
        assert!(
            !failed,
            "append constructor value {value}: expected success"
        );
    }
}

impl AppendItem {
    pub fn new(value: i32, trace: &Rc<AppendTrace>) -> Self {
        Self {
            value,
            payload: Some(Box::new(value)),
            trace: Rc::clone(trace),
        }
    }

    pub fn copied(&self) -> Self {
        let payload = self.payload.as_ref().map(|value| Box::new(**value));
        self.trace.constructed('C', self.value);
        Self {
            value: self.value,
            payload,
            trace: Rc::clone(&self.trace),
        }
    }

    pub fn moved(&mut self) -> Self {
        let value = std::mem::replace(&mut self.value, -1);
        let payload = self.payload.take();
        self.trace.constructed('M', value);
        Self {
            value,
            payload,
            trace: Rc::clone(&self.trace),
        }
    }

    pub fn constructed(value: i32, phase: char, trace: &Rc<AppendTrace>) -> Self {
        let payload = Some(Box::new(value));
        trace.constructed(phase, value);
        Self {
            value,
            payload,
            trace: Rc::clone(trace),
        }
    }

    pub fn state(&self) -> String {
        format!("{}/{}", self.value, u8::from(self.payload.is_some()))
    }
}

impl Drop for AppendItem {
    fn drop(&mut self) {
        if self.trace.active.get() {
            self.trace
                .events
                .borrow_mut()
                .push_str(&format!("D{},", self.value));
        }
    }
}
