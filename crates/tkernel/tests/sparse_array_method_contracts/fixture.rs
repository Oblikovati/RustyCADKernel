//! Instance-owned traces distinguish copy construction from assignment and destruction.
use rustycad_tkernel::ncollection::SparseArray;
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
pub struct SparseTrace {
    pub events: String,
    pub rejected: Option<i32>,
}

pub struct SparseTrackedValue {
    pub value: i32,
    pub trace: Rc<RefCell<SparseTrace>>,
}

impl SparseTrackedValue {
    pub fn new(value: i32, trace: &Rc<RefCell<SparseTrace>>) -> Self {
        Self {
            value,
            trace: Rc::clone(trace),
        }
    }
}

impl Clone for SparseTrackedValue {
    fn clone(&self) -> Self {
        let mut trace = self.trace.borrow_mut();
        trace.events.push_str(&format!("C{},", self.value));
        assert_ne!(
            trace.rejected,
            Some(self.value),
            "copy value {}; expected an accepted value",
            self.value
        );
        Self::new(self.value, &self.trace)
    }

    fn clone_from(&mut self, source: &Self) {
        self.trace
            .borrow_mut()
            .events
            .push_str(&format!("A{}>{},", self.value, source.value));
        self.value = source.value;
    }
}

impl Drop for SparseTrackedValue {
    fn drop(&mut self) {
        self.trace
            .borrow_mut()
            .events
            .push_str(&format!("D{},", self.value));
    }
}

pub fn tracked_sparse_array(
    increment: usize,
    values: &[(usize, i32)],
    trace: &Rc<RefCell<SparseTrace>>,
) -> SparseArray<SparseTrackedValue> {
    let mut array = SparseArray::new(increment);
    for &(index, value) in values {
        array.set_value(index, SparseTrackedValue::new(value, trace));
    }
    array
}

pub fn assert_sparse_oracle_row(actual: &str) {
    let expected = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/oracle/tkernel_sparse_array_method_contracts.expected"
    ));
    assert!(
        expected.lines().any(|line| line == actual),
        "oracle row {actual:?}; expected an exact native output row"
    );
}
