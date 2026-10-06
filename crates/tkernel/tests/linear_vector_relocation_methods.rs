//! Exact Reserve/grow event order plus LocalArray's protected storage predicate.
use rustycad_tkernel::ncollection::{LinearVector, LocalArray};
use std::cell::RefCell;
use std::rc::Rc;

struct Relocated {
    value: i32,
    trace: Rc<RefCell<String>>,
}

impl Relocated {
    fn new(value: i32, trace: &Rc<RefCell<String>>) -> Self {
        Self {
            value,
            trace: Rc::clone(trace),
        }
    }

    fn move_observed(&mut self) -> Self {
        self.trace
            .borrow_mut()
            .push_str(&format!("M{},", self.value));
        Self::new(std::mem::replace(&mut self.value, -1), &self.trace)
    }

    fn copy_observed(&mut self) -> Self {
        self.trace
            .borrow_mut()
            .push_str(&format!("C{},", self.value));
        Self::new(self.value, &self.trace)
    }
}

impl Drop for Relocated {
    fn drop(&mut self) {
        self.trace
            .borrow_mut()
            .push_str(&format!("D{},", self.value));
    }
}

fn fixture_row(row: &str) {
    let expected = include_str!("../../../tools/oracle/tkernel_linear_relocation.expected");
    assert!(
        expected.lines().any(|line| line == row),
        "native oracle missing row {row:?}"
    );
}

fn filled(capacity: usize, count: usize, trace: &Rc<RefCell<String>>) -> LinearVector<Relocated> {
    let mut vector = LinearVector::with_capacity(capacity);
    for index in 0..count {
        vector.emplace_append(|| Relocated::new(10 + i32::try_from(index).unwrap(), trace));
    }
    trace.borrow_mut().clear();
    vector
}

fn reserve_case(label: &str, capacity: usize, count: usize, minimum: usize, copy: bool) {
    let trace = Rc::new(RefCell::new(String::new()));
    let mut vector = filled(capacity, count, &trace);
    let before = vector.data_ptr();
    let callback = if copy {
        Relocated::copy_observed
    } else {
        Relocated::move_observed
    };
    vector.reserve_with_relocation(minimum, callback);
    check_vector_row(label, &vector, before, &trace);
}

fn check_vector_row(
    label: &str,
    vector: &LinearVector<Relocated>,
    before: *const Relocated,
    trace: &Rc<RefCell<String>>,
) {
    let values: String = vector
        .iter()
        .map(|item| format!("{},", item.value))
        .collect();
    fixture_row(&format!(
        "{label}:{}:{}:{}:{}:{values}",
        vector.len(),
        vector.capacity(),
        u8::from(before == vector.data_ptr()),
        trace.borrow()
    ));
}

#[test]
fn empty_reserve_uses_native_initial_capacity_without_element_callbacks() {
    reserve_case("empty-noop", 0, 0, 0, false);
    reserve_case("empty-grow", 0, 0, 1, false);
}

#[test]
fn sufficient_capacity_skips_relocation_callbacks() {
    reserve_case("retained-noop", 4, 3, 4, false);
}

#[test]
fn noexcept_move_relocation_constructs_all_then_destroys_old_values() {
    reserve_case("move-double", 4, 3, 5, false);
    reserve_case("move-full", 6, 6, 7, false);
}

#[test]
fn explicit_minimum_above_doubled_capacity_is_preserved() {
    reserve_case("move-minimum", 4, 3, 20, false);
}

#[test]
fn copy_fallback_preserves_sources_until_all_copies_are_complete() {
    reserve_case("copy-double", 4, 3, 5, true);
    reserve_case("copy-minimum", 4, 3, 20, true);
}

#[test]
fn relocation_does_not_require_clone_or_default_bounds() {
    reserve_case("move-only", 4, 3, 5, false);
}

fn reject_eleven(item: &mut Relocated) -> Relocated {
    assert_ne!(
        item.value, 11,
        "relocation value 11: expected non-panicking move"
    );
    item.move_observed()
}

#[test]
fn callback_panic_preserves_registered_storage_and_drops_partial_replacement() {
    let trace = Rc::new(RefCell::new(String::new()));
    let mut vector = filled(4, 3, &trace);
    let before = vector.data_ptr();
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vector.reserve_with_relocation(5, reject_eleven);
    }));
    assert!(failed.is_err());
    assert_eq!((vector.len(), vector.capacity()), (3, 4));
    assert_eq!(vector.data_ptr(), before);
    assert_eq!(*trace.borrow(), "M10,D10,");
    assert!(vector.iter().map(|item| item.value).eq([-1, 11, 12]));
}

#[test]
fn local_inline_predicate_preserves_retained_heap_state() {
    let mut array = LocalArray::<i32, 4>::default();
    let mut states = vec![array.is_inline()];
    array.allocate(3);
    states.push(array.is_inline());
    array.reallocate(6, false);
    states.push(array.is_inline());
    array.reallocate(0, true);
    states.push(array.is_inline());
    array.reallocate(2, false);
    states.push(array.is_inline());
    array.reallocate(7, false);
    array.deallocate();
    states.push(array.is_inline());
    check_storage_states(&states);
}

fn check_storage_states(states: &[bool]) {
    let encoded: Vec<String> = states
        .iter()
        .map(|&state| u8::from(state).to_string())
        .collect();
    fixture_row(&format!("local:{}", encoded.join(":")));
}

#[test]
fn borrowed_elements_remain_valid_through_callback_relocation() {
    let owner = String::from("retained");
    let mut vector = LinearVector::with_capacity(1);
    vector.append(owner.as_str());
    vector.reserve_with_relocation(3, std::mem::take);
    assert_eq!(vector.as_slice(), [owner.as_str()]);
}
