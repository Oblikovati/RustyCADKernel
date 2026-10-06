use rustycad_tkernel::ncollection::HashPolicy;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Default)]
pub struct SwapTrace {
    pub events: RefCell<String>,
    pub phase: Cell<usize>,
    pub rejected: Cell<usize>,
    pub drops: Cell<usize>,
}
impl SwapTrace {
    pub fn event(&self, tag: char, state: i32) {
        self.events.borrow_mut().push_str(&format!("{tag}{state},"));
        self.phase.set(self.phase.get() + 1);
        if self.phase.get() == self.rejected.get() {
            self.events.borrow_mut().push_str("X,");
            panic!(
                "policy phase {}; expected an accepted swap",
                self.phase.get()
            );
        }
    }
}
pub struct Payload {
    pub key: i32,
    pub trace: Rc<SwapTrace>,
}
impl Payload {
    pub fn new(key: i32, trace: &Rc<SwapTrace>) -> Self {
        Self {
            key,
            trace: Rc::clone(trace),
        }
    }
}
impl Drop for Payload {
    fn drop(&mut self) {
        self.trace.drops.set(self.trace.drops.get() + 1);
    }
}
pub struct SwapPolicy {
    pub state: i32,
    pub trace: Rc<SwapTrace>,
}
impl SwapPolicy {
    pub fn new(state: i32, trace: &Rc<SwapTrace>) -> Self {
        Self {
            state,
            trace: Rc::clone(trace),
        }
    }
    fn take(source: &mut Self) -> Self {
        let state = std::mem::replace(&mut source.state, -99);
        source.trace.event('M', state);
        Self::new(state, &source.trace)
    }
    fn assign(&mut self, source: &mut Self) {
        self.state = std::mem::replace(&mut source.state, -99);
        self.trace.event('A', self.state);
    }
    pub fn native_self_swap(policy: &mut Self) {
        let mut temporary = Self::take(policy);
        policy.state = -99;
        policy.trace.event('A', policy.state);
        policy.assign(&mut temporary);
    }
    pub fn native_swap(first: &mut Self, second: &mut Self) {
        let mut temporary = Self::take(first);
        first.assign(second);
        second.assign(&mut temporary);
    }
}
impl Drop for SwapPolicy {
    fn drop(&mut self) {
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("D{},", self.state));
    }
}
impl HashPolicy<Payload> for SwapPolicy {
    fn hash(&self, key: &Payload) -> usize {
        (key.key % 3) as usize
    }
    fn equal(&self, left: &Payload, right: &Payload) -> bool {
        left.key == right.key
    }
}
pub fn native_row(label: &str, row: String) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_ordered_policy_exchange.expected")
            .lines()
            .find(|line| line.starts_with(&format!("{label} ")))
            .unwrap();
    assert_eq!(row, expected);
}
