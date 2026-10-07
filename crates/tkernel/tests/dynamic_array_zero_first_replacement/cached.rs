use super::{DynamicArray, DynamicArrayCachedIterator, retained_integers};
type WritableCursor = DynamicArrayCachedIterator<i32, true>;
pub(crate) fn cached_rows() -> Vec<String> {
    let mut array = retained_integers();
    array.emplace_value(0_usize, || 7).unwrap();
    array[0_usize] = 8;
    let mut rows = vec![const_capture_row(&array)];
    let original = std::ptr::from_ref(array.first().unwrap());
    let mut cursor = WritableCursor::from_mutable(&mut array, 0).unwrap();
    rows.push(capture_row(&cursor, original));
    cursor.advance_with(&mut array).unwrap();
    rows.push(format!("advance more={}", u8::from(cursor.more())));
    cursor.retreat_with(&mut array).unwrap();
    unsafe {
        *cursor.change_value() = 9;
    }
    rows.push(format!("retreat value={}", array.first().unwrap()));
    rows.extend(move_reload_rows(&mut array, &mut cursor));
    rows
}
fn capture_row(cursor: &WritableCursor, original: *const i32) -> String {
    let value = unsafe { cursor.value() };
    let same = u8::from(cursor.pointer() == original);
    format!(
        "capture value={value} same={same} more={}",
        u8::from(cursor.more())
    )
}
fn move_reload_rows(array: &mut DynamicArray<i32>, cursor: &mut WritableCursor) -> Vec<String> {
    let mut target = DynamicArray::from_moved(array);
    array.set_increment(2);
    array.append(101);
    array.append(102);
    let value = unsafe { cursor.value() };
    let mut rows = vec![format!(
        "move value={value} target={}",
        target.first().unwrap()
    )];
    rows.extend(reload_rows(array, cursor));
    target.erase_last();
    rows
}
fn reload_rows(array: &mut DynamicArray<i32>, cursor: &mut WritableCursor) -> Vec<String> {
    cursor.offset_with(array, 0).unwrap();
    let value = unsafe { cursor.value() };
    let same = u8::from(cursor.pointer() == std::ptr::from_ref(array.first().unwrap()));
    let mut rows = vec![format!("reload value={value} same={same}")];
    cursor.advance_with(array).unwrap();
    let more = u8::from(cursor.more());
    rows.push(format!("snapshot more={more} source_size={}", array.size()));
    rows
}
fn const_capture_row(array: &DynamicArray<i32>) -> String {
    let cursor = DynamicArrayCachedIterator::from_shared(array, 0).unwrap();
    let value = unsafe { cursor.value() };
    let same = u8::from(cursor.pointer() == std::ptr::from_ref(array.first().unwrap()));
    format!("const_capture value={value} same={same}")
}
