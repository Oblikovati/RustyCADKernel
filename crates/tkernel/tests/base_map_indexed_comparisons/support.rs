use rustycad_tkernel::{ncollection::IndexedCursorSource, standard::Failure};
use std::{cell::RefCell, rc::Rc};

pub struct ProbeMap {
    tag: char,
    extent: usize,
    calls: Rc<RefCell<String>>,
    value: i32,
}

impl ProbeMap {
    /// Shares observable Size tracing; `ProbeMap::new('a', 3, &calls)`.
    pub fn new(tag: char, extent: usize, calls: &Rc<RefCell<String>>) -> Self {
        Self {
            tag,
            extent,
            calls: Rc::clone(calls),
            value: 42,
        }
    }
}

impl IndexedCursorSource for ProbeMap {
    type Key = i32;
    type Value = i32;
    fn indexed_size(&self) -> usize {
        self.calls.borrow_mut().push(self.tag);
        self.extent
    }
    fn indexed_key(&self, _index: usize) -> Result<&i32, Failure> {
        Ok(&self.value)
    }
    fn indexed_value(&self, _index: usize) -> Result<&i32, Failure> {
        Ok(&self.value)
    }
}

/// Returns and clears the call sequence; `take_trace(&calls)`.
pub fn take_trace(calls: &Rc<RefCell<String>>) -> String {
    let trace = std::mem::take(&mut *calls.borrow_mut());
    if trace.is_empty() {
        "-".to_owned()
    } else {
        trace
    }
}
