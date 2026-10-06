use rustycad_tkernel::ncollection::SparseArray;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone, Default)]
pub struct TraceLog {
    events: Rc<RefCell<String>>,
    pub reject_create: Rc<Cell<i32>>,
    pub reject_assign: Rc<Cell<i32>>,
    pub reject_drop: Rc<Cell<i32>>,
}
impl TraceLog {
    pub fn new() -> Self {
        let result = Self::default();
        result.reject_create.set(-1);
        result.reject_assign.set(-1);
        result.reject_drop.set(-1);
        result
    }
    pub fn item(&self, value: i32) -> TraceItem {
        TraceItem {
            value,
            trace: self.clone(),
        }
    }
    pub fn clear(&self) {
        self.events.borrow_mut().clear();
    }
    pub fn events(&self) -> String {
        self.events.borrow().clone()
    }
    pub fn array(&self, increment: usize, values: &[(usize, i32)]) -> SparseArray<TraceItem> {
        let mut result = SparseArray::new(increment);
        for &(index, value) in values {
            result.set_value(index, self.item(value));
        }
        result
    }
}
pub struct TraceItem {
    pub value: i32,
    trace: TraceLog,
}
impl Clone for TraceItem {
    fn clone(&self) -> Self {
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("C{},", self.value));
        assert_ne!(
            self.value,
            self.trace.reject_create.get(),
            "rejected clone {}",
            self.value
        );
        self.trace.item(self.value)
    }
    fn clone_from(&mut self, source: &Self) {
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("A{}>{},", self.value, source.value));
        self.value = source.value;
        assert_ne!(
            self.value,
            self.trace.reject_assign.get(),
            "rejected assignment {}",
            self.value
        );
    }
}
impl Drop for TraceItem {
    fn drop(&mut self) {
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("D{},", self.value));
        if self.trace.reject_drop.get() == self.value {
            self.trace.reject_drop.set(-1);
            panic!("rejected destruction {}", self.value);
        }
    }
}
pub fn assert_native_row(prefix: &str, actual: String) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_sparse_typed_storage_contracts.expected");
    let row = expected
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap();
    assert_eq!(actual, row);
}
pub fn caught(operation: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err()
}
