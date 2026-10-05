//! Native Init assigns from the borrowed value once per slot, in storage order.
use rustycad_tkernel::ncollection::Array1;
use std::cell::RefCell;

struct AssignmentProbe<'a> {
    slot: i32,
    value: i32,
    assignments: &'a RefCell<Vec<i32>>,
}

impl Clone for AssignmentProbe<'_> {
    fn clone(&self) -> Self {
        panic!(
            "unexpected value copy for slot {}; expected assignment",
            self.slot
        );
    }

    fn clone_from(&mut self, source: &Self) {
        self.assignments.borrow_mut().push(self.slot);
        self.value = source.value;
    }
}

fn assignment_probe(slot: i32, assignments: &RefCell<Vec<i32>>) -> AssignmentProbe<'_> {
    AssignmentProbe {
        slot,
        value: slot,
        assignments,
    }
}

#[test]
fn borrowed_init_assigns_every_slot_without_copy_construction() {
    let assignments = RefCell::new(Vec::new());
    let mut buffer = [1, 2, 3].map(|slot| assignment_probe(slot, &assignments));
    let source = assignment_probe(9, &assignments);
    let mut array = Array1::from_slice(&mut buffer);
    array.update_lower_bound(-1);
    array.init_ref(&source);
    assert_eq!(*assignments.borrow(), [1, 2, 3]);
    let values: Vec<i32> = array.iter().map(|item| item.value).collect();
    assert_eq!(values, [9, 9, 9]);
    assert_eq!((array.lower(), array.upper(), source.value), (-1, 1, 9));
}

#[test]
fn empty_borrowed_init_does_not_copy_or_assign_source() {
    let assignments = RefCell::new(Vec::new());
    let source = assignment_probe(9, &assignments);
    let mut array = Array1::default();
    array.init_ref(&source);
    assert!(assignments.borrow().is_empty());
    assert_eq!((array.lower(), array.upper(), source.value), (1, 0, 9));
}

#[test]
fn owned_initializer_also_assigns_the_last_slot() {
    let assignments = RefCell::new(Vec::new());
    let mut buffer = [1, 2].map(|slot| assignment_probe(slot, &assignments));
    Array1::from_slice(&mut buffer).init(assignment_probe(9, &assignments));
    assert_eq!(*assignments.borrow(), [1, 2]);
    assert_eq!((buffer[0].value, buffer[1].value), (9, 9));
}
