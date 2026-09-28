fn shared(text: &[u8]) -> Handle<HAsciiString> {
    Handle::new(HAsciiString::from(AsciiString::from_raw(text)))
}

fn h_bytes(text: &HAsciiString) -> Vec<u8> {
    text.string().as_bytes().to_vec()
}

#[test]
fn h_ascii_string_is_a_transient_edited_through_every_handle() {
    let first = Handle::new(HAsciiString::new("ab"));
    let second = first.clone();
    second.assign_cat("cd");
    assert_eq!(h_bytes(&first), b"abcd");
    assert!(first.is_kind_name("Standard_Transient"));
    assert!(first.is_instance_name("TCollection_HAsciiString"));
    assert_eq!(first.as_bytes_with_nul(), b"abcd\0");
}

#[test]
fn h_constructors_mirror_ascii_string() {
    assert_eq!(h_bytes(&HAsciiString::new(b"a\0b")), b"a");
    assert_eq!(h_bytes(&HAsciiString::from_char(0)), b"");
    assert_eq!(h_bytes(&HAsciiString::filled(2, b'z')), b"zz");
    assert_eq!(h_bytes(&HAsciiString::from_integer(-7)), b"-7");
    assert_eq!(h_bytes(&HAsciiString::from_real(0.25)), b"0.25");
    assert_eq!(h_bytes(&HAsciiString::from_string(&AsciiString::from_raw(b"\0x"))), b"\0x");
    assert_eq!(h_bytes(&HAsciiString::copied(&shared(b"q"))), b"q");
    let wide = ExtendedString::from_utf16(&[0x20AC, 0x41, 0]);
    let wide = HExtendedString::from(wide);
    assert_eq!(h_bytes(&HAsciiString::from_h_extended_string(&wide, b'?')), b"?A");
    assert_eq!(h_bytes(&HAsciiString::from_h_extended_string(&wide, 0)), "€A".as_bytes());
}

#[test]
fn h_self_aliasing_edits_do_not_deadlock() {
    let text = shared(b"ab");
    text.assign_cat_string(&text);
    text.insert_string(2, &text).unwrap();
    text.prepend(&text);
    assert_eq!(h_bytes(&text), b"aababbabaababbab");
    assert!(text.is_same_state(&text));
}

#[test]
fn h_null_handles_raise_null_object_where_occt_checks() {
    let text = shared(b"ab");
    let null = Handle::<HAsciiString>::default();
    assert_eq!(class(text.is_different(&null).unwrap_err()), "Standard_NullObject");
    assert_eq!(class(text.is_same_string(&null).unwrap_err()), "Standard_NullObject");
    assert!(text.is_same_string_with_case(&null, true).is_err());
}

#[test]
fn h_strncmp_members_ignore_bytes_after_a_shared_nul() {
    let text = shared(b"a\0b");
    let other = shared(b"a\0c");
    assert!(!text.is_different(&other).unwrap());
    assert!(text.is_same_string(&other).unwrap());
    assert!(text.is_same_state(&other));
    assert!(!text.is_same_string_with_case(&other, true).unwrap());
    assert!(text.is_different(&shared(b"a\0bc")).unwrap());
}

#[test]
fn h_members_returning_new_objects() {
    let text = shared(b"one two");
    assert_eq!(h_bytes(&text.cat("!")), b"one two!");
    assert_eq!(h_bytes(&text.cat_string(&shared(b"?"))), b"one two?");
    assert_eq!(h_bytes(&text.token(" ", 2)), b"two");
    assert_eq!(h_bytes(&text.sub_string(1, 3).unwrap()), b"one");
    assert_eq!(h_bytes(&text.split(3).unwrap()), b" two");
    assert_eq!(h_bytes(&text), b"one");
    assert!(text.sub_string(0, 1).is_err());
}

#[test]
fn h_c_string_arguments_stop_at_nul() {
    let text = shared(b"ab");
    text.assign_cat(b"c\0d");
    text.insert(1, b">\0x").unwrap();
    text.set_value(4, b"C\0").unwrap();
    assert_eq!(h_bytes(&text), b">abC");
    assert_eq!(text.search(b"b\0zz"), 3);
    assert_eq!(text.search_from_end("C"), 4);
}

#[test]
fn h_edits_and_queries_delegate_with_occt_checks() {
    let text = shared(b" Ab1 ");
    text.left_adjust();
    text.right_adjust();
    text.upper_case();
    assert_eq!(h_bytes(&text), b"AB1");
    assert_eq!(class(text.center(-2, b'*').unwrap_err()), "Standard_NegativeValue");
    text.center(5, b'*').unwrap();
    assert_eq!(h_bytes(&text), b"*AB1*");
    assert_eq!(text.first_location_in_set(&shared(b"1"), 1, 5).unwrap(), 4);
    assert_eq!(text.location(&shared(b"B1"), 1, 5).unwrap(), 3);
    assert_eq!(text.location_char(2, b'*', 1, 5).unwrap(), 5);
    assert!(text.first_location_not_in_set(&shared(b"*"), 0, 5).is_err());
    assert_eq!(text.value(2).unwrap(), b'A');
    text.remove(1, 1).unwrap();
    text.truncate(3).unwrap();
    text.lower_case();
    text.capitalize();
    assert_eq!(h_bytes(&text), b"Ab1");
    assert!(text.is_less(&shared(b"b")) && text.is_greater(&shared(b"A")));
}

#[test]
fn h_numeric_and_output_members() {
    let text = shared(b"42.5x");
    assert_eq!(text.integer_value().unwrap(), 42);
    assert_eq!(text.real_value().unwrap(), 42.5);
    assert!(!text.is_integer_value() && text.is_real_value() && text.is_ascii());
    assert_eq!((text.length(), text.useful_length(), text.is_empty()), (5, 5, false));
    let mut output = Vec::new();
    text.print(&mut output).unwrap();
    assert_eq!(output, b"42.5x");
    text.clear();
    assert!(text.is_empty());
}
