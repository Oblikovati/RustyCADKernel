use rustycad_tkernel::ncollection::{FlatDataMap, FlatMap, HashPolicy};
use std::{cell::RefCell, rc::Rc};
pub type BorrowTrace = Rc<RefCell<String>>;
#[derive(Debug)]
pub struct BorrowElement {
    pub number: i32,
    role: char,
    trace: BorrowTrace,
}
impl Clone for BorrowElement {
    fn clone(&self) -> Self {
        self.trace.borrow_mut().push(self.role);
        borrow_element(self.number, self.role, &self.trace)
    }
}
pub struct BorrowHash(pub BorrowTrace);
impl HashPolicy<BorrowElement> for BorrowHash {
    fn hash(&self, key: &BorrowElement) -> usize {
        self.0.borrow_mut().push('H');
        key.number as usize
    }
    fn equal(&self, left: &BorrowElement, right: &BorrowElement) -> bool {
        self.0.borrow_mut().push('E');
        left.number == right.number
    }
}
pub fn borrow_element(number: i32, role: char, trace: &BorrowTrace) -> BorrowElement {
    BorrowElement {
        number,
        role,
        trace: Rc::clone(trace),
    }
}
pub fn borrowed_set(trace: &BorrowTrace) -> FlatMap<BorrowElement, BorrowHash> {
    let mut map: FlatMap<BorrowElement, BorrowHash> =
        FlatMap::with_hasher(BorrowHash(Rc::clone(trace)), 0);
    map.add(borrow_element(1, 'K', trace));
    map
}
pub fn borrowed_dictionary(
    trace: &BorrowTrace,
) -> FlatDataMap<BorrowElement, BorrowElement, BorrowHash> {
    let mut map: FlatDataMap<BorrowElement, BorrowElement, BorrowHash> =
        FlatDataMap::with_hasher(BorrowHash(Rc::clone(trace)), 0);
    map.bind(
        borrow_element(1, 'K', trace),
        borrow_element(10, 'V', trace),
    );
    map
}
