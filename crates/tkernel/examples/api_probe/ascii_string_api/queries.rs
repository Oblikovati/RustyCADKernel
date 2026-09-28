//! AsciiString constructors, read-only members and stream I/O; twin of
//! tkernel_ascii_string_api_queries.cxx.
use super::{
    Input, Outcome, c_prefix, flag, make, member, next_char, next_extended, next_int, next_real,
    next_text, next_wide, real_bits, respond, show,
};
use crate::common::hex;
use rustycad_tkernel::tcollection::AsciiString;

type Make = fn(&mut Input<'_>) -> AsciiString;
type Query = fn(&AsciiString, &mut Input<'_>) -> Outcome;

/// "ascii_make <member> <fields...>": the constructed string.
pub(super) fn make_case(input: &mut Input<'_>) -> String {
    let construct = member(MAKERS, input);
    respond(Ok(show(&construct(input))))
}

/// "ascii_query <member> <hex string> <fields...>": the member's result.
pub(super) fn query_case(input: &mut Input<'_>) -> String {
    let query = member(QUERIES, input);
    let text = make(&next_text(input));
    respond(query(&text, input))
}

/// "ascii_stream read|extract <hex input>": the string, the failbit and the unread rest.
pub(super) fn stream_case(input: &mut Input<'_>) -> String {
    let _read_or_extract = input.next().expect("stream member");
    let mut stream = std::io::Cursor::new(next_text(input));
    let mut text = AsciiString::new("junk");
    let extracted = text.read(&mut stream).expect("in-memory stream");
    let rest = &stream.get_ref()[stream.position() as usize..];
    format!("{} {} {}", show(&text), flag(!extracted), hex(rest))
}

/// `base.Cat(...)`-style makers read the base string first.
fn base(input: &mut Input<'_>) -> AsciiString {
    make(&next_text(input))
}

const MAKERS: &[(&str, Make)] = &[
    ("default", |_| AsciiString::default()),
    ("cstring", |i| AsciiString::from_bytes(next_text(i))),
    ("ptr_len", |i| {
        let text = next_text(i);
        AsciiString::with_max_length(text, next_int(i))
    }),
    ("view", |i| AsciiString::from_view(&next_text(i))),
    ("char", |i| AsciiString::from_char(next_char(i))),
    ("filled", |i| {
        let length = next_int(i) as usize;
        AsciiString::filled(length, next_char(i))
    }),
    ("int", |i| AsciiString::from_integer(next_int(i))),
    ("real", |i| AsciiString::from_real(next_real(i))),
    ("copy", |i| base(i).clone()),
    ("join_char", |i| {
        AsciiString::joined_with_char(&base(i), next_char(i))
    }),
    ("join_cstring", |i| {
        AsciiString::joined_with_c_string(&base(i), next_text(i))
    }),
    ("join", |i| AsciiString::joined(&base(i), &base(i))),
    ("wide", |i| AsciiString::from_wide(&next_wide(i))),
    ("extended", |i| {
        let wide = next_extended(i);
        AsciiString::from_extended(&wide, Some(next_char(i)))
    }),
    ("empty_string", |_| AsciiString::empty_string().clone()),
    ("cat_char", |i| base(i).cat(next_char(i))),
    ("plus_char", |i| &base(i) + next_char(i)),
    ("cat_int", |i| base(i).cat(next_int(i))),
    ("plus_int", |i| &base(i) + next_int(i)),
    ("cat_real", |i| base(i).cat(next_real(i))),
    ("plus_real", |i| &base(i) + next_real(i)),
    ("cat_span", |i| base(i).cat(next_text(i).as_slice())),
    ("cat_cstring", |i| base(i).cat(c_prefix(&next_text(i)))),
    ("plus_cstring", |i| &base(i) + c_prefix(&next_text(i))),
    ("cat_view", |i| base(i).cat(next_text(i).as_slice())),
    ("cat_string", |i| base(i).cat(base(i))),
    ("plus_string", |i| &base(i) + &base(i)),
    ("cat_extended", |i| {
        let head = base(i);
        let wide = next_extended(i);
        head.cat_extended(&wide, Some(next_char(i)))
    }),
    ("plus_extended", |i| &base(i) + &next_extended(i)),
    ("cat_wide", |i| base(i).cat_wide(&next_wide(i))),
    ("plus_wide", |i| base(i).cat_wide(&next_wide(i))),
];

/// Every comparison overload of `s` against `t`, in the C++ `compare_all` order.
fn compare_all(s: &AsciiString, t: &[u8]) -> String {
    let (other, c) = (make(t), c_prefix(t));
    let flags = [
        s.is_equal(&other),
        s.is_equal(c),
        s.is_equal(t),
        s.is_equal(t),
        *s == other,
        s.is_equal(c),
        s.is_different(&other),
        s.is_different(c),
        *s != other,
        s.is_less(&other),
        s.is_less(c),
        s.is_less(t),
        *s < other,
        s.is_greater(&other),
        s.is_greater(c),
        s.is_greater(t),
        *s > other,
        s.starts_with(&other),
        s.starts_with(c),
        s.starts_with(t),
        s.ends_with(&other),
        s.ends_with(t),
        AsciiString::strings_equal(s, &other),
        AsciiString::strings_equal(s, t),
        AsciiString::same_bytes(s, &other, true),
        AsciiString::same_bytes(s, &other, false),
        AsciiString::same_bytes(s, c, false),
        AsciiString::same_bytes(c_prefix(s.as_bytes()), c, false),
        AsciiString::same_bytes(s, t, false),
    ];
    flags.iter().map(|&value| flag(value)).collect()
}

const QUERIES: &[(&str, Query)] = &[
    ("value", |s, i| {
        s.value(next_int(i)).map(|byte| byte.to_string())
    }),
    ("sub_string", |s, i| {
        let from = next_int(i);
        s.sub_string(from, next_int(i)).map(|part| show(&part))
    }),
    ("search_span", |s, i| Ok(s.search(next_text(i)).to_string())),
    ("search_cstring", |s, i| {
        Ok(s.search(c_prefix(&next_text(i))).to_string())
    }),
    ("search_string", |s, i| {
        Ok(s.search(make(&next_text(i))).to_string())
    }),
    ("search_from_end_span", |s, i| {
        Ok(s.search_from_end(next_text(i)).to_string())
    }),
    ("search_from_end_cstring", |s, i| {
        Ok(s.search_from_end(c_prefix(&next_text(i))).to_string())
    }),
    ("location", |s, i| {
        let (what, from) = (next_text(i), next_int(i));
        s.location(what, from, next_int(i))
            .map(|found| found.to_string())
    }),
    ("location_char", |s, i| {
        let (count, byte, from) = (next_int(i), next_char(i), next_int(i));
        s.location_char(count, byte, from, next_int(i))
            .map(|found| found.to_string())
    }),
    ("in_set", |s, i| {
        let (set, from) = (next_text(i), next_int(i));
        s.first_location_in_set(set, from, next_int(i))
            .map(|found| found.to_string())
    }),
    ("not_in_set", |s, i| {
        let (set, from) = (next_text(i), next_int(i));
        s.first_location_not_in_set(set, from, next_int(i))
            .map(|found| found.to_string())
    }),
    ("token", |s, i| {
        let separators = next_text(i);
        Ok(show(&s.token(separators, next_int(i))))
    }),
    ("token_default", |s, _| Ok(show(&s.token(" \t", 1)))),
    ("compare", |s, i| Ok(compare_all(s, &next_text(i)))),
    ("properties", |s, _| Ok(properties(s))),
    ("integer_value", |s, _| {
        s.integer_value().map(|value| value.to_string())
    }),
    ("real_value", |s, _| s.real_value().map(real_bits)),
    ("print", |s, _| {
        let mut output = Vec::new();
        s.print(&mut output).expect("in-memory output");
        s.print(&mut output).expect("in-memory output");
        Ok(hex(&output))
    }),
];

/// Flags, useful length and both hashes, in the C++ `properties` order.
fn properties(s: &AsciiString) -> String {
    let flags = [
        s.is_ascii(),
        s.is_empty(),
        s.is_integer_value(),
        s.is_real_value(false),
        s.is_real_value(true),
    ];
    let digits: String = flags.iter().map(|&value| flag(value)).collect();
    let hash = s.hash_code();
    format!("{digits} {} {hash} {hash}", s.useful_length())
}
