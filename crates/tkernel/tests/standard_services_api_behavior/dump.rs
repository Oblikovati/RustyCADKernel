//! Standard_Dump members beyond the legacy suite.
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::standard::dump::{
    self, DumpOutput, DumpStream, DumpValue, DumpValues, JsonKey,
};
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

fn stream_of(text: &[u8]) -> DumpStream {
    let mut stream = DumpStream::default();
    stream.write_fragment(text).unwrap();
    stream
}

#[test]
fn keys_have_their_native_text_and_length() {
    assert_eq!(dump::json_key_to_string(JsonKey::None), "");
    assert_eq!(
        dump::json_key_to_string(JsonKey::SeparatorValueToValue),
        ", "
    );
    assert_eq!(dump::json_key_length(JsonKey::SeparatorKeyToValue), 2);
    assert_eq!(JsonKey::Quote as i32, 5);
    assert_eq!(dump::get_pointer_prefix(), AsciiString::new("0x"));
}

#[test]
fn value_lists_and_keyed_classes_are_written_as_c_strings() {
    let mut stream = DumpStream::default();
    dump::dump_character_values(&mut stream, &["a", "b\0c", ""]).unwrap();
    assert_eq!(stream.bytes(), b"\"a\", \"b\", \"\"");
    let mut reals = DumpStream::default();
    dump::dump_real_values(&mut reals, &[1.0, 0.5, 1e-5, f64::INFINITY]).unwrap();
    assert_eq!(reals.bytes(), b"1, 0.5, 1e-05, inf");
    let mut keyed = stream_of(b"\"x\": 1");
    dump::dump_key_to_class(&mut keyed, "Loc", "\"y\": 2\0tail").unwrap();
    assert_eq!(keyed.bytes(), b"\"x\": 1, \"Loc\": {\"y\": 2}");
}

#[test]
fn pointer_text_keeps_the_short_form_quirk() {
    assert_eq!(
        dump::get_pointer_info(None, true).unwrap(),
        AsciiString::default()
    );
    assert_eq!(dump::get_pointer_info(Some(0), false).unwrap().len(), 0);
    if !cfg!(target_env = "msvc") {
        let long = dump::get_pointer_info(Some(0x1234), false).unwrap();
        assert_eq!(long.as_bytes(), b"0x1234");
        let short = dump::get_pointer_info(Some(0x1234), true).unwrap();
        assert_eq!(
            short.as_bytes(),
            b"0xx1234",
            "the scan stops at the 'x' of \"0x\""
        );
    }
    let null: Handle<StandardTransient> = Handle::default();
    assert!(
        dump::get_handle_pointer_info(&null, true)
            .unwrap()
            .is_empty()
    );
    let live = Handle::new(StandardTransient::default());
    let text = dump::get_handle_pointer_info(&live, false).unwrap();
    assert_eq!(
        text.as_bytes(),
        format!("{:#x}", live.as_ptr() as usize).as_bytes()
    );
}

#[test]
fn field_names_lose_prefixes_and_call_suffixes() {
    let name = |field: &str| dump::dump_field_to_name(field).map(|n| n.as_bytes().to_vec());
    assert_eq!(name("&myValue").unwrap(), b"Value");
    assert_eq!(name("MyX").unwrap(), b"X");
    assert_eq!(name("anObject.get()").unwrap(), b"Object");
    assert_eq!(
        name("angle").unwrap(),
        b"gle",
        "native quirk: \"an\" is always a prefix"
    );
    assert_eq!(name("a").unwrap(), b"a");
    assert_eq!(name("Size()").unwrap(), b"Size");
    assert!(name("()").unwrap_err().is_kind("Standard_OutOfRange"));
}

#[test]
fn format_json_indents_and_adds_missing_braces() {
    let formatted = dump::format_json(&stream_of(b"\"a\": 1, \"b\": [1, 2], \"c\": {\"d\": 2}"), 2);
    // Native quirk: text ending with a nested '}' never gets the added outer brace closed.
    let expected = "{\n  \"a\": 1,\n  \"b\": [1, 2],\n  \"c\": {\n    \"d\": 2\n  }";
    assert_eq!(formatted.as_bytes(), expected.as_bytes());
    assert!(dump::format_json(&stream_of(b""), 3).is_empty());
    assert_eq!(dump::text(&stream_of(b"ab\0cd")).as_bytes(), b"ab");
    assert_eq!(
        dump::format_json(&stream_of(b"x,"), -1).as_bytes(),
        b"{\nx,\n\n}"
    );
}

