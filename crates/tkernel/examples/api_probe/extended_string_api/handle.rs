//! HExtendedString cases (tkernel_extended_string_api_handle.cxx).
use super::{SubCase, describe, dispatch, flag, guarded_kind, next_string, next_units, units_hex};
use crate::common::{hex, next, next_bytes};
use rustycad_tkernel::{
    ncollection::HashPolicy,
    standard::{Failure, Handle},
    tcollection::{
        AsciiString, ExtendedString, HAsciiString, HExtendedString, HExtendedStringHasher,
    },
};
use std::str::SplitWhitespace;

type HString = Handle<HExtendedString>;
type HEdit = fn(&HString, &mut SplitWhitespace<'_>) -> Result<String, Failure>;

fn next_handle(input: &mut SplitWhitespace<'_>) -> HString {
    Handle::new(HExtendedString::from(next_string(input)))
}

fn describe_handle(string: &HExtendedString) -> String {
    describe(&string.string())
}

const HNEW: &[(&str, SubCase)] = &[
    ("empty", |_| describe_handle(&HExtendedString::default())),
    ("cstr", |input| {
        describe_handle(&HExtendedString::from_c_string(&next_bytes(input)))
    }),
    ("u16", |input| {
        describe_handle(&HExtendedString::from_utf16(&next_units(input)))
    }),
    ("unit", |input| {
        describe_handle(&HExtendedString::from_char(next(input)))
    }),
    ("fill", |input| {
        let length = next::<i32>(input);
        let object = HExtendedString::filled(length, next(input));
        guarded_kind(object.map(|object| describe_handle(&object)))
    }),
    ("str", |input| {
        let string: ExtendedString = next_string(input);
        let copied = HExtendedString::from(&string);
        format!(
            "{} {}",
            describe_handle(&copied),
            describe_handle(&HExtendedString::from(string))
        )
    }),
    ("hascii", |input| {
        // The native probe builds the string byte by byte, so embedded NULs stay.
        let ascii = HAsciiString::from(AsciiString::from_raw(&next_bytes(input)));
        describe_handle(&HExtendedString::from_h_ascii_string(&ascii))
    }),
    ("hcopy", |input| {
        let source = next_handle(input);
        let copy = Handle::new(HExtendedString::from_h_extended_string(&source));
        source.assign_cat(&source);
        format!("{} {}", describe_handle(&copy), describe_handle(&source))
    }),
];

/// "hnew <kind> <args>": one HExtendedString constructor.
pub fn hnew_case(input: &mut SplitWhitespace<'_>) -> String {
    dispatch(HNEW, input)
}

const HEDIT: &[(&str, HEdit)] = &[
    ("assign_cat", |h, input| {
        h.assign_cat(&next_handle(input));
        Ok(describe_handle(h))
    }),
    ("assign_self", |h, _| {
        h.assign_cat(h);
        Ok(describe_handle(h))
    }),
    ("cat", |h, input| {
        Ok(format!(
            "{} {}",
            describe_handle(&h.cat(&next_handle(input))),
            describe_handle(h)
        ))
    }),
    ("change", |h, input| {
        h.change_all(next(input), next(input));
        Ok(describe_handle(h))
    }),
    ("clear", |h, _| {
        h.clear();
        Ok(format!("{} {}", describe_handle(h), flag(h.is_empty())))
    }),
    ("insert_unit", |h, input| {
        h.insert_char(next(input), next(input))
            .map(|()| describe_handle(h))
    }),
    ("insert", |h, input| {
        h.insert(next(input), &next_handle(input))
            .map(|()| describe_handle(h))
    }),
    ("query", query),
    ("remove", |h, input| {
        h.remove(next(input), next(input))
            .map(|()| describe_handle(h))
    }),
    ("remove_one", |h, input| {
        h.remove(next(input), 1).map(|()| describe_handle(h))
    }),
    ("remove_all", |h, input| {
        h.remove_all(next(input));
        Ok(describe_handle(h))
    }),
    ("set_unit", |h, input| {
        h.set_value(next(input), next(input))
            .map(|()| describe_handle(h))
    }),
    ("set", |h, input| {
        h.set_value_string(next(input), &next_handle(input))
            .map(|()| describe_handle(h))
    }),
    ("split", |h, input| {
        let tail = h.split(next(input))?;
        Ok(format!("{} {}", describe_handle(&tail), describe_handle(h)))
    }),
    ("token", |h, input| {
        let separators = next_units(input);
        let which = next::<i32>(input);
        let (token, first) = (h.token(&separators, which), h.token(&separators, 1));
        Ok(format!(
            "{} {}",
            describe_handle(&token),
            describe_handle(&first)
        ))
    }),
    ("trunc", |h, input| {
        h.trunc(next(input)).map(|()| describe_handle(h))
    }),
    ("value", |h, input| {
        h.value(next(input)).map(|unit| unit.to_string())
    }),
    ("print", |h, _| {
        let mut printed = Vec::new();
        h.print(&mut printed).expect("in-memory stream");
        Ok(hex(&printed))
    }),
];

fn query(h: &HString, input: &mut SplitWhitespace<'_>) -> Result<String, Failure> {
    let other = next_handle(input);
    let flags = [
        h.is_empty(),
        h.is_less(&other),
        h.is_greater(&other),
        h.is_ascii(),
        h.is_same_state(&other),
    ];
    Ok(format!(
        "{} {} {} {} {}",
        flags.map(flag).concat(),
        h.length(),
        h.search(&other),
        h.search_from_end(&other),
        units_hex(&h.to_ext_string())
    ))
}

/// "hedit <op> <string> <args>": the result, or the exception class.
pub fn hedit_case(input: &mut SplitWhitespace<'_>) -> String {
    let operation = input.next().expect("handle operation");
    let string = next_handle(input);
    let edit = HEDIT
        .iter()
        .find(|(name, _)| *name == operation)
        .expect("handle operation")
        .1;
    guarded_kind(edit(&string, input))
}

/// std::hash and std::equal_to of handles: null, shared and equal-content handles.
pub fn hkey_case(input: &mut SplitWhitespace<'_>) -> String {
    let (first, second, null) = (next_handle(input), next_handle(input), HString::default());
    let policy = HExtendedStringHasher;
    let equal = [
        policy.equal(&first, &second),
        policy.equal(&first, &first),
        policy.equal(&null, &null),
        policy.equal(&first, &null),
        policy.equal(&null, &second),
    ];
    format!(
        "{} {} {}",
        policy.hash(&first),
        policy.hash(&null),
        equal.map(flag).concat()
    )
}
