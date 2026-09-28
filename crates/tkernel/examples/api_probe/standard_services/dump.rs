//! Rust side of the Standard_Dump cases (tkernel_standard_services_dump.cxx).
use crate::common::{decode, guarded, hex, next, next_bytes};
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::standard::dump::{
    self, DumpOutput, DumpStream, DumpValue, DumpValues, JsonKey,
};
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::io::{self, Cursor, Write};
use std::str::SplitWhitespace;

/// The bytes the native probe's `TCollection_AsciiString(data, length)` keeps: that
/// constructor stops at the first NUL.
fn ascii_bytes(input: &mut SplitWhitespace<'_>) -> Vec<u8> {
    let bytes = next_bytes(input);
    bytes
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or_default()
        .to_vec()
}

/// A readable string stream (modes s and o) or a plain std::ostream whose content
/// Standard_Dump cannot read (mode x).
enum ProbeStream {
    Readable(DumpStream),
    Opaque(Cursor<Vec<u8>>),
}
impl DumpOutput for ProbeStream {
    fn position(&self) -> i64 {
        match self {
            Self::Readable(stream) => stream.position(),
            Self::Opaque(cursor) => cursor.position() as i64,
        }
    }
    fn content(&self) -> Option<&[u8]> {
        match self {
            Self::Readable(stream) => Some(stream.bytes()),
            Self::Opaque(_) => None,
        }
    }
    fn write_fragment(&mut self, bytes: &[u8]) -> io::Result<()> {
        match self {
            Self::Readable(stream) => stream.write_fragment(bytes),
            Self::Opaque(cursor) => cursor.write_all(bytes),
        }
    }
}

/// Runs `writer` on a stream holding `prefix` with its put position at `position`.
fn with_dump_stream(
    mode: &str,
    prefix: &[u8],
    position: u64,
    writer: impl FnOnce(&mut ProbeStream) -> io::Result<()>,
) -> String {
    let mut stream = if mode == "x" {
        ProbeStream::Opaque(Cursor::default())
    } else {
        ProbeStream::Readable(DumpStream::default())
    };
    stream.write_fragment(prefix).expect("in-memory write");
    match &mut stream {
        ProbeStream::Readable(readable) => readable.seek(position),
        ProbeStream::Opaque(cursor) => cursor.set_position(position),
    }
    writer(&mut stream).expect("in-memory write");
    let bytes = match &stream {
        ProbeStream::Readable(readable) => readable.bytes().to_vec(),
        ProbeStream::Opaque(cursor) => cursor.get_ref().clone(),
    };
    format!("{} pos={}", hex(&bytes), stream.position())
}

/// "services_dump_separator <s|o|x> <hex prefix> <put position>".
pub fn separator_case(input: &mut SplitWhitespace<'_>) -> String {
    let mode = input.next().expect("mode").to_owned();
    let prefix = next_bytes(input);
    let position: u64 = next(input);
    with_dump_stream(&mode, &prefix, position, dump::add_values_separator)
}

/// "services_dump_key_to_class <s|o|x> <hex prefix> <hex key> <hex field>".
pub fn key_to_class_case(input: &mut SplitWhitespace<'_>) -> String {
    let mode = input.next().expect("mode").to_owned();
    let prefix = next_bytes(input);
    let key = next_bytes(input);
    let field = next_bytes(input);
    with_dump_stream(&mode, &prefix, prefix.len() as u64, |stream| {
        dump::dump_key_to_class(stream, &key, &field)
    })
}

/// "services_dump_character_values <hex>...".
pub fn character_values_case(input: &mut SplitWhitespace<'_>) -> String {
    let values: Vec<Vec<u8>> = input.map(decode).collect();
    let mut stream = DumpStream::default();
    dump::dump_character_values(&mut stream, &values).expect("in-memory write");
    hex(stream.bytes())
}

/// "services_dump_real_values <bits>...".
pub fn real_values_case(input: &mut SplitWhitespace<'_>) -> String {
    let values: Vec<f64> = input
        .map(|bits| f64::from_bits(bits.parse().expect("bits")))
        .collect();
    let mut stream = DumpStream::default();
    dump::dump_real_values(&mut stream, &values).expect("in-memory write");
    hex(stream.bytes())
}

/// "services_dump_field_to_name <hex>".
pub fn field_to_name_case(input: &mut SplitWhitespace<'_>) -> String {
    guarded(dump::dump_field_to_name(ascii_bytes(input)).map(|name| hex(name.as_bytes())))
}

