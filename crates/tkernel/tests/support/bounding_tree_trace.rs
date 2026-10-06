use rustycad_tkernel::ncollection::BoundingVolume;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone, Default)]
pub struct Trace(Rc<State>);
#[derive(Default)]
struct State {
    events: RefCell<String>,
    rejected: Cell<(char, char)>,
    assignment_addresses: RefCell<Vec<usize>>,
    destructor_addresses: RefCell<Vec<usize>>,
}
thread_local! {
    static DEFAULT_TRACE: RefCell<Trace> = RefCell::new(Trace::default());
}
impl Trace {
    pub fn install_defaults(&self) {
        // Native Default has no recorder argument; each fixture explicitly supplies its trace.
        DEFAULT_TRACE.with(|slot| *slot.borrow_mut() = self.clone());
    }
    pub fn item<const KIND: char>(&self, value: i32) -> Item<KIND> {
        Item {
            value,
            trace: self.clone(),
        }
    }
    pub fn clear(&self) {
        self.0.events.borrow_mut().clear();
        self.0.assignment_addresses.borrow_mut().clear();
        self.0.destructor_addresses.borrow_mut().clear();
    }
    pub fn reject(&self, phase: char, kind: char) {
        self.0.rejected.set((phase, kind));
    }
    pub fn events(&self) -> String {
        self.0.events.borrow().clone()
    }
    pub fn addresses(&self) -> Vec<usize> {
        self.0.assignment_addresses.borrow().clone()
    }
    pub fn destructor_addresses(&self) -> Vec<usize> {
        self.0.destructor_addresses.borrow().clone()
    }
    fn record(&self, phase: char, kind: char, value: i32, address: usize) {
        self.0
            .events
            .borrow_mut()
            .push_str(&format!("{phase}{kind}{value},"));
        if phase == 'A' {
            self.0.assignment_addresses.borrow_mut().push(address);
        }
        if phase == 'D' {
            self.0.destructor_addresses.borrow_mut().push(address);
        }
        if self.0.rejected.get() == (phase, kind) {
            self.0.rejected.set(('\0', '\0'));
            panic!("payload {kind} phase {phase} rejected; expected permitted callback");
        }
    }
}
pub struct Item<const KIND: char> {
    pub value: i32,
    trace: Trace,
}
impl<const KIND: char> Clone for Item<KIND> {
    fn clone(&self) -> Self {
        self.trace.record('C', KIND, self.value, 0);
        self.trace.item(self.value)
    }
    fn clone_from(&mut self, source: &Self) {
        self.value = source.value;
        self.trace
            .record('A', KIND, self.value, std::ptr::from_ref(self).addr());
    }
}
impl<const KIND: char> Default for Item<KIND> {
    fn default() -> Self {
        DEFAULT_TRACE.with(|trace| trace.borrow().item(0))
    }
}
impl<const KIND: char> Drop for Item<KIND> {
    fn drop(&mut self) {
        self.trace
            .record('D', KIND, self.value, std::ptr::from_ref(self).addr());
    }
}
impl BoundingVolume for Item<'B'> {
    fn add(&mut self, _: &Self) {}
    fn is_out(&self, _: &Self) -> bool {
        false
    }
    fn square_extent(&self) -> f64 {
        0.0
    }
}
pub fn caught(operation: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err()
}