#[test]
fn split_json_renames_repeated_keys_and_records_positions() {
    let mut values = DumpValues::default();
    let text = "\"a\": 1, \"b\": \"x\", \"a\": [1, 2], \"c\": \"\"";
    assert!(dump::split_json(text, &mut values).unwrap());
    let entries: Vec<(String, String, i32)> = (1..=values.extent())
        .map(|i| {
            let key = String::from_utf8_lossy(values.find_key(i).unwrap().as_bytes()).into_owned();
            let value = values.find_from_index(i).unwrap();
            (
                key,
                String::from_utf8_lossy(value.value.as_bytes()).into_owned(),
                value.start_position,
            )
        })
        .collect();
    let expected = [
        ("a", "1", 6),
        ("b", "x", 15),
        ("a_1", "1, 2", 25),
        ("c", "", 38),
    ];
    let expected: Vec<(String, String, i32)> = expected
        .iter()
        .map(|(k, v, p)| (k.to_string(), v.to_string(), *p))
        .collect();
    assert_eq!(entries, expected);
}

#[test]
fn split_json_refuses_nested_braces_and_bad_text() {
    let mut values = DumpValues::default();
    let nested = dump::split_json("{\"a\": 1}", &mut values).unwrap_err();
    assert_eq!(
        nested.message(),
        "TCollection_AsciiString::SubString() out of range"
    );
    assert!(!dump::split_json("xy", &mut values).unwrap());
    // The scan stops before the last byte, so a one-byte text is accepted untouched.
    assert!(dump::split_json("x", &mut values).unwrap());
    assert!(dump::split_json("", &mut values).unwrap());
    assert_eq!(DumpValue::default().start_position, 0);
}

#[test]
fn hierarchical_indices_select_values_with_pairs() {
    let mut values: IndexedDataMap<AsciiString, AsciiString> = IndexedDataMap::default();
    values.add(AsciiString::new("a"), AsciiString::new("1"));
    values.add(AsciiString::new("b"), AsciiString::new("{\"c\": 2}"));
    values.add(AsciiString::new("d"), AsciiString::new("x:y"));
    assert_eq!(dump::hierarchical_value_indices(&values).unwrap(), [2]);
    assert!(dump::has_child_key("\"k\": v"));
    assert!(!dump::has_child_key("k:v"));
}

#[test]
fn field_names_are_matched_through_their_dump_name() {
    let mut position = 1;
    assert!(dump::process_field_name("\"Gap\": 1", "myGap", &mut position).unwrap());
    assert_eq!(position, 8);
    let mut past = 20;
    assert!(dump::process_field_name("\"Gap\": 1", "myGap", &mut past).is_err());
    assert!(!dump::process_stream_name("\"Gap\": 1", "Gap", &mut past).unwrap());
    let mut empty = 1;
    assert!(!dump::process_field_name("", "myGap", &mut empty).unwrap());
}

#[test]
fn init_value_stops_at_the_nearest_delimiter() {
    let mut position = 1;
    let value = dump::init_value("1.5}, x", &mut position).unwrap();
    assert_eq!((value.as_bytes(), position), (&b"1.5"[..], 5));
    let mut whole = 1;
    assert_eq!(
        dump::init_value("abc", &mut whole).unwrap().as_bytes(),
        b"abc"
    );
    assert_eq!(whole, 3);
}

#[test]
fn init_real_values_with_no_outputs_still_moves_past_the_bracket() {
    let mut position = 1;
    assert!(dump::init_real_values("[1] tail]", &mut position, &mut []).unwrap());
    assert_eq!(position, 4);
    let mut zero = 0;
    let failure = dump::init_real_values("[1]", &mut zero, &mut []).unwrap_err();
    assert_eq!(
        failure.message(),
        "",
        "Location's range check has no message"
    );
}
