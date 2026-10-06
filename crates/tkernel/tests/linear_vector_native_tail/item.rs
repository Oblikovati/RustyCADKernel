use super::*;

#[derive(Default)]
pub struct TailTrace {
    pub serial: Cell<usize>,
    pub failing: Cell<usize>,
    pub active: Cell<bool>,
    pub events: RefCell<String>,
}

pub struct TailItem<'a> {
    pub value: i32,
    pub trace: &'a TailTrace,
}

impl<'a> TailItem<'a> {
    pub fn constructed(trace: &'a TailTrace, fill: bool) -> Self {
        let serial = trace.serial.get() + 1;
        trace.serial.set(serial);
        let value = if fill {
            99
        } else {
            100 + i32::try_from(serial).unwrap()
        };
        let phase = if fill { 'C' } else { 'A' };
        trace
            .events
            .borrow_mut()
            .push_str(&format!("{phase}{value},"));
        assert_ne!(
            serial,
            trace.failing.get(),
            "injected tail constructor failure {serial}"
        );
        Self { value, trace }
    }

    pub fn moved(&mut self) -> Self {
        let value = std::mem::replace(&mut self.value, -1);
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("M{value},"));
        Self {
            value,
            trace: self.trace,
        }
    }
}

impl Drop for TailItem<'_> {
    fn drop(&mut self) {
        if self.trace.active.get() {
            self.trace
                .events
                .borrow_mut()
                .push_str(&format!("D{},", self.value));
        }
    }
}
