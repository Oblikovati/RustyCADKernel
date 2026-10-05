use rustycad_tkernel::ncollection::{FlatDataMap, FlatMap, HashPolicy};
use std::{cell::RefCell, rc::Rc};

pub type CopyTrace = Rc<RefCell<String>>;

#[derive(Debug)]
pub struct FlatCopyElement {
    pub number: i32,
    pub role: char,
    pub trace: CopyTrace,
}

impl Clone for FlatCopyElement {
    fn clone(&self) -> Self {
        self.trace.borrow_mut().push(self.role);
        Self {
            number: self.number,
            role: self.role,
            trace: Rc::clone(&self.trace),
        }
    }
}

impl Drop for FlatCopyElement {
    fn drop(&mut self) {
        self.trace.borrow_mut().push(self.role.to_ascii_lowercase());
    }
}

pub struct FlatCopyPolicy {
    pub salt: usize,
    pub trace: CopyTrace,
}

impl Clone for FlatCopyPolicy {
    fn clone(&self) -> Self {
        self.trace.borrow_mut().push('C');
        Self {
            salt: self.salt,
            trace: Rc::clone(&self.trace),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        source.trace.borrow_mut().push('A');
        self.salt = source.salt;
        self.trace = Rc::clone(&source.trace);
    }
}

impl HashPolicy<FlatCopyElement> for FlatCopyPolicy {
    fn hash(&self, key: &FlatCopyElement) -> usize {
        self.trace.borrow_mut().push('H');
        key.number as usize + self.salt
    }
    fn equal(&self, left: &FlatCopyElement, right: &FlatCopyElement) -> bool {
        left.number == right.number
    }
}

pub fn copy_element(number: i32, role: char, trace: &CopyTrace) -> FlatCopyElement {
    FlatCopyElement {
        number,
        role,
        trace: Rc::clone(trace),
    }
}

pub fn copy_set(
    trace: &CopyTrace,
    number: i32,
    salt: usize,
) -> FlatMap<FlatCopyElement, FlatCopyPolicy> {
    let policy: FlatCopyPolicy = FlatCopyPolicy {
        salt,
        trace: Rc::clone(trace),
    };
    let mut set: FlatMap<FlatCopyElement, FlatCopyPolicy> = FlatMap::with_hasher(policy, 40);
    set.add(copy_element(number, 'K', trace));
    set
}

pub fn copy_dictionary(
    trace: &CopyTrace,
    number: i32,
    salt: usize,
) -> FlatDataMap<FlatCopyElement, FlatCopyElement, FlatCopyPolicy> {
    let policy: FlatCopyPolicy = FlatCopyPolicy {
        salt,
        trace: Rc::clone(trace),
    };
    let mut pairs: FlatDataMap<FlatCopyElement, FlatCopyElement, FlatCopyPolicy> =
        FlatDataMap::with_hasher(policy, 40);
    pairs.bind(
        copy_element(number, 'K', trace),
        copy_element(number * 10, 'V', trace),
    );
    pairs
}
