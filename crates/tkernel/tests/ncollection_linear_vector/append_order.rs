use rustycad_tkernel::ncollection::LinearVector;
use std::panic::{AssertUnwindSafe, catch_unwind};

struct ThrowingDefault {
    value: i32,
}

impl Default for ThrowingDefault {
    fn default() -> Self {
        panic!("default construction failed after capacity growth");
    }
}

struct DefaultValue {
    value: i32,
}

impl Default for DefaultValue {
    fn default() -> Self {
        Self { value: 77 }
    }
}

fn seeded<T>(capacity: usize, value: T) -> LinearVector<T> {
    let mut vector = LinearVector::with_capacity(capacity);
    vector.append(value);
    vector
}

fn assert_default_panics(vector: &mut LinearVector<ThrowingDefault>) {
    let failure = catch_unwind(AssertUnwindSafe(|| {
        vector.appended();
    }));
    assert!(failure.is_err());
}

fn assert_emplace_panics(vector: &mut LinearVector<ThrowingDefault>) {
    let failure = catch_unwind(AssertUnwindSafe(|| {
        vector.emplace_append(|| panic!("emplace construction failed"));
    }));
    assert!(failure.is_err());
}

fn assert_values<T>(vector: &LinearVector<T>, expected: &[i32], value: impl Fn(&T) -> i32) {
    assert_eq!(vector.len(), expected.len());
    assert_eq!(vector.iter().map(value).collect::<Vec<_>>(), expected);
}

#[test]
fn appended_default_failure_grows_empty_and_full_vectors_first() {
    let mut empty = LinearVector::<ThrowingDefault>::default();
    assert_default_panics(&mut empty);
    assert_eq!(empty.capacity(), 2);
    assert_values(&empty, &[], |item| item.value);
    let mut full = seeded(1, ThrowingDefault { value: 42 });
    assert_default_panics(&mut full);
    assert_eq!(full.capacity(), 2);
    assert_values(&full, &[42], |item| item.value);
}

#[test]
fn appended_default_failure_keeps_capacity_when_a_slot_is_available() {
    let mut vector = seeded(2, ThrowingDefault { value: 42 });
    assert_default_panics(&mut vector);
    assert_eq!(vector.capacity(), 2);
    assert_values(&vector, &[42], |item| item.value);
}

#[test]
fn appended_success_grows_and_preserves_existing_values() {
    let mut empty = LinearVector::<DefaultValue>::default();
    assert_eq!(empty.appended().value, 77);
    assert_eq!(empty.capacity(), 2);
    let mut spare = seeded(2, DefaultValue { value: 42 });
    assert_eq!(spare.appended().value, 77);
    assert_eq!(spare.capacity(), 2);
    assert_values(&spare, &[42, 77], |item| item.value);
    let mut full = seeded(1, DefaultValue { value: 42 });
    assert_eq!(full.appended().value, 77);
    assert_eq!(full.capacity(), 2);
    assert_values(&full, &[42, 77], |item| item.value);
}

#[test]
fn emplace_failure_grows_empty_and_full_vectors_first() {
    let mut empty = LinearVector::<ThrowingDefault>::default();
    assert_emplace_panics(&mut empty);
    assert_eq!(empty.capacity(), 2);
    assert_values(&empty, &[], |item| item.value);
    let mut full = seeded(1, ThrowingDefault { value: 42 });
    assert_emplace_panics(&mut full);
    assert_eq!(full.capacity(), 2);
    assert_values(&full, &[42], |item| item.value);
}

#[test]
fn emplace_failure_keeps_capacity_when_a_slot_is_available() {
    let mut vector = seeded(2, ThrowingDefault { value: 42 });
    assert_emplace_panics(&mut vector);
    assert_eq!(vector.capacity(), 2);
    assert_values(&vector, &[42], |item| item.value);
}

#[test]
fn emplace_success_observes_growth_and_no_growth_states() {
    let mut empty = LinearVector::<DefaultValue>::default();
    empty.emplace_append(|| DefaultValue { value: 77 });
    assert_eq!(empty.capacity(), 2);
    assert_values(&empty, &[77], |item| item.value);
    let mut full = seeded(1, DefaultValue { value: 42 });
    full.emplace_append(|| DefaultValue { value: 77 });
    assert_eq!(full.capacity(), 2);
    assert_values(&full, &[42, 77], |item| item.value);
    let mut spare = seeded(2, DefaultValue { value: 42 });
    spare.emplace_append(|| DefaultValue { value: 77 });
    assert_eq!(spare.capacity(), 2);
    assert_values(&spare, &[42, 77], |item| item.value);
}
