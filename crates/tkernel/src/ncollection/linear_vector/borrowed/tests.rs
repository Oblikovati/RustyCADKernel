use super::super::LinearVector;
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
mod contracts;
mod oracle;

#[derive(Default)]
struct BorrowTrace {
    fail_copy: Cell<bool>,
    fail_assign: Cell<bool>,
    events: RefCell<String>,
}
struct BorrowedItem {
    value: i32,
    trace: Rc<BorrowTrace>,
}
impl BorrowedItem {
    fn new(value: i32, trace: &Rc<BorrowTrace>) -> Self {
        Self {
            value,
            trace: Rc::clone(trace),
        }
    }
    fn default_in(trace: &Rc<BorrowTrace>) -> Self {
        trace.events.borrow_mut().push('D');
        Self::new(0, trace)
    }
}
impl Clone for BorrowedItem {
    fn clone(&self) -> Self {
        self.trace.events.borrow_mut().push('C');
        assert!(!self.trace.fail_copy.get(), "copy constructor failure");
        Self::new(self.value, &self.trace)
    }
    fn clone_from(&mut self, donor: &Self) {
        self.trace.events.borrow_mut().push('A');
        self.value = donor.value + 100;
        assert!(!self.trace.fail_assign.get(), "copy assignment failure");
        self.value = donor.value;
    }
}
fn row(name: &str, target: &LinearVector<BorrowedItem>, donor: &BorrowedItem) -> String {
    let values = target
        .iter()
        .map(|item| item.value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{name} {} {} {} {} {}\n",
        target.len(),
        target.capacity(),
        donor.value,
        donor.trace.events.borrow(),
        if values.is_empty() { "-" } else { &values }
    )
}
