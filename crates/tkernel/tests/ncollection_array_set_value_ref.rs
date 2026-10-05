//! Borrowed SetValue uses copy assignment after validating the destination.
use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
struct Counts {
    assignments: Cell<usize>,
    constructions: Cell<usize>,
}

struct Probe {
    identity: usize,
    value: i32,
    counts: Rc<Counts>,
}

impl Clone for Probe {
    fn clone(&self) -> Self {
        self.counts
            .constructions
            .set(self.counts.constructions.get() + 1);
        Self {
            identity: self.identity,
            value: self.value,
            counts: Rc::clone(&self.counts),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.counts
            .assignments
            .set(self.counts.assignments.get() + 1);
        self.value = source.value;
    }
}

fn probe(identity: usize, value: i32, counts: &Rc<Counts>) -> Probe {
    Probe {
        identity,
        value,
        counts: Rc::clone(counts),
    }
}

fn assert_probe(probe: &Probe, identity: usize, value: i32) {
    assert_eq!((probe.identity, probe.value), (identity, value));
}

fn assert_index_error(result: Result<(), ArrayError>, index: i64) {
    assert_eq!(
        result,
        Err(ArrayError::Index {
            index,
            lower: 0,
            upper: 0
        })
    );
}

fn assert_same_target(actual: &Probe, expected: *const Probe) {
    assert_eq!(actual as *const Probe, expected);
}

#[test]
fn array1_borrowed_set_assigns_in_place_and_keeps_source() {
    let counts = Rc::new(Counts::default());
    let mut buffer = [probe(1, 10, &counts), probe(2, 20, &counts)];
    let source = probe(3, 99, &counts);
    let mut array = Array1::from_slice(&mut buffer);
    array.update_lower_bound(-1);
    let target = array.change_value(0).unwrap() as *const Probe;
    array.set_value_ref(0, &source).unwrap();
    assert_same_target(array.value(0).unwrap(), target);
    assert_probe(array.value(0).unwrap(), 2, 99);
    assert_probe(&source, 3, 99);
    assert_counts(&counts, 1, 0);
}

#[test]
fn array1_borrowed_set_checks_index_before_assignment() {
    let counts = Rc::new(Counts::default());
    let mut buffer = [probe(1, 10, &counts)];
    let source = probe(2, 20, &counts);
    let mut array = Array1::from_slice(&mut buffer);
    assert_index_error(array.set_value_ref(2, &source), 2);
    assert_counts(&counts, 0, 0);
}

#[test]
fn array2_borrowed_set_assigns_in_place_and_keeps_source() {
    let counts = Rc::new(Counts::default());
    let mut buffer = [probe(1, 10, &counts), probe(2, 20, &counts)];
    let source = probe(3, 99, &counts);
    let mut array = Array2::from_buffer(&mut buffer, 1, 2).unwrap();
    let target = array.change_value(0, 1).unwrap() as *const Probe;
    array.set_value_ref(0, 1, &source).unwrap();
    assert_same_target(array.value(0, 1).unwrap(), target);
    assert_probe(array.value(0, 1).unwrap(), 2, 99);
    assert_probe(&source, 3, 99);
    assert_counts(&counts, 1, 0);
}

#[test]
fn array2_borrowed_set_checks_both_indices_before_assignment() {
    let counts = Rc::new(Counts::default());
    let mut buffer = [probe(1, 10, &counts)];
    let source = probe(2, 20, &counts);
    let mut array = Array2::from_buffer(&mut buffer, 1, 1).unwrap();
    assert_index_error(array.set_value_ref(1, 0, &source), 1);
    assert_index_error(array.set_value_ref(0, 1, &source), 1);
    assert_counts(&counts, 0, 0);
}

fn assert_counts(counts: &Counts, assignments: usize, constructions: usize) {
    assert_eq!(counts.assignments.get(), assignments);
    assert_eq!(counts.constructions.get(), constructions);
}
