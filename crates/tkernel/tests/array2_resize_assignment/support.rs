use rustycad_tkernel::ncollection::Array2;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Default)]
pub struct ResizeTrace {
    pub events: RefCell<Vec<String>>,
    pub failure_value: Cell<i32>,
}

// Default has no parameter for a test recorder. The guard injects one per test thread;
// constructed items retain their own Rc so teardown never depends on ambient context.
thread_local! {
    static DEFAULT_TRACE: RefCell<Option<Rc<ResizeTrace>>> = const { RefCell::new(None) };
}

pub struct ResizeTraceScope {
    pub trace: Rc<ResizeTrace>,
    previous: Option<Rc<ResizeTrace>>,
}

impl ResizeTraceScope {
    pub fn new() -> Self {
        let trace = Rc::new(ResizeTrace::default());
        let previous = DEFAULT_TRACE.with(|slot| slot.replace(Some(Rc::clone(&trace))));
        Self { trace, previous }
    }
    pub fn clear(&self, failure: i32) {
        self.trace.events.borrow_mut().clear();
        self.trace.failure_value.set(failure);
    }
    pub fn row(&self) -> String {
        self.trace.events.borrow().join(",")
    }
}

impl Drop for ResizeTraceScope {
    fn drop(&mut self) {
        DEFAULT_TRACE.with(|slot| slot.replace(self.previous.take()));
    }
}

pub struct ResizeItem {
    pub value: i32,
    pub trace: Rc<ResizeTrace>,
}

impl ResizeItem {
    pub fn supplied(value: i32, trace: &Rc<ResizeTrace>) -> Self {
        Self {
            value,
            trace: Rc::clone(trace),
        }
    }
}

impl Default for ResizeItem {
    fn default() -> Self {
        let trace = DEFAULT_TRACE.with(|slot| {
            Rc::clone(
                slot.borrow()
                    .as_ref()
                    .expect("expected an active ResizeTraceScope"),
            )
        });
        trace.events.borrow_mut().push("D".into());
        Self { value: 0, trace }
    }
}

impl Clone for ResizeItem {
    fn clone(&self) -> Self {
        self.trace
            .events
            .borrow_mut()
            .push(format!("C{}", self.value));
        Self::supplied(self.value, &self.trace)
    }
    fn clone_from(&mut self, donor: &Self) {
        record_assignment(self, donor, "Q");
        self.value = donor.value;
    }
}

impl Drop for ResizeItem {
    fn drop(&mut self) {
        self.trace
            .events
            .borrow_mut()
            .push(format!("X{}", self.value));
    }
}

fn record_assignment(target: &ResizeItem, donor: &ResizeItem, kind: &str) {
    target
        .trace
        .events
        .borrow_mut()
        .push(format!("{kind}{}:{}", target.value, donor.value));
    assert_ne!(
        donor.value,
        target.trace.failure_value.get(),
        "donor value {}: expected a value outside the configured assignment failure",
        donor.value
    );
}

pub fn assign_move(target: &mut ResizeItem, donor: &mut ResizeItem) {
    record_assignment(target, donor, "A");
    target.value = donor.value;
    donor.value = -donor.value;
}

pub fn initialized_owned() -> Array2<'static, ResizeItem> {
    let mut array = Array2::<ResizeItem>::with_bounds(-2, -1, 3, 5).unwrap();
    for (index, item) in array.as_mut_slice().iter_mut().enumerate() {
        item.value = index as i32 + 1;
    }
    array
}

pub fn values_row(values: &[ResizeItem]) -> String {
    values
        .iter()
        .map(|item| item.value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn state(array: &Array2<'_, ResizeItem>) -> String {
    format!(
        "{},{};{},{};{},{};{};{}",
        array.nb_rows(),
        array.nb_columns(),
        array.lower_row(),
        array.lower_col(),
        array.array1().lower(),
        array.array1().upper(),
        u8::from(array.is_deletable()),
        values_row(array.as_slice())
    )
}

pub fn assert_native_row(label: &str, value: impl std::fmt::Display) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_array2_resize_assignment.expected");
    let row = format!("{label}={value}");
    assert!(
        expected.lines().any(|line| line == row),
        "missing native row: {row}"
    );
}

pub fn resize_copy(array: &mut Array2<'_, ResizeItem>, trim: bool) {
    if trim {
        array
            .resize_trim_bounds_with_assignment(7, 8, -3, -2, true, assign_move)
            .unwrap();
        return;
    }
    array
        .resize_bounds_with_assignment(7, 8, -3, -2, true, assign_move)
        .unwrap();
}
