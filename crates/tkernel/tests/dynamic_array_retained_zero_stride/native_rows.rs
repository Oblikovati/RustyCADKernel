use super::super::*;
fn retained_case(release: bool, erase: bool) -> String {
    let destroyed = Cell::new(0);
    let mut array = retained_payload_array(&destroyed);
    let original = array.storage_extension().block_at(0).unwrap();
    let slot = append_payload(&mut array, 11, &destroyed);
    array.change_value(0_usize).unwrap().value = 12;
    let mut rows = retained_row(&array, slot.cast_const() == original);
    if erase {
        array.erase_last();
    } else {
        array.clear(release);
    }
    rows += &after_clear_row(&mut array, &destroyed, release, erase);
    if !erase && !release {
        repair_retained_payload(&mut array);
        rows += &format!("repair destroyed={}\n", destroyed.get());
    }
    rows
}
fn after_clear_row(
    array: &mut DynamicArray<RetainedPayload<'_>>,
    destroyed: &Cell<usize>,
    release: bool,
    erase: bool,
) -> String {
    format!(
        "after release={} erase={} size={} destroyed={} blocks={}\n",
        u8::from(release),
        u8::from(erase),
        array.size(),
        destroyed.get(),
        block_count(array)
    )
}
fn retained_row(array: &DynamicArray<RetainedPayload<'_>>, same: bool) -> String {
    format!(
        "retained value={} first={} last={} size={} same={}\n",
        array.value(0_usize).unwrap().value,
        array.first().unwrap().value,
        array.last().unwrap().value,
        array.size(),
        u8::from(same)
    )
}
fn repair_retained_payload(array: &mut DynamicArray<RetainedPayload<'_>>) {
    let pointer = array.storage_extension().change_block_at(0).unwrap();
    // Native zero-stride Clear omitted this destructor. The live bytes remain uniquely unowned.
    unsafe {
        pointer.drop_in_place();
    }
}
fn omitted_live_row(
    array: &mut DynamicArray<RetainedPayload<'_>>,
    destroyed: &Cell<usize>,
) -> String {
    let pointer = array.storage_extension().block_at(0).unwrap();
    // Clear relinquished automatic ownership without touching these live object bytes.
    let value = unsafe { (*pointer).value };
    format!("omitted_live value={value} destroyed={}\n", destroyed.get())
}
fn reset_rows() -> String {
    let destroyed = Cell::new(0);
    let mut array = retained_payload_array(&destroyed);
    let original = array.storage_extension().block_at(0).unwrap();
    append_payload(&mut array, 21, &destroyed);
    array.clear(false);
    let mut rows = omitted_live_row(&mut array, &destroyed);
    repair_retained_payload(&mut array);
    rows += &reset_construct_row(&mut array, &destroyed, original);
    array.clear(true);
    rows += &format!(
        "reset_clear destroyed={} blocks={}\n",
        destroyed.get(),
        block_count(&mut array)
    );
    rows
}
fn reset_construct_row<'a>(
    array: &mut DynamicArray<RetainedPayload<'a>>,
    destroyed: &'a Cell<usize>,
    original: *const RetainedPayload<'a>,
) -> String {
    array.set_increment(2);
    // The test payload retains the counter's original lifetime across reset construction.
    let pointer = append_payload(array, 22, destroyed);
    let same = pointer.cast_const() == original;
    let blocks = block_count(array);
    format!(
        "reset value={} same={} blocks={} destroyed={}\n",
        array.first().unwrap().value,
        u8::from(same),
        blocks,
        destroyed.get()
    )
}
fn append_rows() -> String {
    let mut borrowed = retained_integers();
    let mut moved = retained_integers();
    let mut appended = retained_integers();
    let mut donor = 7;
    borrowed.append_ref(&donor);
    moved.append_moved_with_constructor(&mut donor, |source| *source);
    *appended.appended() = 9;
    let blocks = block_count(&mut appended);
    format!(
        "append borrowed={} moved={} default={} blocks={}\n",
        borrowed.first().unwrap(),
        moved.last().unwrap(),
        appended.value(0).unwrap(),
        blocks
    )
}
fn active_row() -> String {
    let mut array = DynamicArray::with_increment(2);
    array.append(1);
    array.set_increment(usize::MAX);
    array.append(2);
    let blocks = block_count(&mut array);
    format!(
        "active value={} size={} blocks={}\n",
        array.last().unwrap(),
        array.size(),
        blocks
    )
}
fn empty_integer_row(label: &str, array: &mut DynamicArray<i32>) -> String {
    let blocks = block_count(array);
    format!("{label} size={} blocks={}\n", array.size(), blocks)
}
fn freed_reset_row(array: &mut DynamicArray<i32>) -> String {
    array.set_increment(2);
    array.append(33);
    let blocks = block_count(array);
    format!(
        "freed_reset value={} size={} blocks={}\n",
        array.first().unwrap(),
        array.size(),
        blocks
    )
}
fn freed_reset_rows() -> String {
    let mut array = retained_integers();
    array.append(1);
    array.clear(true);
    let mut rows = empty_integer_row("freed", &mut array);
    rows += &freed_reset_row(&mut array);
    let mut empty = DynamicArray::<i32>::with_increment(usize::MAX);
    empty.clear(true);
    rows + &empty_integer_row("empty", &mut empty)
}
fn failed_constructor_row(
    array: &mut DynamicArray<RetainedPayload<'_>>,
    destroyed: &Cell<usize>,
) -> String {
    let caught = catch_unwind(AssertUnwindSafe(|| {
        array.emplace_append(|| panic!("negative probe construction"));
    }))
    .is_err();
    format!(
        "throw caught={} size={} blocks={} destroyed={}\n",
        u8::from(caught),
        array.size(),
        block_count(array),
        destroyed.get()
    )
}
fn throwing_rows() -> String {
    let destroyed = Cell::new(0);
    let mut array = retained_payload_array(&destroyed);
    let mut rows = failed_constructor_row(&mut array, &destroyed);
    append_payload(&mut array, 41, &destroyed);
    array.erase_last();
    rows += &format!(
        "retry size={} destroyed={}\n",
        array.size(),
        destroyed.get()
    );
    rows
}
fn resource_rows() -> String {
    let owner = std::rc::Rc::new(7);
    let witness = std::rc::Rc::downgrade(&owner);
    let mut array = DynamicArray::with_increment(2);
    for _ in 0..3 {
        array.append(owner.clone());
    }
    array.clear(false);
    array.set_increment(usize::MAX);
    array.append(owner);
    array.clear(false);
    let mut rows = resource_row("omitted", &witness);
    let slot = array.storage_extension().change_block_at(0).unwrap();
    // Clear abandoned this one strong owner in retained live bytes; repair its release exactly once.
    unsafe {
        slot.drop_in_place();
    }
    rows += &resource_row("repaired", &witness);
    rows
}
fn resource_row(label: &str, witness: &std::rc::Weak<i32>) -> String {
    format!(
        "{label}_resource strong={} expired={}\n",
        witness.strong_count(),
        u8::from(witness.upgrade().is_none())
    )
}
pub(crate) fn trace() -> String {
    retained_case(false, true)
        + &retained_case(false, false)
        + &retained_case(true, false)
        + &reset_rows()
        + &append_rows()
        + &active_row()
        + &freed_reset_rows()
        + &throwing_rows()
        + &resource_rows()
}
