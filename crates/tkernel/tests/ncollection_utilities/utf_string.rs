use rustycad_tkernel::ncollection::{String8, UtfString};

fn utf8(text: &str) -> String8 {
    String8::from_unicode_units(text.as_bytes(), -1)
}

#[test]
fn conversions_track_size_and_length() {
    let text = utf8("a\u{e9}\u{1f600}");
    assert_eq!((text.size(), text.length(), text.is_empty()), (7, 3, false));
    let wide = text.to_utf16();
    assert_eq!(wide.to_c_string(), [0x61, 0xE9, 0xD83D, 0xDE00, 0]);
    assert_eq!((wide.size(), wide.length()), (8, 3));
    assert_eq!(text.to_utf32().to_c_string(), [0x61, 0xE9, 0x1F600, 0]);
    assert_eq!(text.to_utf_wide().length(), 3);
    assert_eq!(wide.to_utf8(), text);
}

#[test]
fn length_limit_counts_symbols_not_units() {
    let text = UtfString::<u32>::from_unicode_units("\u{e9}\u{e9}\u{e9}".as_bytes(), 2);
    assert_eq!((text.size(), text.length()), (8, 2));
}

#[test]
fn dropped_illegal_symbols_still_count_in_length() {
    let text = UtfString::<u8>::from_unicode_units(&[0xD800u16], -1);
    assert_eq!((text.size(), text.length(), text.is_empty()), (0, 1, true));
    // operator+ sums lengths; operator+= skips an empty right side entirely.
    assert_eq!((&text + &text).length(), 2);
    let mut appended = text.clone();
    appended += &text;
    assert_eq!(appended.length(), 1);
}

#[test]
fn same_unit_copies_keep_invalid_sequences() {
    // 0xFF announces five trailing bytes: the symbol swallows 0x41 and four padding zeros.
    let text = String8::from_unicode_units(&[0xFFu8, 0x41], -1);
    assert_eq!(text.to_c_string(), [0xFF, 0x41, 0, 0, 0, 0, 0]);
    assert_eq!((text.size(), text.length()), (6, 1));
    let converted = text.to_utf16();
    assert_eq!((converted.size(), converted.length()), (0, 1));
}

#[test]
fn indexed_access_scans_symbols() {
    let text = utf8("a\u{e9}b");
    assert_eq!(
        (text.get_char(1), text.get_char(3), text.get_char(-1)),
        (0xE9, 0, 0)
    );
    assert_eq!(text.get_char_buffer(2), Some(&b"b\0"[..]));
    assert_eq!(text.get_char_buffer(3), None);
    assert_eq!(text.iterator().current(), 0x61);
}

#[test]
fn sub_string_uses_symbol_indices() {
    let text = utf8("a\u{e9}bc");
    assert_eq!(text.sub_string(1, 3), utf8("\u{e9}b"));
    assert!(text.sub_string(2, 2).is_empty());
    assert!(text.sub_string(9, 10).is_empty());
    // The symbol budget is end - start, so a negative start widens the copy.
    assert_eq!(text.sub_string(-5, 1), text);
}

#[test]
fn assignment_swap_clear_and_equality() {
    let mut first = utf8("one");
    let mut second = utf8("two");
    first.swap(&mut second);
    assert_eq!((first.clone(), second.clone()), (utf8("two"), utf8("one")));
    first.assign(&second);
    assert!(first.is_equal(&second) && first == second);
    first.clear();
    assert!(first.is_empty() && first != second && first == String8::new());
    first.from_unicode(&[0x1F600u32], -1);
    assert_eq!(first, utf8("\u{1f600}"));
}

#[test]
fn locale_conversions_follow_the_c_locale() {
    let mut text = utf8("keep");
    text.from_locale(b"abc", -1);
    assert_eq!(text, utf8("a")); // mblen measures the first character only
    text.from_locale(&[0xC3, 0xA9], -1);
    assert!(text.is_empty() && text.length() == 0);
    text.from_locale(b"", -1);
    assert!(text.is_empty());
}

#[test]
fn to_locale_mirrors_wcstombs_results() {
    let mut buffer = [0x55u8; 4];
    assert!(utf8("abcdef").to_locale(&mut buffer));
    assert_eq!(buffer, *b"abcd"); // a full buffer gets no terminator
    let mut buffer = [0x55u8; 4];
    assert!(utf8("a\u{e9}").to_locale(&mut buffer)); // error still reports true
    assert_eq!(buffer, [b'a', 0x55, 0x55, 0x55]);
    let mut buffer = [0x55u8; 2];
    assert!(!String8::new().to_locale(&mut buffer));
    assert_eq!(buffer, [0, 0x55]);
    assert!(!utf8("a").to_locale(&mut []));
}
