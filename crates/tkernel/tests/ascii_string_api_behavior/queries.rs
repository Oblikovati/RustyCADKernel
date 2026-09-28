#[test]
fn value_and_sub_string_ranges() {
    let text = AsciiString::from_raw(b"ab\0cd");
    assert_eq!(text.value(3).unwrap(), 0);
    assert!(text.value(0).is_err());
    assert!(text.value(6).is_err());
    assert_eq!(bytes(&text.sub_string(1, 5).unwrap()), b"ab");
    assert_eq!(bytes(&text.sub_string(4, 5).unwrap()), b"cd");
    assert!(text.sub_string(3, 2).is_err());
    assert!(text.sub_string(1, 6).is_err());
}

#[test]
fn location_misses_matches_overlapping_a_failed_prefix() {
    let text = AsciiString::new("aaab");
    assert_eq!(text.location("aab", 1, 4).unwrap(), 0);
    assert_eq!(text.location("ab", 1, 4).unwrap(), 3);
    assert_eq!(AsciiString::new("xaab").location("aab", 1, 4).unwrap(), 2);
    assert_eq!(text.location("", 9, -9).unwrap(), 0);
    assert!(text.location("a", 0, 4).is_err());
}

#[test]
fn location_char_counts_occurrences_inside_the_range() {
    let text = AsciiString::new("banana");
    assert_eq!(text.location_char(2, b'a', 1, 6).unwrap(), 4);
    assert_eq!(text.location_char(3, b'a', 3, 6).unwrap(), 0);
    assert_eq!(text.location_char(0, b'a', 1, 6).unwrap(), 0);
    assert!(AsciiString::default().location_char(1, b'a', 1, 1).is_err());
}

#[test]
fn first_location_in_and_not_in_set() {
    let text = AsciiString::new("  x1");
    assert_eq!(text.first_location_not_in_set(" ", 1, 4).unwrap(), 3);
    assert_eq!(text.first_location_in_set("0123456789", 1, 4).unwrap(), 4);
    assert_eq!(text.first_location_in_set("z", 1, 4).unwrap(), 0);
    assert_eq!(text.first_location_in_set("", 7, 0).unwrap(), 0);
    assert!(text.first_location_in_set("x", 2, 5).is_err());
}

#[test]
fn token_scans_like_occt() {
    let text = AsciiString::new(",,a,,bc,");
    assert_eq!(bytes(&text.token(",", 1)), b"a");
    assert_eq!(bytes(&text.token(",", 2)), b"bc");
    assert!(text.token(",", 3).is_empty());
    assert!(text.token(",", 0).is_empty());
    assert_eq!(bytes(&text.token("", 1)), b",,a,,bc,");
    assert_eq!(bytes(&text.token(b",\0a", 1)), b"a");
    assert_eq!(bytes(&AsciiString::from_raw(b"a\0b c").token(" ", 1)), b"a");
}

#[test]
fn comparisons_use_unsigned_bytes_and_full_spans() {
    let text = AsciiString::from_raw(b"a\xff");
    assert!(text.is_greater("ab"));
    assert!(AsciiString::new("ab").is_less(&text));
    assert!(text.starts_with("a") && text.ends_with(b"\xff") && text.starts_with(""));
    assert!(!AsciiString::new("a").starts_with("ab"));
    assert!(AsciiString::strings_equal(&text, b"a\xff"));
    assert!(AsciiString::same_bytes("AbC", "aBc", false));
    assert!(!AsciiString::same_bytes("AbC", "aBc", true));
    assert!(!AsciiString::same_bytes(b"\xe9", b"\xc9", false));
}

#[test]
fn is_ascii_accepts_printable_ascii_only() {
    assert!(AsciiString::new(" ~").is_ascii());
    assert!(!AsciiString::new("\t").is_ascii());
    assert!(!AsciiString::from_raw(b"\x7f").is_ascii());
    assert!(!AsciiString::from_raw(b"\x80").is_ascii());
}

#[test]
fn useful_length_decodes_permissively() {
    assert_eq!(AsciiString::new("ab  ").useful_length(), 2);
    assert_eq!(AsciiString::from_raw("é \t".as_bytes()).useful_length(), 2);
    assert_eq!(AsciiString::from_raw(b"\xc0\x80tail").useful_length(), 0);
    assert_eq!(AsciiString::from_raw(b"\xc3\0x ").useful_length(), 3);
    assert_eq!(AsciiString::from_raw(b"x\0y").useful_length(), 1);
}

#[test]
fn numeric_errors_are_numeric_error_class() {
    let failure = AsciiString::new("abc").integer_value().unwrap_err();
    assert_eq!(class(failure), "Standard_NumericError");
    assert_eq!(class(AsciiString::new(" ").real_value().unwrap_err()), "Standard_NumericError");
    assert_eq!(AsciiString::new(" -9223372036854775809").integer_value().unwrap(), 0);
    assert!(!AsciiString::from_raw(b"12\0.5").is_integer_value());
    assert!(AsciiString::from_raw(b"1.5\0x").is_real_value(true));
}

#[test]
fn print_stops_at_nul_and_read_takes_one_word() {
    let mut output = Vec::new();
    AsciiString::from_raw(b"ab\0cd").print(&mut output).unwrap();
    assert_eq!(output, b"ab");
    let mut input = std::io::Cursor::new(b"  one two".to_vec());
    let mut text = AsciiString::default();
    assert!(text.read(&mut input).unwrap());
    assert_eq!((bytes(&text), input.position()), (&b"one"[..], 5));
    let mut long = std::io::Cursor::new(vec![b'q'; 9000]);
    text.read(&mut long).unwrap();
    assert_eq!((text.len(), long.position()), (8189, 8189));
    assert!(!text.read(&mut std::io::Cursor::new(b"   ".to_vec())).unwrap());
    assert!(text.is_empty());
}

#[test]
fn hash_trait_uses_hash_code() {
    use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
    let text = AsciiString::new("key");
    let mut hasher = BuildHasherDefault::<std::collections::hash_map::DefaultHasher>::default().build_hasher();
    hasher.write_usize(text.hash_code());
    let direct = hasher.finish();
    let state = BuildHasherDefault::<std::collections::hash_map::DefaultHasher>::default();
    assert_eq!(state.hash_one(&text), direct);
}
