use rustycad_tkernel::standard::Failure;
use std::{cell::RefCell, rc::Rc};
pub type Calls = Rc<RefCell<String>>;

#[derive(Debug)]
pub struct OwnedProbe {
    pub number: i32,
    calls: Calls,
}
impl OwnedProbe {
    fn new(number: i32, calls: &Calls) -> Self {
        calls.borrow_mut().push('c');
        Self {
            number,
            calls: Rc::clone(calls),
        }
    }
}
impl Drop for OwnedProbe {
    fn drop(&mut self) {
        self.calls.borrow_mut().push('d');
    }
}

#[derive(Debug)]
pub struct WritableProbe<'a> {
    slot: &'a mut i32,
    calls: Calls,
}
impl WritableProbe<'_> {
    /// Writes through the native proxy projection; `proxy.set(17)`.
    pub fn set(&mut self, value: i32) {
        self.calls.borrow_mut().push('=');
        *self.slot = value;
    }
}
impl Drop for WritableProbe<'_> {
    fn drop(&mut self) {
        self.calls.borrow_mut().push('q');
    }
}

pub struct ProxyMap {
    pub values: [i32; 2],
    calls: Calls,
}
impl ProxyMap {
    /// Creates an owner with observable getters; `ProxyMap::new(&calls)`.
    pub fn new(calls: &Calls) -> Self {
        Self {
            values: [10, 20],
            calls: Rc::clone(calls),
        }
    }
    /// Records the native Size call; `map.size()`.
    pub fn size(&self) -> usize {
        self.calls.borrow_mut().push('s');
        self.values.len()
    }
    fn check(&self, index: usize) -> Result<(), Failure> {
        if (1..=self.values.len()).contains(&index) {
            return Ok(());
        }
        self.calls.borrow_mut().push('e');
        Err(Failure::with_type(
            "Standard_OutOfRange",
            &format!("index={index}; expected 1..=2"),
            "",
        ))
    }
    /// Returns an owned non-Clone result; `map.read(1)?`.
    pub fn read(&self, index: usize) -> Result<OwnedProbe, Failure> {
        self.calls.borrow_mut().push_str(&format!("r{index}"));
        self.check(index)?;
        Ok(OwnedProbe::new(self.values[index - 1], &self.calls))
    }
    /// Projects a writable proxy without copying its owner; `map.write(1)?`.
    pub fn write(&mut self, index: usize) -> Result<WritableProbe<'_>, Failure> {
        self.calls.borrow_mut().push_str(&format!("w{index}"));
        self.check(index)?;
        self.calls.borrow_mut().push('p');
        Ok(WritableProbe {
            slot: &mut self.values[index - 1],
            calls: Rc::clone(&self.calls),
        })
    }
    /// Selects native nonconst FindKey independently; `map.mutable_key(1)?`.
    pub fn mutable_key(&mut self, index: usize) -> Result<i32, Failure> {
        self.calls.borrow_mut().push_str(&format!("K{index}"));
        self.check(index)?;
        Ok(index as i32 * 111)
    }
    /// Returns an owned key independent of the value type; `map.key(1)?`.
    pub fn key(&self, index: usize) -> Result<i32, Failure> {
        self.calls.borrow_mut().push_str(&format!("k{index}"));
        self.check(index)?;
        Ok(index as i32 * 11)
    }
}

/// Returns and clears accessor events; `take_trace(&calls)`.
pub fn take_trace(calls: &Calls) -> String {
    let trace = std::mem::take(&mut *calls.borrow_mut());
    if trace.is_empty() {
        "-".to_owned()
    } else {
        trace
    }
}
