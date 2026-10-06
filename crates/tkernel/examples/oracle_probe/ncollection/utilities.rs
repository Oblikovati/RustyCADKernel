//! NCollection utilities probe operations (twin of `tkernel_ncollection_utilities_probe.cxx`).
use rustycad_tkernel::ncollection::{String8, UtfIterator, UtfString, UtfUnit, WideChar};
use std::str::SplitWhitespace;

/// The output of a utilities operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    match op {
        "ncoll_utilities_locale" => Some(locale_probe(input)),
        "ncoll_utilities_utf" => Some(match input.next().expect("unit kind") {
            "8" => utf_probe::<u8>(input),
            "16" => utf_probe::<u16>(input),
            _ => utf_probe::<u32>(input),
        }),
        _ => None,
    }
}

/// Hex units joined by commas, `-` for none (the native `utf_units`).
fn units_text<T: UtfUnit>(units: &[T]) -> String {
    if units.is_empty() {
        return "-".to_string();
    }
    let hex: Vec<String> = units
        .iter()
        .map(|&unit| format!("{:x}", unit.to_bits()))
        .collect();
    hex.join(",")
}

fn read_source<T: UtfUnit>(input: &mut SplitWhitespace<'_>) -> Vec<T> {
    input
        .map(|token| T::from_bits(u32::from_str_radix(token, 16).expect("hex unit")))
        .collect()
}

fn bit(value: bool) -> u8 {
    u8::from(value)
}

fn walk_symbol<F: UtfUnit>(iter: &UtfIterator<'_, F>) -> String {
    let (mut utf8, mut utf16) = ([0u8; 8], [0u16; 4]);
    let count8 = iter.get_utf8(&mut utf8).expect("8 bytes hold any symbol");
    let count16 = iter.get_utf16(&mut utf16).expect("4 units hold any symbol");
    format!(
        " {:x}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        iter.current(),
        iter.index(),
        iter.buffer_here(),
        iter.buffer_next(),
        bit(iter.is_valid()),
        iter.advance_bytes_utf8(),
        iter.advance_bytes_utf16(),
        iter.advance_code_units_utf16(),
        iter.advance_bytes_utf32(),
        iter.advance_bytes_utf::<u16>(),
        units_text(&utf8[..count8]),
        units_text(&utf16[..count16])
    )
}

fn walk<F: UtfUnit>(source: &[F]) -> String {
    let mut out = String::from(" walk");
    let mut iter = UtfIterator::new(source);
    let mut count = 0;
    while count < 64 && iter.current() != 0 {
        out += &walk_symbol(&iter);
        count += 1;
        iter.advance();
    }
    out
}

fn state<T: UtfUnit>(text: &UtfString<T>) -> String {
    format!(
        " {}/{}/{}/{}",
        text.size(),
        text.length(),
        bit(text.is_empty()),
        units_text(text.to_c_string())
    )
}

fn queries<T: UtfUnit>(text: &UtfString<T>) -> String {
    let mut out = String::from(" chars");
    for index in -1..=text.length() + 1 {
        let offset = text
            .get_char_buffer(index)
            .map_or(-1, |rest| (text.to_c_string().len() - rest.len()) as i64);
        out += &format!(" {:x}@{offset}", text.get_char(index));
    }
    out += " sub";
    for (start, end) in [(1, 3), (0, 1), (2, 2)] {
        out += &state(&text.sub_string(start, end));
    }
    out += " to";
    out += &state(&text.to_utf8());
    out += &state(&text.to_utf16());
    out += &state(&text.to_utf32());
    out + &state(&text.to_utf_wide())
}

fn joins<T: UtfUnit>(text: &UtfString<T>) -> String {
    let mut appended = text.clone();
    appended += text;
    format!(
        " join{}{} {}{}",
        state(&(text + text)),
        state(&appended),
        bit(appended == *text),
        bit(text.sub_string(0, 1000) != *text)
    )
}

fn locale<T: UtfUnit>(text: &UtfString<T>) -> String {
    let mut out = String::from(" locale");
    for size in [0usize, 1, 2, 5, 16] {
        let mut buffer = [0x55u8; 24];
        let done = text.to_locale(&mut buffer[..size]);
        out += &format!(" {}/{}", bit(done), units_text(&buffer[..size]));
    }
    out
}

fn convert<T: UtfUnit, F: UtfUnit>(source: &[F]) -> String {
    let mut out = String::new();
    for length in [-1, 2] {
        let text = UtfString::<T>::from_unicode_units(source, length);
        out += " |";
        out += &state(&text);
        if length < 0 {
            out += &(queries(&text) + &joins(&text) + &locale(&text));
        }
    }
    out
}

fn utf_probe<F: UtfUnit>(input: &mut SplitWhitespace<'_>) -> String {
    let source = read_source::<F>(input);
    let mut out = walk(&source);
    out += &convert::<u8, F>(&source);
    out += &convert::<u16, F>(&source);
    out += &convert::<u32, F>(&source);
    out + &convert::<WideChar, F>(&source)
}

fn locale_probe(input: &mut SplitWhitespace<'_>) -> String {
    let source = read_source::<u8>(input);
    let mut out = String::new();
    for length in [-1, 1] {
        let mut text = String8::from_unicode_units(b"seed", -1);
        text.from_locale(&source, length);
        out += &state(&text);
    }
    out
}
