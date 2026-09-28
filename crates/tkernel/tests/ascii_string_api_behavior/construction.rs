#[test]
fn c_string_constructors_stop_at_the_first_nul() {
    assert_eq!(bytes(&AsciiString::from_bytes(b"ab\0cd")), b"ab");
    assert_eq!(bytes(&AsciiString::new("ab\0cd")), b"ab");
    assert_eq!(bytes(&AsciiString::from_view(b"ab\0cd")), b"ab");
    assert_eq!(bytes(&AsciiString::from_raw(b"ab\0cd")), b"ab\0cd");
}

#[test]
fn bounded_constructor_clamps_negative_and_long_lengths() {
    assert_eq!(bytes(&AsciiString::with_max_length("hello", 3)), b"hel");
    assert_eq!(bytes(&AsciiString::with_max_length("hello", -1)), b"");
    assert_eq!(bytes(&AsciiString::with_max_length("hello", i32::MAX)), b"hello");
    assert_eq!(bytes(&AsciiString::with_max_length(b"a\0b", 3)), b"a");
}

#[test]
fn character_and_filled_constructors_follow_occt() {
    assert!(AsciiString::from_char(0).is_empty());
    assert_eq!(bytes(&AsciiString::from_char(0xFF)), [0xFF]);
    assert_eq!(bytes(&AsciiString::filled(3, 0)), [0, 0, 0]);
    assert_eq!(AsciiString::filled(0, b'x').length(), 0);
}

#[test]
fn numeric_constructors_use_percent_d_and_percent_g() {
    assert_eq!(bytes(&AsciiString::from_integer(i32::MIN)), b"-2147483648");
    assert_eq!(bytes(&AsciiString::from_real(f64::NAN)), b"nan");
    assert_eq!(bytes(&AsciiString::from_real(-f64::NAN)), b"-nan");
    assert_eq!(bytes(&AsciiString::from_real(5e-324)), b"4.94066e-324");
    assert_eq!(bytes(&AsciiString::from_real(123456.0)), b"123456");
    assert_eq!(bytes(&AsciiString::from_real(1234567.0)), b"1.23457e+06");
}

#[test]
fn joining_constructors_keep_or_drop_nul_like_their_overload() {
    let head = AsciiString::from_raw(b"a\0");
    assert_eq!(bytes(&AsciiString::joined_with_char(&head, 0)), b"a\0\0");
    assert_eq!(bytes(&AsciiString::joined_with_c_string(&head, b"b\0c")), b"a\0b");
    assert_eq!(bytes(&AsciiString::joined(&head, &AsciiString::from_raw(b"\0z"))), b"a\0\0z");
}

#[test]
fn wide_constructor_drops_surrogates_and_out_of_range_values() {
    let text = AsciiString::from_wide(&[0x41, 0xD800, 0x20AC, 0x110000, 0x8000_0000, 0x42, 0, 0x43]);
    assert_eq!(bytes(&text), "A€B".as_bytes());
    assert!(AsciiString::from_wide(&[0]).is_empty());
}

#[test]
fn extended_conversions_replace_or_encode() {
    let wide = ExtendedString::from_utf16(&[0x41, 0xE9, 0x20AC, 0]);
    assert_eq!(bytes(&AsciiString::from_extended(&wide, Some(b'?'))), b"A\xe9?");
    let mut text = AsciiString::new(">");
    text.assign_cat_extended(&wide, None);
    assert_eq!(bytes(&text), ">Aé€".as_bytes());
    let expected = [">Aé€A".as_bytes(), b"\xe9#"].concat();
    assert_eq!(bytes(&text.cat_extended(&wide, Some(b'#'))), expected);
}

#[test]
fn empty_string_move_swap_and_clear() {
    assert!(AsciiString::empty_string().is_empty());
    let mut first = AsciiString::new("one");
    let mut second = AsciiString::new("two!");
    first.swap(&mut second);
    assert_eq!((bytes(&first), bytes(&second)), (&b"two!"[..], &b"one"[..]));
    first.move_assign(&mut second);
    assert_eq!((bytes(&first), second.is_empty()), (&b"one"[..], true));
    first.clear();
    assert_eq!(first.as_bytes_with_nul(), [0]);
}

#[test]
fn next_prime_for_map_keeps_the_int_size_t_conversions() {
    assert_eq!(next_prime_for_map(0), 101);
    assert_eq!(next_prime_for_map(101), 1009);
    assert_eq!(next_prime_for_map(-1), 101);
    assert_eq!(next_prime_for_map(-5), -4);
    assert_eq!(next_prime_for_map(2_038_431_744), 2_038_431_745);
    assert_eq!(next_prime_for_map(i32::MAX), i32::MIN);
}
