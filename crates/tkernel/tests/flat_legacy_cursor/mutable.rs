//! Exclusive mutable cursor cases use the same native receipt as shared inspection.
use super::{append_failure, append_state, append_triple, data_fixture};
use rustycad_tkernel::ncollection::{FlatDataCursorMut, FlatDataMap};

fn mutate_current_bindings(output: &mut String, cursor: &mut FlatDataCursorMut<'_, i32, i32>) {
    while cursor.more() {
        let key = *cursor.key().unwrap();
        *cursor.change_value().unwrap() += key;
        let alias = cursor.as_cursor();
        append_triple(
            output,
            &key,
            cursor.value().unwrap(),
            alias.value().unwrap(),
        );
        cursor.advance();
    }
}

pub(super) fn append_mutable_cursor(output: &mut String) {
    let mut map = data_fixture();
    let mut other = FlatDataMap::<i32, i32>::default();
    other.bind(3, 30);
    {
        let mut cursor = map.cursor_mut();
        append_state(output, cursor.more(), cursor.is_equal(&other.cursor_mut()));
        mutate_current_bindings(output, &mut cursor);
        cursor.advance();
        let equal = cursor.is_equal(&FlatDataCursorMut::default());
        append_state(output, cursor.more(), equal);
        append_failure(output, cursor.change_value().map(|value| &*value));
    }
    let first = map.find(&3).unwrap();
    let second = map.find(&11).unwrap();
    append_triple(output, first, second, map.find(&4).unwrap());
}

fn assert_cursor_failure(error: rustycad_tkernel::standard::Failure, method: &str) {
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    let expected = format!("NCollection_FlatDataMap::Iterator::{method}");
    assert_eq!(error.message(), expected);
}

#[test]
fn default_mutable_cursor_preserves_exhaustion_and_native_failure_messages() {
    let mut cursor = FlatDataCursorMut::<i32, i32>::default();
    assert!(!cursor.more());
    cursor.advance();
    assert!(cursor.is_equal(&FlatDataCursorMut::default()));
    for (result, method) in [(cursor.key(), "Key"), (cursor.value(), "Value")] {
        assert_cursor_failure(result.unwrap_err(), method);
    }
    assert_cursor_failure(cursor.change_value().unwrap_err(), "ChangeValue");
}
