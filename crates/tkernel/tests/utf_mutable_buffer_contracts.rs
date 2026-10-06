//! Writable cursor parity uses actual storage edits and native cached positions.
use rustycad_tkernel::ncollection::{UtfIteratorMut, UtfUnit};

fn mutable_row<T: UtfUnit>(label: &str, cursor: &UtfIteratorMut<'_, T>) -> String {
    let view = cursor.view();
    format!(
        "{label} {} {} {} {}",
        view.current(),
        view.index(),
        view.buffer_here(),
        view.buffer_next()
    )
}

fn byte_edit_rows() -> Vec<String> {
    let mut bytes = [0xc3_u8, 0xa9, b'b', 0, 0, 0, 0, 0];
    let mut cursor = UtfIteratorMut::new(&mut bytes);
    let mut rows = vec![mutable_row("initial", &cursor)];
    cursor.change_buffer_here().unwrap()[..2].copy_from_slice(b"xy");
    rows.push(mutable_row("cached", &cursor));
    cursor.advance();
    rows.push(mutable_row("cached_next", &cursor));
    cursor.change_buffer_here().unwrap()[1] = b'z';
    cursor.advance();
    rows.push(mutable_row("edited_next", &cursor));
    cursor.rewind();
    rows.push(mutable_row("rewound", &cursor));
    rows
}

fn half_edit_rows() -> Vec<String> {
    let mut halves = [0xd83d_u16, 0xde00, u16::from(b'q'), 0, 0];
    let mut cursor = UtfIteratorMut::new(&mut halves);
    let mut rows = vec![mutable_row("wide_initial", &cursor)];
    cursor.change_buffer_here().unwrap()[1] = u16::from(b'r');
    rows.push(mutable_row("wide_cached", &cursor));
    cursor.rewind();
    rows.push(mutable_row("wide_rewound", &cursor));
    cursor.advance();
    rows.push(mutable_row("wide_next", &cursor));
    rows
}

#[test]
fn writable_cursor_matches_all_native_observation_rows() {
    let mut rows = byte_edit_rows();
    rows.extend(half_edit_rows());
    let mut null = UtfIteratorMut::<u8>::from_optional(None);
    let absent = null.change_buffer_here().is_none();
    rows.push(format!(
        "null {} {} {}",
        u8::from(absent),
        null.view().current(),
        null.view().index()
    ));
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../../tools/oracle/tkernel_utf_mutable_buffer.expected")
    );
}

#[test]
fn mutable_access_keeps_original_allocation_and_remaining_extent() {
    let mut bytes = *b"abc\0";
    let base = bytes.as_ptr();
    let mut cursor = UtfIteratorMut::new(&mut bytes);
    assert_eq!(cursor.view().buffer_here_ptr(), base);
    cursor.advance();
    let tail = cursor.change_buffer_here().unwrap();
    assert_eq!(tail.len(), 3);
    assert_eq!(tail.as_ptr(), base.wrapping_add(1));
    tail[0] = b'z';
    assert_eq!(cursor.view().current(), u32::from(b'b'));
    cursor.rewind();
    cursor.advance();
    assert_eq!(cursor.view().current(), u32::from(b'z'));
}

#[test]
fn rebinding_resets_cache_and_releases_old_exclusive_storage() {
    let mut first = *b"ab\0";
    let mut second = [0x1f600_u32, 0];
    let mut byte_cursor = UtfIteratorMut::new(&mut first);
    byte_cursor.advance();
    byte_cursor.init_optional(None);
    assert!(byte_cursor.view().buffer_here_ptr().is_null());
    assert_eq!(byte_cursor.view().index(), 0);
    let mut wide_cursor = UtfIteratorMut::from_optional(None);
    wide_cursor.init_optional(Some(&mut second));
    assert_eq!(wide_cursor.view().current(), 0x1f600);
    let mut out = [0_u8; 4];
    assert_eq!(wide_cursor.view().get_utf8(&mut out), Ok(4));
    assert_eq!(out, [0xf0, 0x9f, 0x98, 0x80]);
}

#[test]
fn synthetic_padding_never_produces_writable_out_of_bounds_storage() {
    let mut bytes = *b"a";
    let mut cursor = UtfIteratorMut::new(&mut bytes);
    cursor.advance();
    assert!(cursor.change_buffer_here().unwrap().is_empty());
    cursor.advance();
    assert!(cursor.change_buffer_here().is_none());
    assert_eq!(cursor.view().current(), 0);
    cursor.rewind();
    assert_eq!(cursor.change_buffer_here().unwrap(), b"a");
    let mut empty: [u16; 0] = [];
    assert!(
        UtfIteratorMut::new(&mut empty)
            .change_buffer_here()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn signed_storage_edits_use_existing_native_width_dispatch() {
    let mut halves = [-10179_i16, -8704, 0];
    let mut cursor = UtfIteratorMut::new(&mut halves);
    assert_eq!(cursor.view().current(), (-10179_i32) as u32);
    cursor.change_buffer_here().unwrap()[0] = 65;
    cursor.rewind();
    assert_eq!(cursor.view().current(), 65);
    assert_eq!(cursor.view().buffer_next(), 1);
    cursor.advance();
    assert_eq!(cursor.view().current(), (-8704_i32) as u32);
}

#[test]
#[should_panic(expected = "UtfIterator buffer is null; expected a non-null code-unit buffer")]
fn null_mutable_cursor_has_the_shared_checked_increment_boundary() {
    UtfIteratorMut::<u8>::from_optional(None).advance();
}
