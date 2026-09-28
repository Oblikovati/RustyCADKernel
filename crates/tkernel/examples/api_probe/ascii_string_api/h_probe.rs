//! HAsciiString cases; twin of tkernel_ascii_string_api_h_probe.cxx. A handle field is the
//! hex of its string, or "null" for a null handle.
use super::{
    Input, Outcome, flag, make, member, next_char, next_extended, next_flag, next_int, next_real,
    next_text, real_bits, respond, show,
};
use crate::common::{decode, hex};
use rustycad_tkernel::standard::{Handle, RuntimeTyped, StandardTransient, Transient};
use rustycad_tkernel::tcollection::HAsciiString;

type HMake = fn(&mut Input<'_>) -> HAsciiString;
type HEdit = fn(&HAsciiString, &mut Input<'_>) -> Outcome;

fn next_handle(input: &mut Input<'_>) -> Handle<HAsciiString> {
    let token = input.next().expect("handle field");
    if token == "null" {
        return Handle::default();
    }
    Handle::new(HAsciiString::from(make(&decode(token))))
}

fn show_handle(text: &HAsciiString) -> String {
    show(&text.string())
}

/// "ascii_h_make <member> <fields...>".
pub(super) fn h_make_case(input: &mut Input<'_>) -> String {
    let construct = member(H_MAKERS, input);
    respond(Ok(show_handle(&construct(input))))
}

/// "ascii_h_edit <member> <handle> <fields...>": the edited string and any extra output.
pub(super) fn h_edit_case(input: &mut Input<'_>) -> String {
    let edit = member(H_EDITS, input);
    let text = next_handle(input);
    respond(edit(&text, input).map(|extra| show_handle(&text) + &extra))
}

/// "ascii_h_query <member> <handle> <fields...>".
pub(super) fn h_query_case(input: &mut Input<'_>) -> String {
    let query = member(H_QUERIES, input);
    let text = next_handle(input);
    respond(query(&text, input))
}

fn done() -> Outcome {
    Ok(String::new())
}

fn checked(result: Result<(), rustycad_tkernel::standard::Failure>) -> Outcome {
    result.map(|()| String::new())
}

fn extra(result: Handle<HAsciiString>) -> String {
    format!(" {}", show_handle(&result))
}

const H_MAKERS: &[(&str, HMake)] = &[
    ("default", |_| HAsciiString::default()),
    ("cstring", |i| HAsciiString::new(next_text(i))),
    ("char", |i| HAsciiString::from_char(next_char(i))),
    ("filled", |i| {
        let length = next_int(i) as usize;
        HAsciiString::filled(length, next_char(i))
    }),
    ("int", |i| HAsciiString::from_integer(next_int(i))),
    ("real", |i| HAsciiString::from_real(next_real(i))),
    ("string", |i| {
        HAsciiString::from_string(&make(&next_text(i)))
    }),
    ("moved_string", |i| HAsciiString::from(make(&next_text(i)))),
    ("handle", |i| HAsciiString::copied(&next_handle(i))),
    ("extended", |i| {
        let wide = next_extended(i);
        HAsciiString::from_extended(&wide, next_char(i))
    }),
];

const H_EDITS: &[(&str, HEdit)] = &[
    ("assign_cat", |h, i| {
        h.assign_cat(next_text(i));
        done()
    }),
    ("assign_cat_handle", |h, i| {
        h.assign_cat_string(&next_handle(i));
        done()
    }),
    ("assign_cat_self", |h, _| {
        h.assign_cat_string(h);
        done()
    }),
    ("cat", |h, i| Ok(extra(h.cat(next_text(i))))),
    ("cat_handle", |h, i| {
        Ok(extra(h.cat_string(&next_handle(i))))
    }),
    ("capitalize", |h, _| {
        h.capitalize();
        done()
    }),
    ("upper", |h, _| {
        h.upper_case();
        done()
    }),
    ("lower", |h, _| {
        h.lower_case();
        done()
    }),
    ("center", |h, i| {
        let w = next_int(i);
        checked(h.center(w, next_char(i)))
    }),
    ("left_justify", |h, i| {
        let w = next_int(i);
        checked(h.left_justify(w, next_char(i)))
    }),
    ("right_justify", |h, i| {
        let w = next_int(i);
        checked(h.right_justify(w, next_char(i)))
    }),
    ("change_all", |h, i| {
        let (from, to) = (next_char(i), next_char(i));
        h.change_all(from, to, next_flag(i));
        done()
    }),
    ("clear", |h, _| {
        h.clear();
        done()
    }),
    ("left_adjust", |h, _| {
        h.left_adjust();
        done()
    }),
    ("right_adjust", |h, _| {
        h.right_adjust();
        done()
    }),
    ("insert_char", |h, i| {
        let p = next_int(i);
        checked(h.insert_char(p, next_char(i)))
    }),
    ("insert", |h, i| {
        let p = next_int(i);
        checked(h.insert(p, next_text(i)))
    }),
    ("insert_handle", |h, i| {
        let p = next_int(i);
        checked(h.insert_string(p, &next_handle(i)))
    }),
    ("insert_self", |h, i| {
        checked(h.insert_string(next_int(i), h))
    }),
    ("insert_after", |h, i| {
        let p = next_int(i);
        checked(h.insert_after(p, &next_handle(i)))
    }),
    ("insert_before", |h, i| {
        let p = next_int(i);
        checked(h.insert_before(p, &next_handle(i)))
    }),
    ("prepend", |h, i| {
        h.prepend(&next_handle(i));
        done()
    }),
    ("prepend_self", |h, _| {
        h.prepend(h);
        done()
    }),
    ("remove_all", |h, i| {
        h.remove_all(next_char(i));
        done()
    }),
    ("remove_all_case", |h, i| {
        let byte = next_char(i);
        h.remove_all_with_case(byte, next_flag(i));
        done()
    }),
    ("remove", |h, i| {
        let p = next_int(i);
        checked(h.remove(p, next_int(i)))
    }),
    ("set_value_char", |h, i| {
        let p = next_int(i);
        checked(h.set_value_char(p, next_char(i)))
    }),
    ("set_value", |h, i| {
        let p = next_int(i);
        checked(h.set_value(p, next_text(i)))
    }),
    ("set_value_handle", |h, i| {
        let p = next_int(i);
        checked(h.set_value_string(p, &next_handle(i)))
    }),
    ("split", |h, i| h.split(next_int(i)).map(extra)),
    ("sub_string", |h, i| {
        let from = next_int(i);
        h.sub_string(from, next_int(i)).map(extra)
    }),
    ("token", |h, i| {
        let separators = next_text(i);
        Ok(extra(h.token(separators, next_int(i))))
    }),
    ("token_default", |h, _| Ok(extra(h.token(" \t", 1)))),
    ("trunc", |h, i| checked(h.truncate(next_int(i)))),
];

const H_QUERIES: &[(&str, HEdit)] = &[
    (
        "is_less",
        |h, i| Ok(flag(h.is_less(&next_handle(i))).into()),
    ),
    ("is_greater", |h, i| {
        Ok(flag(h.is_greater(&next_handle(i))).into())
    }),
    ("is_different", |h, i| {
        h.is_different(&next_handle(i)).map(|v| flag(v).into())
    }),
    ("is_same_string", |h, i| {
        h.is_same_string(&next_handle(i)).map(|v| flag(v).into())
    }),
    ("is_same_string_case", |h, i| {
        let other = next_handle(i);
        h.is_same_string_with_case(&other, next_flag(i))
            .map(|v| flag(v).into())
    }),
    ("is_same_state", |h, i| {
        Ok(flag(h.is_same_state(&next_handle(i))).into())
    }),
    ("in_set", |h, i| {
        let (set, from) = (next_handle(i), next_int(i));
        h.first_location_in_set(&set, from, next_int(i))
            .map(|v| v.to_string())
    }),
    ("not_in_set", |h, i| {
        let (set, from) = (next_handle(i), next_int(i));
        h.first_location_not_in_set(&set, from, next_int(i))
            .map(|v| v.to_string())
    }),
    ("location", |h, i| {
        let (what, from) = (next_handle(i), next_int(i));
        h.location(&what, from, next_int(i)).map(|v| v.to_string())
    }),
    ("location_char", |h, i| {
        let (count, byte, from) = (next_int(i), next_char(i), next_int(i));
        h.location_char(count, byte, from, next_int(i))
            .map(|v| v.to_string())
    }),
    ("search", |h, i| Ok(h.search(next_text(i)).to_string())),
    ("search_handle", |h, i| {
        Ok(h.search_string(&next_handle(i)).to_string())
    }),
    ("search_from_end", |h, i| {
        Ok(h.search_from_end(next_text(i)).to_string())
    }),
    ("search_from_end_handle", |h, i| {
        Ok(h.search_from_end_string(&next_handle(i)).to_string())
    }),
    ("value", |h, i| h.value(next_int(i)).map(|v| v.to_string())),
    ("integer_value", |h, _| {
        h.integer_value().map(|v| v.to_string())
    }),
    ("real_value", |h, _| h.real_value().map(real_bits)),
    ("properties", |h, _| Ok(h_properties(h))),
];

/// Flags, sizes, `ToCString`, `Print` and the runtime type, in the C++ `properties` order.
fn h_properties(h: &HAsciiString) -> String {
    let mut printed = Vec::new();
    h.print(&mut printed).expect("in-memory output");
    let flags = [
        h.is_ascii(),
        h.is_empty(),
        h.is_integer_value(),
        h.is_real_value(),
    ];
    let digits: String = flags.iter().map(|&value| flag(value)).collect();
    let c_string = h.as_bytes_with_nul();
    let kind = Transient::dynamic_type(h);
    format!(
        "{digits} {} {} {} {} {} {}",
        h.length(),
        h.useful_length(),
        hex(&c_string[..c_string.len() - 1]),
        hex(&printed),
        kind.name(),
        flag(kind.is_subtype(StandardTransient::static_type())),
    )
}
