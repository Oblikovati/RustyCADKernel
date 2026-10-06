//! Native whole-vector moves preserve payload identity without element callbacks.
use rustycad_tkernel::ncollection::LinearVector;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Default)]
struct MoveTrace {
    events: RefCell<String>,
    panic_once: Cell<bool>,
}

struct MoveItem {
    value: Box<i32>,
    trace: Rc<MoveTrace>,
}

impl MoveItem {
    fn new(value: i32, trace: &Rc<MoveTrace>) -> Self {
        Self {
            value: Box::new(value),
            trace: Rc::clone(trace),
        }
    }
}

impl Drop for MoveItem {
    fn drop(&mut self) {
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("D{},", self.value));
        if *self.value == 20 && self.trace.panic_once.replace(false) {
            panic!("destructor value 20: expected non-panicking native destructor");
        }
    }
}

type MoveVector = LinearVector<MoveItem>;

fn filled(capacity: usize, count: usize, base: i32, trace: &Rc<MoveTrace>) -> MoveVector {
    let mut vector = MoveVector::with_capacity(capacity);
    for index in 0..count {
        vector.emplace_append(|| MoveItem::new(base + i32::try_from(index).unwrap(), trace));
    }
    vector
}

struct MoveSnapshot {
    buffer: *const MoveItem,
    payload: *const i32,
}

impl MoveSnapshot {
    fn capture(vector: &MoveVector) -> Self {
        let payload = vector
            .first()
            .map_or(std::ptr::null(), |item| std::ptr::from_ref(&*item.value));
        Self {
            buffer: vector.data_ptr(),
            payload,
        }
    }

    fn identity_row(&self, moved: &MoveVector) -> String {
        let stable = moved
            .first()
            .ok()
            .is_none_or(|item| std::ptr::eq(self.payload, &*item.value));
        format!(
            "{}:{}",
            u8::from(self.buffer == moved.data_ptr()),
            u8::from(stable)
        )
    }
}

fn fixture_row(row: &str) {
    let expected = include_str!("../../../tools/oracle/tkernel_linear_moves.expected");
    assert!(
        expected.lines().any(|line| line == row),
        "native oracle missing row {row:?}"
    );
}

fn movement_row(
    name: &str,
    destination: &MoveVector,
    source: &MoveVector,
    before: &MoveSnapshot,
    trace: &MoveTrace,
) {
    let values: String = destination
        .iter()
        .map(|item| format!("{},", item.value))
        .collect();
    let source_state = source_state(source);
    fixture_row(&format!(
        "{name}:{}:{}:{}:{source_state}:{}:{values}",
        destination.len(),
        destination.capacity(),
        before.identity_row(destination),
        trace.events.borrow()
    ));
}

fn source_state(source: &MoveVector) -> String {
    format!(
        "{}:{}:{}",
        source.len(),
        source.capacity(),
        u8::from(source.data_ptr().is_null())
    )
}

fn construct_case(name: &str, capacity: usize, count: usize, retained: bool) {
    let trace = Rc::new(MoveTrace::default());
    let mut source = filled(capacity, count, 10, &trace);
    if retained {
        source.clear(false);
    }
    let before = MoveSnapshot::capture(&source);
    trace.events.borrow_mut().clear();
    let destination = MoveVector::from_moved(&mut source);
    movement_row(name, &destination, &source, &before, &trace);
    source.emplace_append(|| MoveItem::new(99, &trace));
    assert_eq!((source.len(), *source[0].value), (1, 99));
    assert_eq!(destination.len(), if retained { 0 } else { count });
}

#[test]
fn move_constructor_handles_empty_and_reserved_sources() {
    construct_case("empty", 0, 0, false);
    construct_case("reserved", 7, 0, false);
}

#[test]
fn move_constructor_preserves_buffer_and_payload_addresses() {
    construct_case("single", 1, 1, false);
    construct_case("multiple", 7, 3, false);
}

#[test]
fn move_constructor_transfers_retained_empty_storage() {
    construct_case("retained", 7, 3, true);
}

fn assign_case(name: &str, capacity: usize, count: usize) {
    let trace = Rc::new(MoveTrace::default());
    let mut source = filled(capacity, count, 10, &trace);
    let mut destination = filled(3, 2, 20, &trace);
    let before = MoveSnapshot::capture(&source);
    let receiver = std::ptr::from_ref(&destination);
    trace.events.borrow_mut().clear();
    assert!(std::ptr::eq(receiver, destination.move_from(&mut source)));
    movement_row(name, &destination, &source, &before, &trace);
}

#[test]
fn move_assignment_releases_receiver_before_adopting_source_storage() {
    assign_case("assign-empty", 0, 0);
    assign_case("assign-reserved", 7, 0);
    assign_case("assign-values", 7, 3);
}

#[test]
fn move_self_assignment_keeps_values_and_allocation_without_callbacks() {
    let trace = Rc::new(MoveTrace::default());
    let mut vector = filled(4, 3, 10, &trace);
    let before = vector.data_ptr();
    let receiver = std::ptr::from_ref(&vector);
    assert!(std::ptr::eq(receiver, vector.assign_self()));
    fixture_row(&format!(
        "self:{}:{}:{}:{}",
        vector.len(),
        vector.capacity(),
        u8::from(before == vector.data_ptr()),
        trace.events.borrow()
    ));
}

#[test]
fn receiver_destructor_panic_leaves_source_untouched() {
    let trace = Rc::new(MoveTrace::default());
    let mut source = filled(4, 2, 10, &trace);
    let mut destination = filled(3, 2, 20, &trace);
    let before = MoveSnapshot::capture(&source);
    trace.panic_once.set(true);
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        destination.move_from(&mut source);
    }));
    assert!(failed.is_err());
    assert_eq!((source.len(), source.capacity()), (2, 4));
    assert_eq!(before.identity_row(&source), "1:1");
    assert_eq!((destination.len(), destination.capacity()), (0, 0));
    assert_eq!(*trace.events.borrow(), "D20,D21,");
    assert_eq!((*source[0].value, *source[1].value), (10, 11));
}

#[test]
fn constructor_and_assignment_keep_borrowed_values_and_public_traits() {
    fn traits<T: Send + Sync + Unpin + std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
    fn shorten<'a, 'b: 'a>(vector: LinearVector<&'b str>) -> LinearVector<&'a str> {
        vector
    }
    traits::<LinearVector<i32>>();
    let owner = String::from("borrowed");
    let mut source = LinearVector::with_capacity(3);
    source.append(owner.as_str());
    let mut moved = shorten(LinearVector::from_moved(&mut source));
    let mut destination = LinearVector::default();
    destination.move_from(&mut moved);
    assert_eq!(destination.as_slice(), [owner.as_str()]);
    assert!(source.is_empty() && moved.is_empty());
}
