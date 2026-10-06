use rustycad_tkernel::ncollection::HashPolicy;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone, Default)]
pub struct CollisionHash {
    pub calls: Rc<RefCell<String>>,
    pub rejected: Rc<Cell<Option<i32>>>,
}
impl HashPolicy<i32> for CollisionHash {
    fn hash(&self, key: &i32) -> usize {
        self.calls.borrow_mut().push_str(&format!("{key},"));
        assert_ne!(
            Some(*key),
            self.rejected.get(),
            "hash of key {key}; expected accepted key"
        );
        0
    }
    fn equal(&self, first: &i32, second: &i32) -> bool {
        first == second
    }
}
