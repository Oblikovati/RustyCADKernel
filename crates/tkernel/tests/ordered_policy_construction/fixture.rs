use rustycad_tkernel::ncollection::HashPolicy;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Default)]
pub struct Trace {
    pub events: RefCell<String>,
    pub reject: Cell<bool>,
}
impl Trace {
    pub fn callback(&self, event: char) {
        self.events.borrow_mut().push(event);
        if self.reject.get() {
            self.events.borrow_mut().push('X');
            panic!("policy callback {event}; expected an accepted construction");
        }
    }
    pub fn text(&self) -> String {
        let text = self.events.borrow();
        if text.is_empty() {
            "-".to_owned()
        } else {
            text.clone()
        }
    }
}
pub struct Policy {
    pub state: i32,
    pub trace: Rc<Trace>,
}
impl Policy {
    pub fn new(state: i32, reject: bool) -> Self {
        let trace = Rc::new(Trace::default());
        trace.reject.set(reject);
        Self { state, trace }
    }
    pub fn take(source: &mut Self) -> Self {
        let state = std::mem::replace(&mut source.state, -99);
        source.trace.callback('M');
        Self {
            state,
            trace: Rc::clone(&source.trace),
        }
    }
}
impl Clone for Policy {
    fn clone(&self) -> Self {
        self.trace.callback('C');
        Self {
            state: self.state,
            trace: Rc::clone(&self.trace),
        }
    }
}
impl HashPolicy<i32> for Policy {
    fn hash(&self, key: &i32) -> usize {
        (key % 3) as usize
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}
pub struct MoveOnly(pub Policy);
impl MoveOnly {
    pub fn take(source: &mut Self) -> Self {
        Self(Policy::take(&mut source.0))
    }
}
impl HashPolicy<i32> for MoveOnly {
    fn hash(&self, key: &i32) -> usize {
        self.0.hash(key)
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}
pub fn native_row(label: &str, actual: String) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_ordered_policy_construction.expected")
            .lines()
            .find(|line| line.starts_with(&format!("{label} ")))
            .unwrap();
    assert_eq!(actual, expected);
}
