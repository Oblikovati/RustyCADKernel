use super::*;

#[derive(Default)]
pub(super) struct InsertTrace {
    pub events: RefCell<String>,
    pub live: RefCell<Vec<bool>>,
    pub active: Cell<bool>,
    pub phase: Cell<char>,
    pub fail_shift: Cell<i32>,
    pub fail_insert: Cell<bool>,
    pub completed: Cell<usize>,
    pub count: Cell<usize>,
    pub insertion: Cell<usize>,
    pub destroyed_slot: Cell<Option<usize>>,
    pub buffer: Cell<*const InsertItem>,
}

pub(super) struct InsertItem {
    pub value: i32,
    pub resource: Option<Box<i32>>,
    pub trace: Rc<InsertTrace>,
}

impl InsertItem {
    pub fn new(value: i32, trace: &Rc<InsertTrace>) -> Self {
        Self {
            value,
            resource: Some(Box::new(value)),
            trace: Rc::clone(trace),
        }
    }

    pub fn copied(&self, phase: char) -> Self {
        self.trace.phase.set(phase);
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("{phase}{},", self.value));
        let resource = self.resource.as_ref().map(|value| Box::new(**value));
        assert!(
            !(phase == 'I' && self.trace.fail_insert.get()),
            "copy insertion: expected success"
        );
        Self {
            value: self.value,
            resource,
            trace: Rc::clone(&self.trace),
        }
    }

    pub fn moved(&mut self, phase: char) -> Self {
        self.trace.phase.set(phase);
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("{phase}{},", self.value));
        let value = std::mem::replace(&mut self.value, -1);
        let resource = self.resource.take();
        let failed = if phase == 'V' {
            self.trace.fail_insert.get()
        } else {
            value == self.trace.fail_shift.get()
        };
        assert!(!failed, "move construction value {value}: expected success");
        Self {
            value,
            resource,
            trace: Rc::clone(&self.trace),
        }
    }

    pub fn shift(source: &mut Self) -> Self {
        let slot = source.trace.count.get() - 1 - source.trace.completed.get();
        source
            .trace
            .buffer
            .set(std::ptr::from_ref(source).wrapping_sub(slot));
        let moved = source.moved('M');
        source.trace.live.borrow_mut()[slot + 1] = true;
        source.trace.destroyed_slot.set(Some(slot));
        source.trace.completed.set(source.trace.completed.get() + 1);
        moved
    }

    pub fn inserted(donor: &mut Self, moving: bool) -> Self {
        let inserted = if moving {
            donor.moved('V')
        } else {
            donor.copied('I')
        };
        donor.trace.live.borrow_mut()[donor.trace.insertion.get()] = true;
        inserted
    }

    pub fn state(&self) -> String {
        format!("{}/{}", self.value, u8::from(self.resource.is_some()))
    }
}

impl Drop for InsertItem {
    fn drop(&mut self) {
        if !self.trace.active.get() {
            return;
        }
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("D{},", self.value));
        if self.trace.phase.get() != 'M' {
            return;
        }
        let slot = self
            .trace
            .destroyed_slot
            .take()
            .expect("shift destructor: expected source slot");
        assert!(std::ptr::eq(
            self.trace.buffer.get().wrapping_add(slot),
            self
        ));
        self.trace.live.borrow_mut()[slot] = false;
    }
}
