use super::*;
#[test]
fn containment_drop_keeps_raw_allocations_for_test_only_repair_and_reclamation() {
    let mut array = DynamicArray::with_increment(2);
    array.append(String::from("first"));
    array.append(String::from("second"));
    let mut state = array
        .into_clear_state()
        .unwrap_or_else(|(_, e)| panic!("{e}"));
    let (pointer, length, capacity, allocator) = state.raw_blocks[0].raw_ownership_for_test();
    interrupt_first_slot(&mut state);
    drop(state);
    unsafe {
        pointer.write(String::from("repaired"));
        drop(AllocVec::from_raw_parts_in(
            pointer, length, capacity, allocator,
        ));
    }
}
fn interrupt_first_slot(state: &mut DynamicArrayClearState<String>) {
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        state.clear_with_destruction(false, |slot, _| {
            slot.drop_in_place();
            panic!("ended slot0");
        });
    }));
    assert!(caught.is_err());
    assert_eq!(state.dead_slots(), 0..1);
}
