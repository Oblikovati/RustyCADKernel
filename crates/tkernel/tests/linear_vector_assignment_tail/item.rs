use super::*;

#[derive(Default)]
pub struct AssignmentTrace {
    pub copies: Cell<usize>,
    pub assignments: Cell<usize>,
    pub fail_copy: Cell<usize>,
    pub fail_assignment: Cell<usize>,
    pub active: Cell<bool>,
    pub events: RefCell<String>,
    pub slots: RefCell<Vec<*const ()>>,
}

pub struct AssignmentItem<'a> {
    pub value: i32,
    pub trace: &'a AssignmentTrace,
}

impl Clone for AssignmentItem<'_> {
    fn clone(&self) -> Self {
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("C{},", self.value));
        let count = self.trace.copies.get() + 1;
        self.trace.copies.set(count);
        assert_ne!(
            count,
            self.trace.fail_copy.get(),
            "copy {count}: expected successful tail construction"
        );
        Self {
            value: self.value,
            trace: self.trace,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        let previous = std::mem::replace(&mut self.value, source.value);
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("A{previous}>{},", self.value));
        let count = self.trace.assignments.get() + 1;
        self.trace.assignments.set(count);
        assert_ne!(
            count,
            self.trace.fail_assignment.get(),
            "assignment {count}: expected successful common assignment"
        );
    }
}

impl AssignmentItem<'_> {
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

impl Drop for AssignmentItem<'_> {
    fn drop(&mut self) {
        if !self.trace.active.get() {
            return;
        }
        let address = std::ptr::from_ref(self).cast();
        let position = self
            .trace
            .slots
            .borrow()
            .iter()
            .position(|&slot| slot == address);
        let slot = position.map_or_else(|| "X".to_string(), |index| index.to_string());
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("D{}@{slot},", self.value));
    }
}
