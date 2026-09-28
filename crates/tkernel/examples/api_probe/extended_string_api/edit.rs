//! Append and in-place edit cases (tkernel_extended_string_api_edit.cxx).
use super::{c_units, describe, guarded_kind, next_real, next_span, next_string, next_units};
use crate::common::next;
use rustycad_tkernel::{standard::Failure, tcollection::ExtendedString};
use std::str::SplitWhitespace;

type Append = fn(&ExtendedString, &mut SplitWhitespace<'_>) -> String;
type Edit = fn(&mut ExtendedString, &mut SplitWhitespace<'_>) -> Result<(), Failure>;

/// "<AssignCat> <operator+=> <Cat> <operator+>" for one appended value.
fn append_all<T: Into<ExtendedString> + Clone>(string: &ExtendedString, value: T) -> String {
    let (mut assigned, mut added) = (string.clone(), string.clone());
    assigned.assign_cat(value.clone());
    added += value.clone();
    let joined = [assigned, added, string.cat(value.clone()), string + value];
    joined.iter().map(describe).collect::<Vec<_>>().join(" ")
}

const APPEND: &[(&str, Append)] = &[
    ("str", |string, input| {
        append_all(string, &next_string(input))
    }),
    ("u16", |string, input| {
        append_all(string, c_units(&next_units(input)))
    }),
    ("int", |string, input| {
        append_all(string, next::<i32>(input))
    }),
    ("real", |string, input| append_all(string, next_real(input))),
    ("char", |string, input| {
        append_all(string, next::<u8>(input))
    }),
    ("unit", |string, input| {
        let unit = next::<u16>(input);
        let mut assigned = string.clone();
        assigned.assign_cat(unit);
        let joined = [assigned, string.cat(unit), string + unit];
        joined.iter().map(describe).collect::<Vec<_>>().join(" ")
    }),
    ("view", |string, input| {
        let units = next_units(input);
        let (mut assigned, mut added) = (string.clone(), string.clone());
        assigned.assign_cat(units.as_slice());
        added += units.as_slice();
        format!("{} {}", describe(&assigned), describe(&added))
    }),
    ("span", span_append),
    ("prepend", |string, input| {
        let units = next_units(input);
        let (mut by_string, mut by_pointer) = (string.clone(), string.clone());
        by_string.prepend(ExtendedString::from_raw(&units));
        by_pointer.prepend(c_units(&units));
        format!("{} {}", describe(&by_string), describe(&by_pointer))
    }),
    ("self", |string, input| {
        let (offset, length) = (next::<usize>(input), next::<usize>(input));
        let mut whole = string.clone();
        whole.assign_cat_self(0..whole.len());
        let (mut appended, mut prepended) = (string.clone(), string.clone());
        appended.assign_cat_self(offset..offset + length);
        prepended.prepend_self(offset..offset + length);
        [whole, appended, prepended]
            .iter()
            .map(describe)
            .collect::<Vec<_>>()
            .join(" ")
    }),
];

fn span_append(string: &ExtendedString, input: &mut SplitWhitespace<'_>) -> String {
    let span = next_span(input);
    let length = next::<i32>(input);
    let (mut assigned, mut prepended) = (string.clone(), string.clone());
    assigned.assign_cat_span(span.as_deref(), length);
    prepended.prepend_span(span.as_deref(), length);
    let cat = string.cat_span(span.as_deref(), length);
    format!(
        "{} {} {}",
        describe(&assigned),
        describe(&prepended),
        guarded_kind(cat.map(|result| describe(&result)))
    )
}

/// "append <kind> <string> <args>".
pub fn append_case(input: &mut SplitWhitespace<'_>) -> String {
    let kind = input.next().expect("append kind");
    let string = next_string(input);
    let append = APPEND
        .iter()
        .find(|(name, _)| *name == kind)
        .expect("append kind")
        .1;
    append(&string, input)
}

const EDIT: &[(&str, Edit)] = &[
    ("insert_unit", |s, input| {
        s.insert_char(next(input), next(input))
    }),
    ("insert_str", |s, input| {
        s.insert(next(input), next_string(input))
    }),
    ("insert_u16", |s, input| {
        let index = next(input);
        s.insert(index, c_units(&next_units(input)))
    }),
    ("insert_span", |s, input| {
        let (index, span) = (next(input), next_span(input));
        s.insert_span(index, span.as_deref(), next(input))
    }),
    ("set_unit", |s, input| s.set_value(next(input), next(input))),
    ("set_str", |s, input| {
        s.set_value_units(next(input), next_string(input))
    }),
    ("set_u16", |s, input| {
        let index = next(input);
        s.set_value_units(index, c_units(&next_units(input)))
    }),
    ("set_span", |s, input| {
        let (index, span) = (next(input), next_span(input));
        s.set_value_span(index, span.as_deref(), next(input))
    }),
    ("remove", |s, input| s.remove(next(input), next(input))),
    ("remove_one", |s, input| s.remove(next(input), 1)),
    ("remove_all", |s, input| {
        s.remove_all(next(input));
        Ok(())
    }),
    ("trunc", |s, input| s.trunc(next(input))),
    ("split", |s, input| {
        let tail = s.split(next(input))?;
        s.assign_cat(0x7C_u16);
        s.assign_cat(&tail);
        Ok(())
    }),
    ("sub", |s, input| {
        *s = s.sub_string(next(input), next(input))?;
        Ok(())
    }),
    ("change", |s, input| {
        s.change_all(next(input), next(input));
        Ok(())
    }),
    ("copy_span", |s, input| {
        let span = next_span(input);
        s.copy_span(span.as_deref(), next(input));
        Ok(())
    }),
    ("copy_u16", |s, input| {
        s.copy(c_units(&next_units(input)));
        Ok(())
    }),
    ("copy_str", |s, input| {
        s.copy(next_string(input));
        Ok(())
    }),
    ("assign_u16", |s, input| {
        s.copy(c_units(&next_units(input)));
        Ok(())
    }),
    ("assign_view", |s, input| {
        s.copy(next_units(input));
        Ok(())
    }),
    ("clear", |s, _| {
        s.clear();
        Ok(())
    }),
    ("swap", |s, input| {
        let mut other = next_string(input);
        s.swap(&mut other);
        s.assign_cat(0x7C_u16);
        s.assign_cat(&other);
        Ok(())
    }),
    ("move", |s, input| {
        let mut source = next_string(input);
        s.move_assign(&mut source);
        s.assign_cat(0x7C_u16);
        s.assign_cat(&source);
        Ok(())
    }),
    ("left_adjust", |s, _| {
        s.left_adjust();
        Ok(())
    }),
    ("right_adjust", |s, _| {
        s.right_adjust();
        Ok(())
    }),
    ("capitalize", |s, _| {
        s.capitalize();
        Ok(())
    }),
    ("left_justify", |s, input| {
        s.left_justify(next(input), next(input))
    }),
    ("right_justify", |s, input| {
        s.right_justify(next(input), next(input))
    }),
    ("center", |s, input| s.center(next(input), next(input))),
];

/// "edit <op> <string> <args>": the edited string, or the exception class.
pub fn edit_case(input: &mut SplitWhitespace<'_>) -> String {
    let operation = input.next().expect("edit operation");
    let mut string = next_string(input);
    let edit = EDIT
        .iter()
        .find(|(name, _)| *name == operation)
        .expect("edit operation")
        .1;
    guarded_kind(edit(&mut string, input).map(|()| describe(&string)))
}
