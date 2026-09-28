//! Construction, conversion and query cases (tkernel_extended_string_api_string.cxx).
use super::{
    SubCase, c_units, describe, dispatch, flag, guarded_kind, next_real, next_span, next_string,
    next_units, units_hex,
};
use crate::common::{hex, next, next_bytes};
use rustycad_tkernel::{
    ncollection::HashKey,
    tcollection::{AsciiString, ExtendedString},
};
use std::str::SplitWhitespace;

const NEW: &[(&str, SubCase)] = &[
    ("empty", |_| {
        let empty = ExtendedString::empty_string();
        format!(
            "{} {}",
            describe(&ExtendedString::default()),
            describe(empty)
        )
    }),
    ("cstr", |input| {
        let bytes = next_bytes(input);
        describe(&ExtendedString::from_c_string(
            &bytes,
            next::<i32>(input) != 0,
        ))
    }),
    ("ascii", |input| {
        let ascii = AsciiString::from_raw(&next_bytes(input));
        describe(&ExtendedString::from_ascii_string(
            &ascii,
            next::<i32>(input) != 0,
        ))
    }),
    ("u16", |input| {
        let string = ExtendedString::from_utf16(&next_units(input));
        format!("{} {}", describe(&string), describe(&string))
    }),
    ("span", |input| {
        let span = next_span(input);
        let length = next::<i32>(input);
        guarded_kind(ExtendedString::from_span(span.as_deref(), length).map(|s| describe(&s)))
    }),
    ("view", |input| describe(&next_string(input))),
    ("wide", wide_case),
    ("char", |input| {
        describe(&ExtendedString::from(next::<u8>(input)))
    }),
    ("unit", |input| {
        describe(&ExtendedString::from_char(next(input)))
    }),
    ("fill", |input| {
        let length = next::<i32>(input);
        guarded_kind(ExtendedString::filled(length, next(input)).map(|s| describe(&s)))
    }),
    ("int", |input| {
        describe(&ExtendedString::from_integer(next(input)))
    }),
    ("real", |input| {
        describe(&ExtendedString::from_real(next_real(input)))
    }),
];

fn wide_case(input: &mut SplitWhitespace<'_>) -> String {
    let values: Vec<u32> = next_bytes(input)
        .as_chunks::<4>()
        .0
        .iter()
        .map(|word| u32::from_le_bytes(*word))
        .collect();
    describe(&ExtendedString::from_wide(&values))
}

/// "new <kind> <args>": one constructor.
pub fn new_case(input: &mut SplitWhitespace<'_>) -> String {
    dispatch(NEW, input)
}

/// Every UTF-8 view of an explicit code-unit sequence.
pub fn convert_case(input: &mut SplitWhitespace<'_>) -> String {
    let string = next_string(input);
    let mut buffer = vec![0x7F_u8; string.utf8_length() + 1];
    let written = string.write_utf8(&mut buffer).expect("sized buffer");
    let mut printed = Vec::new();
    string.print(&mut printed).expect("in-memory stream");
    let ascii = AsciiString::from_extended(&string, None);
    format!(
        "{} {written} {} {} {} {} {} {} {} {} {}",
        string.utf8_length(),
        hex(&buffer),
        flag(string.is_ascii()),
        string.hash_code(),
        string.occt_hash(),
        hex(&printed),
        hex(string.to_string().as_bytes()),
        hex(ascii.as_bytes()),
        units_hex(string.as_utf16_with_nul()),
        units_hex(string.as_utf16())
    )
}

/// The numeric predicates and conversions.
pub fn numeric_case(input: &mut SplitWhitespace<'_>) -> String {
    let string = next_string(input);
    format!(
        "{} {} {} {} {}",
        flag(string.is_integer_value()),
        guarded_kind(string.integer_value().map(|value| value.to_string())),
        flag(string.is_real_value(false)),
        flag(string.is_real_value(true)),
        guarded_kind(string.real_value().map(|value| value.to_bits().to_string()))
    )
}

/// Token with an explicit index and with the default index 1.
pub fn token_case(input: &mut SplitWhitespace<'_>) -> String {
    let string = next_string(input);
    let separators = next_units(input);
    let which = next::<i32>(input);
    format!(
        "{} {}",
        describe(&string.token(&separators, which)),
        describe(&string.token(&separators, 1))
    )
}

/// FirstLocationInSet and FirstLocationNotInSet.
pub fn locate_case(input: &mut SplitWhitespace<'_>) -> String {
    let string = next_string(input);
    let set = next_string(input);
    let (from, to) = (next::<i32>(input), next::<i32>(input));
    format!(
        "{} {}",
        guarded_kind(
            string
                .first_location_in_set(&set, from, to)
                .map(|i| i.to_string())
        ),
        guarded_kind(
            string
                .first_location_not_in_set(&set, from, to)
                .map(|i| i.to_string())
        )
    )
}

/// Value at a one-based index.
pub fn value_case(input: &mut SplitWhitespace<'_>) -> String {
    let string = next_string(input);
    guarded_kind(string.value(next(input)).map(|unit| unit.to_string()))
}

/// The (pointer, length) comparison and search cores.
pub fn span_case(input: &mut SplitWhitespace<'_>) -> String {
    let string = next_string(input);
    let span = next_span(input);
    let (units, length) = (span.as_deref(), next::<i32>(input));
    let flags = [
        string.is_equal_span(units, length),
        string.is_different_span(units, length),
        string.is_less_span(units, length),
        string.is_greater_span(units, length),
        string.starts_with_span(units, length),
        string.ends_with_span(units, length),
    ];
    format!(
        "{} {} {}",
        flags.map(flag).concat(),
        string.search_span(units, length),
        string.search_from_end_span(units, length)
    )
}

/// The whole-string, `const char16_t*` and operator forms of every comparison and search.
pub fn whole_case(input: &mut SplitWhitespace<'_>) -> String {
    let string = next_string(input);
    let units = next_units(input);
    let (other, pointer) = (ExtendedString::from_raw(&units), c_units(&units));
    let flags = [
        string.is_equal(&other),
        string == other,
        string.is_equal(pointer),
        string.is_equal(pointer),
        ExtendedString::is_equal_strings(&string, &other),
        string.is_different(&other),
        string != other,
        string.is_different(pointer),
        string.is_different(pointer),
        string.is_less(&other),
        string < other,
        string.is_less(pointer),
        string.is_less(pointer),
        string.is_greater(&other),
        string > other,
        string.is_greater(pointer),
        string.is_greater(pointer),
        string.starts_with(&other),
        string.starts_with(pointer),
        string.ends_with(&other),
        string.ends_with(pointer),
        string.is_same_string(&other, true),
        string.is_same_string(&other, false),
    ];
    let searches = [
        string.search(&other),
        string.search(pointer),
        string.search_from_end(&other),
        string.search_from_end(pointer),
    ];
    format!(
        "{} {}",
        flags.map(flag).concat(),
        searches.map(|i| i.to_string()).join(" ")
    )
}
