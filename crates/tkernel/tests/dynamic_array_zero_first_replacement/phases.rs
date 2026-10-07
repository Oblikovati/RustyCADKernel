use super::payload::{Events, Payload, retained_payloads, row};
use rustycad_tkernel::ncollection::DynamicArray;
use std::panic::{AssertUnwindSafe, catch_unwind};
const EMPLACE_CASES: [(&str, i32); 5] = [
    ("new_ctor_throw", -1),
    ("new", 10),
    ("replace", 20),
    ("existing_ctor_throw", -1),
    ("assignment_throw", 99),
];
fn emplace(array: &mut DynamicArray<Payload>, value: i32, events: &Events) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        array
            .emplace_value_with_assignment(0_usize, || Payload::new(value, events), Payload::assign)
            .unwrap();
    }))
    .is_err()
}
pub(crate) fn emplace_rows() -> Vec<String> {
    let mut array = retained_payloads();
    let events = Events::default();
    let mut rows = Vec::new();
    for (name, value) in EMPLACE_CASES {
        let caught = emplace(&mut array, value, &events);
        rows.push(row(name, &array, &events, caught));
    }
    rows.extend(recovery_rows(&mut array, &events));
    rows
}
fn recovery_rows(array: &mut DynamicArray<Payload>, events: &Events) -> Vec<String> {
    let mut rows = Vec::new();
    emplace(array, 21, events);
    array.erase_last();
    rows.push(row("erase_recovery", array, events, false));
    array.set_increment(2);
    emplace(array, 22, events);
    rows.push(row("reset", array, events, false));
    array.clear(true);
    rows.push(row("reset_clear", array, events, false));
    rows
}
fn borrowed(array: &mut DynamicArray<Payload>, donor: &Payload) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        array
            .set_value_ref_with_assignment(0_i32, donor, Payload::copied, Payload::assign)
            .unwrap();
    }))
    .is_err()
}
pub(crate) fn borrowed_rows() -> Vec<String> {
    let mut array = retained_payloads();
    let events = Events::default();
    let mut donor = Payload::new(30, &events);
    events.borrow_mut().clear();
    let mut rows = Vec::new();
    for (name, value) in [
        ("borrow_new", 30),
        ("borrow_replace", 31),
        ("borrow_throw", -2),
    ] {
        donor.value = value;
        let caught = borrowed(&mut array, &donor);
        rows.push(row(name, &array, &events, caught));
    }
    array.erase_last();
    rows
}
fn moved(array: &mut DynamicArray<Payload>, donor: &mut Payload) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        array
            .set_value_moved_with_assignment(0_usize, donor, Payload::moved, Payload::assign)
            .unwrap();
    }))
    .is_err()
}
pub(crate) fn moved_rows() -> Vec<String> {
    let mut array = retained_payloads();
    let events = Events::default();
    let mut donor = Payload::new(40, &events);
    events.borrow_mut().clear();
    let mut rows = Vec::new();
    for (name, value) in [("move_new", 40), ("move_replace", 41), ("move_throw", -3)] {
        donor.value = value;
        let caught = moved(&mut array, &mut donor);
        rows.push(row(name, &array, &events, caught));
        if let Some(donor_row) = donor_observation(value, donor.value) {
            rows.push(donor_row);
        }
    }
    array.erase_last();
    rows
}
fn donor_observation(value: i32, donor: i32) -> Option<String> {
    match value {
        40 => Some(format!("move_donor={donor}")),
        -3 => Some(format!("throw_donor={donor}")),
        _ => None,
    }
}
pub(crate) fn internal_rows() -> Vec<String> {
    let mut array = retained_payloads();
    let events = Events::default();
    emplace(&mut array, 50, &events);
    events.borrow_mut().clear();
    array
        .set_value_from_index_with_assignment(0_usize, 0_usize, Payload::copied, Payload::assign)
        .unwrap();
    let mut rows = vec![row("self_copy", &array, &events, false)];
    array
        .set_value_moved_from_index_with_assignment(0_i32, 0_i32, Payload::moved, Payload::assign)
        .unwrap();
    rows.push(row("self_move", &array, &events, false));
    rows.push(internal_throw_row(&mut array, &events));
    array.erase_last();
    rows
}
fn internal_throw_row(array: &mut DynamicArray<Payload>, events: &Events) -> String {
    array.change_first().unwrap().value = -3;
    let caught = catch_unwind(AssertUnwindSafe(|| {
        array
            .set_value_moved_from_index_with_assignment(
                0_usize,
                0_usize,
                Payload::moved,
                Payload::assign,
            )
            .unwrap();
    }))
    .is_err();
    row("self_move_throw", array, events, caught)
}
