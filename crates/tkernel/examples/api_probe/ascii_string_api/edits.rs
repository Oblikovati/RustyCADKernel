//! AsciiString members that change the string; twin of tkernel_ascii_string_api_edits.cxx.
use super::{
    Input, Outcome, c_prefix, make, member, next_char, next_extended, next_flag, next_int,
    next_real, next_text, next_wide, respond, show,
};
use rustycad_tkernel::tcollection::AsciiString;

type Edit = fn(&mut AsciiString, &mut Input<'_>) -> Outcome;

/// "ascii_edit <member> <hex string> <fields...>": the edited string and any extra output.
pub(super) fn edit_case(input: &mut Input<'_>) -> String {
    let edit = member(EDITS, input);
    let mut text = make(&next_text(input));
    respond(edit(&mut text, input).map(|extra| show(&text) + &extra))
}

/// No extra output after a member returning nothing.
fn done() -> Outcome {
    Ok(String::new())
}

/// No extra output after a member that may raise.
fn checked(result: Result<(), rustycad_tkernel::standard::Failure>) -> Outcome {
    result.map(|()| String::new())
}

/// `(offset, length)` of a pointer into the string, as a zero-based range.
fn own_span(input: &mut Input<'_>) -> std::ops::Range<usize> {
    let offset = next_int(input) as usize;
    offset..offset + next_int(input) as usize
}

const EDITS: &[(&str, Edit)] = &[
    ("assign_cat_char", |s, i| {
        s.assign_cat(next_char(i));
        done()
    }),
    ("plus_assign_char", |s, i| {
        *s += next_char(i);
        done()
    }),
    ("assign_cat_int", |s, i| {
        s.assign_cat(next_int(i));
        done()
    }),
    ("plus_assign_int", |s, i| {
        *s += next_int(i);
        done()
    }),
    ("assign_cat_real", |s, i| {
        s.assign_cat(next_real(i));
        done()
    }),
    ("plus_assign_real", |s, i| {
        *s += next_real(i);
        done()
    }),
    ("assign_cat_span", |s, i| {
        s.assign_cat(next_text(i).as_slice());
        done()
    }),
    ("assign_cat_cstring", |s, i| {
        s.assign_cat(c_prefix(&next_text(i)));
        done()
    }),
    ("plus_assign_cstring", |s, i| {
        *s += c_prefix(&next_text(i));
        done()
    }),
    ("assign_cat_view", |s, i| {
        s.assign_cat(next_text(i).as_slice());
        done()
    }),
    ("assign_cat_string", |s, i| {
        s.assign_cat(make(&next_text(i)));
        done()
    }),
    ("plus_assign_string", |s, i| {
        *s += &make(&next_text(i));
        done()
    }),
    ("assign_cat_extended", |s, i| {
        let wide = next_extended(i);
        s.assign_cat_extended(&wide, Some(next_char(i)));
        done()
    }),
    ("plus_assign_extended", |s, i| {
        *s += &next_extended(i);
        done()
    }),
    ("assign_cat_wide", |s, i| {
        s.assign_cat_wide(&next_wide(i));
        done()
    }),
    ("plus_assign_wide", |s, i| {
        s.assign_cat_wide(&next_wide(i));
        done()
    }),
    ("assign_cat_self", |s, i| {
        s.assign_cat_self(own_span(i));
        done()
    }),
    ("prepend", |s, i| {
        s.prepend(&make(&next_text(i)));
        done()
    }),
    ("insert_char", |s, i| {
        let position = next_int(i);
        checked(s.insert_char(position, next_char(i)))
    }),
    ("insert_span", |s, i| {
        let position = next_int(i);
        checked(s.insert(position, next_text(i)))
    }),
    ("insert_cstring", |s, i| {
        let position = next_int(i);
        checked(s.insert(position, c_prefix(&next_text(i))))
    }),
    ("insert_string", |s, i| {
        let position = next_int(i);
        checked(s.insert(position, make(&next_text(i))))
    }),
    ("insert_self", |s, i| {
        let position = next_int(i);
        checked(s.insert_self(position, own_span(i)))
    }),
    ("insert_after", |s, i| {
        let index = next_int(i);
        checked(s.insert_after(index, make(&next_text(i))))
    }),
    ("insert_before", |s, i| {
        let index = next_int(i);
        checked(s.insert_before(index, make(&next_text(i))))
    }),
    ("set_value_char", |s, i| {
        let position = next_int(i);
        checked(s.set_value_char(position, next_char(i)))
    }),
    ("set_value_span", |s, i| {
        let position = next_int(i);
        checked(s.set_value(position, next_text(i)))
    }),
    ("set_value_string", |s, i| {
        let position = next_int(i);
        checked(s.set_value(position, make(&next_text(i))))
    }),
    ("remove", |s, i| {
        let position = next_int(i);
        checked(s.remove(position, next_int(i)))
    }),
    ("remove_all", |s, i| {
        s.remove_all(next_char(i));
        done()
    }),
    ("remove_all_case", |s, i| {
        let byte = next_char(i);
        s.remove_all_with_case(byte, next_flag(i));
        done()
    }),
    ("trunc", |s, i| checked(s.truncate(next_int(i)))),
    ("split", |s, i| {
        s.split(next_int(i)).map(|tail| format!(" {}", show(&tail)))
    }),
    ("copy_span", |s, i| {
        s.copy(next_text(i));
        done()
    }),
    ("copy_cstring", |s, i| {
        s.copy(c_prefix(&next_text(i)));
        done()
    }),
    ("copy_view", |s, i| {
        s.copy(next_text(i));
        done()
    }),
    ("copy_string", |s, i| {
        s.clone_from(&make(&next_text(i)));
        done()
    }),
    ("copy_self", |s, i| {
        s.copy_self(own_span(i));
        done()
    }),
    ("move", |s, i| {
        let mut other = make(&next_text(i));
        s.move_assign(&mut other);
        Ok(format!(" {}", show(&other)))
    }),
    ("swap", |s, i| {
        let mut other = make(&next_text(i));
        s.swap(&mut other);
        Ok(format!(" {}", show(&other)))
    }),
    ("clear", |s, _| {
        s.clear();
        done()
    }),
    ("center", |s, i| {
        let width = next_int(i);
        checked(s.center(width, next_char(i)))
    }),
    ("left_justify", |s, i| {
        let width = next_int(i);
        checked(s.left_justify(width, next_char(i)))
    }),
    ("right_justify", |s, i| {
        let width = next_int(i);
        checked(s.right_justify(width, next_char(i)))
    }),
    ("left_adjust", |s, _| {
        s.left_adjust();
        done()
    }),
    ("right_adjust", |s, _| {
        s.right_adjust();
        done()
    }),
    ("change_all", |s, i| {
        let (from, to) = (next_char(i), next_char(i));
        s.change_all(from, to, next_flag(i));
        done()
    }),
    ("upper", |s, _| {
        s.upper_case();
        done()
    }),
    ("lower", |s, _| {
        s.lower_case();
        done()
    }),
    ("capitalize", |s, _| {
        s.capitalize();
        done()
    }),
];
