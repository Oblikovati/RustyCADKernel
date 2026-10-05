use rustycad_tkernel::ncollection::{FlatDataMap, FlatMap, HashPolicy};
use std::{cell::RefCell, rc::Rc};

pub type MoveTrace = Rc<RefCell<String>>;

#[derive(Debug)]
pub struct MoveElement {
    pub number: i32,
    pub role: char,
    pub trace: MoveTrace,
}
impl Drop for MoveElement {
    fn drop(&mut self) {
        self.trace.borrow_mut().push(self.role);
    }
}

pub struct MovePolicy {
    pub salt: i32,
    pub trace: MoveTrace,
}
impl MovePolicy {
    pub fn take_policy(source: &mut Self) -> Self {
        source.trace.borrow_mut().push('M');
        Self {
            salt: std::mem::replace(&mut source.salt, -1),
            trace: Rc::clone(&source.trace),
        }
    }
    pub fn assign_policy(&mut self, source: &mut Self) {
        source.trace.borrow_mut().push('A');
        self.salt = std::mem::replace(&mut source.salt, -1);
        self.trace = Rc::clone(&source.trace);
    }
}
impl Clone for MovePolicy {
    fn clone(&self) -> Self {
        self.trace.borrow_mut().push('C');
        Self {
            salt: self.salt,
            trace: Rc::clone(&self.trace),
        }
    }
}
impl Drop for MovePolicy {
    fn drop(&mut self) {
        self.trace.borrow_mut().push('P');
    }
}
impl HashPolicy<MoveElement> for MovePolicy {
    fn hash(&self, key: &MoveElement) -> usize {
        self.trace.borrow_mut().push('H');
        key.number.wrapping_add(self.salt) as usize
    }
    fn equal(&self, left: &MoveElement, right: &MoveElement) -> bool {
        left.number == right.number
    }
}

pub fn move_element(number: i32, role: char, trace: &MoveTrace) -> MoveElement {
    MoveElement {
        number,
        role,
        trace: Rc::clone(trace),
    }
}
pub fn move_policy(trace: &MoveTrace, salt: i32) -> MovePolicy {
    MovePolicy {
        salt,
        trace: Rc::clone(trace),
    }
}
pub fn move_set(trace: &MoveTrace, number: i32, salt: i32) -> FlatMap<MoveElement, MovePolicy> {
    let mut set = FlatMap::with_hasher(move_policy(trace, salt), 40);
    set.add(move_element(number, 'k', trace));
    set
}
pub fn move_dictionary(
    trace: &MoveTrace,
    number: i32,
    salt: i32,
) -> FlatDataMap<MoveElement, MoveElement, MovePolicy> {
    let mut map = FlatDataMap::with_hasher(move_policy(trace, salt), 40);
    map.bind(
        move_element(number, 'k', trace),
        move_element(number * 10, 'v', trace),
    );
    map
}
