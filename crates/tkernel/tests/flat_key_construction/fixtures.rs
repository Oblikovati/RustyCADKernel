use rustycad_tkernel::ncollection::HashPolicy;
use std::{cell::RefCell, rc::Rc};

pub type ConstructionTrace = Rc<RefCell<String>>;

pub struct ConstructedKey {
    pub number: i32,
    pub serial: i32,
}

pub struct ConstructionPolicy {
    pub trace: ConstructionTrace,
}
impl HashPolicy<ConstructedKey> for ConstructionPolicy {
    fn hash(&self, key: &ConstructedKey) -> usize {
        self.trace.borrow_mut().push('H');
        key.number as usize
    }
    fn equal(&self, left: &ConstructedKey, right: &ConstructedKey) -> bool {
        self.trace.borrow_mut().push('E');
        left.number == right.number
    }
}
pub fn constructed_key(number: i32, serial: i32, trace: &ConstructionTrace) -> ConstructedKey {
    trace.borrow_mut().push('C');
    ConstructedKey { number, serial }
}
pub fn failed_key_construction(trace: &ConstructionTrace) -> ConstructedKey {
    trace.borrow_mut().push('C');
    panic!("key constructor failed for requested serial -1; expected a nonnegative serial")
}
