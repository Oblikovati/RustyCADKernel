use rustycad_tkernel::ncollection::{FlatDataMap, FlatMap, HashPolicy};
use std::{cell::RefCell, rc::Rc};

pub type InsertionTrace = Rc<RefCell<String>>;
pub struct InsertionHash(pub InsertionTrace);

impl HashPolicy<i32> for InsertionHash {
    fn hash(&self, key: &i32) -> usize {
        self.0.borrow_mut().push('H');
        *key as usize
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}

pub fn insertion_set(trace: &InsertionTrace) -> FlatMap<i32, InsertionHash> {
    let mut map: FlatMap<i32, InsertionHash> =
        FlatMap::with_hasher(InsertionHash(Rc::clone(trace)), 0);
    map.add(1);
    map
}

pub fn insertion_dictionary(trace: &InsertionTrace) -> FlatDataMap<i32, i32, InsertionHash> {
    let mut map: FlatDataMap<i32, i32, InsertionHash> = empty_insertion_dictionary(trace);
    map.bind(1, 10);
    map
}
pub fn empty_insertion_dictionary(trace: &InsertionTrace) -> FlatDataMap<i32, i32, InsertionHash> {
    FlatDataMap::with_hasher(InsertionHash(Rc::clone(trace)), 0)
}
