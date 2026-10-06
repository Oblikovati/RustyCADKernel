use rustycad_tkernel::ncollection::DataMap;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[path = "../support/traced_collision_hash.rs"]
mod hashing;
pub use hashing::CollisionHash;
pub type TestDataMap = DataMap<i32, i32, CollisionHash>;
pub fn populated(buckets: usize) -> TestDataMap {
    let mut map = DataMap::with_hasher(CollisionHash::default(), buckets);
    for key in 1..=3 {
        map.bind(key, key * 10);
    }
    map
}
pub fn keys<V>(map: &DataMap<i32, V, CollisionHash>) -> String {
    map.items().map(|(key, _)| format!("{key},")).collect()
}
pub fn native_row(prefix: &str, actual: String) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_data_map_storage_contracts.expected");
    assert_eq!(
        actual,
        expected
            .lines()
            .find(|line| line.starts_with(prefix))
            .unwrap()
    );
}
pub fn caught(operation: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err()
}
#[derive(Clone, Default)]
pub struct DropTrace {
    pub events: Rc<RefCell<Vec<i32>>>,
    pub reject: Rc<Cell<Option<i32>>>,
}
pub struct DropValue {
    pub number: i32,
    pub trace: DropTrace,
}
impl DropTrace {
    pub fn value(&self, number: i32) -> DropValue {
        DropValue {
            number,
            trace: self.clone(),
        }
    }
}
impl Drop for DropValue {
    fn drop(&mut self) {
        self.trace.events.borrow_mut().push(self.number);
        if self.trace.reject.get() == Some(self.number) {
            self.trace.reject.set(None);
            panic!("drop value {} rejected", self.number);
        }
    }
}