/// "services_dump_format <indent> <hex>": Text and FormatJson.
pub fn format_case(input: &mut SplitWhitespace<'_>) -> String {
    let indent: i32 = next(input);
    let mut stream = DumpStream::default();
    stream
        .write_fragment(&next_bytes(input))
        .expect("in-memory write");
    format!(
        "{} {}",
        hex(dump::text(&stream).as_bytes()),
        hex(dump::format_json(&stream, indent).as_bytes())
    )
}

/// "services_dump_split <hex>": SplitJson, HasChildKey and HierarchicalValueIndices.
pub fn split_case(input: &mut SplitWhitespace<'_>) -> String {
    let text = ascii_bytes(input);
    guarded(split_text(&text))
}

fn split_text(text: &[u8]) -> Result<String, rustycad_tkernel::standard::Failure> {
    let mut values = DumpValues::default();
    let mut texts: IndexedDataMap<AsciiString, AsciiString> = IndexedDataMap::default();
    let mut result = if dump::split_json(text, &mut values)? {
        "1"
    } else {
        "0"
    }
    .to_owned();
    for index in 1..=values.extent() {
        let key = values.find_key(index)?;
        let value = values.find_from_index(index)?;
        texts.add(key.clone(), value.value.clone());
        let child = if dump::has_child_key(&value.value) {
            "c"
        } else {
            ""
        };
        result += &format!(
            " {}={}@{}{child}",
            hex(key.as_bytes()),
            hex(value.value.as_bytes()),
            value.start_position
        );
    }
    let indices: String = dump::hierarchical_value_indices(&texts)?
        .iter()
        .map(|index| format!("{index},"))
        .collect();
    Ok(format!("{result} hier={indices}"))
}

/// "services_dump_keys": key texts and lengths, the pointer prefix and DumpValue.
pub fn keys_case(_: &mut SplitWhitespace<'_>) -> String {
    let keys = [
        JsonKey::None,
        JsonKey::OpenChild,
        JsonKey::CloseChild,
        JsonKey::OpenContainer,
        JsonKey::CloseContainer,
        JsonKey::Quote,
        JsonKey::SeparatorKeyToValue,
        JsonKey::SeparatorValueToValue,
    ];
    let texts: String = keys
        .iter()
        .map(|key| {
            let text = dump::json_key_to_string(*key);
            format!("{}:{} ", hex(text.as_bytes()), dump::json_key_length(*key))
        })
        .collect();
    let empty = DumpValue::default();
    let value = DumpValue::new(AsciiString::new("v"), 7);
    format!(
        "{texts}{} {}@{} {}@{}",
        String::from_utf8_lossy(dump::get_pointer_prefix().as_bytes()),
        String::from_utf8_lossy(empty.value.as_bytes()),
        empty.start_position,
        String::from_utf8_lossy(value.value.as_bytes()),
        value.start_position
    )
}

/// "services_dump_pointer <address> <short>": GetPointerInfo and the null-handle overload.
pub fn pointer_case(input: &mut SplitWhitespace<'_>) -> String {
    let address: usize = next(input);
    let short = next::<i32>(input) != 0;
    let info = dump::get_pointer_info(Some(address), short).expect("pointer text");
    let null: Handle<StandardTransient> = Handle::default();
    let null_info = dump::get_handle_pointer_info(&null, short).expect("null handle");
    format!("{} null={}", hex(info.as_bytes()), null_info.len())
}

/// "services_dump_process <stream|field> <hex text> <hex name> <position>".
pub fn process_case(input: &mut SplitWhitespace<'_>) -> String {
    let kind = input.next().expect("kind").to_owned();
    let text = ascii_bytes(input);
    let name = ascii_bytes(input);
    let mut position: i32 = next(input);
    let found = if kind == "stream" {
        dump::process_stream_name(&text, &name, &mut position)
    } else {
        dump::process_field_name(&text, &name, &mut position)
    };
    guarded(found.map(|found| format!("{} {position}", u8::from(found))))
}

/// "services_dump_init_value <hex text> <position>".
pub fn init_value_case(input: &mut SplitWhitespace<'_>) -> String {
    let text = ascii_bytes(input);
    let mut position: i32 = next(input);
    guarded(
        dump::init_value(&text, &mut position)
            .map(|value| format!("{} {position}", hex(value.as_bytes()))),
    )
}

/// "services_dump_init_reals <hex text> <position> <count>": result, position and outputs.
pub fn init_reals_case(input: &mut SplitWhitespace<'_>) -> String {
    let text = ascii_bytes(input);
    let mut position: i32 = next(input);
    let count: usize = next::<i32>(input).clamp(0, 4) as usize;
    let mut values = vec![0.5; count];
    let result = dump::init_real_values(&text, &mut position, &mut values)
        .map(|done| format!("{} {position}", u8::from(done)));
    let bits: String = values.iter().map(|v| format!(" {}", v.to_bits())).collect();
    guarded(result) + &bits
}
