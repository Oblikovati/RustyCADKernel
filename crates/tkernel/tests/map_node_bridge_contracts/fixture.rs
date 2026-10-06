use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone)]
pub struct KeyTrace {
    pub events: Rc<RefCell<String>>,
    pub allocated: Rc<Cell<bool>>,
    pub reject_copy: Rc<Cell<bool>>,
    pub reject_drop: Rc<Cell<bool>>,
    pub arena: BaseAllocator,
}
impl KeyTrace {
    pub fn new() -> Self {
        Self {
            events: Default::default(),
            allocated: Default::default(),
            reject_copy: Default::default(),
            reject_drop: Default::default(),
            arena: BaseAllocator::incremental(512),
        }
    }
    pub fn key(&self) -> TracedKey {
        TracedKey(self.clone())
    }
    pub fn events(&self) -> String {
        self.events.borrow().clone()
    }
    pub fn live(&self) -> bool {
        self.arena.reset(false).is_err()
    }
}
pub struct TracedKey(pub KeyTrace);
impl Clone for TracedKey {
    fn clone(&self) -> Self {
        self.0.events.borrow_mut().push_str("C,");
        self.0.allocated.set(self.0.live());
        assert!(
            !self.0.reject_copy.get(),
            "key copy rejected; expected accepted key"
        );
        Self(self.0.clone())
    }
}
impl Drop for TracedKey {
    fn drop(&mut self) {
        self.0.events.borrow_mut().push_str("D,");
        assert!(
            !self.0.reject_drop.replace(false),
            "key drop rejected; expected accepted key"
        );
    }
}
pub fn native_row(prefix: &str, actual: String) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_map_node_bridge_contracts.expected");
    assert_eq!(
        actual,
        expected
            .lines()
            .find(|row| row.starts_with(prefix))
            .unwrap()
    );
}
pub fn caught(operation: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err()
}
#[derive(Default, Clone)]
pub struct Collision;
impl HashPolicy<i32> for Collision {
    fn hash(&self, _: &i32) -> usize {
        0
    }
    fn equal(&self, a: &i32, b: &i32) -> bool {
        a == b
    }
}
