//! Encodings and printers shared by the storage probe operations.
use rustycad_tkernel::fsd::{BinaryFile, CmpFile, File, MemoryFileSystem};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::{BaseDriver, HeaderData, OpenMode, RootData, TypeData};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use std::rc::Rc;

/// Lowercase hex, `-` for no bytes (`stor_hex`).
pub fn hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "-".into();
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Bytes from hex, `-` for none (`stor_unhex`).
pub fn unhex(text: &str) -> Vec<u8> {
    if text == "-" {
        return Vec::new();
    }
    (0..text.len() / 2)
        .map(|index| u8::from_str_radix(&text[2 * index..2 * index + 2], 16).expect("hex byte"))
        .collect()
}

/// A string argument as the native `stor_unhex(x).c_str()`: up to the first NUL.
pub fn ascii(text: &str) -> AsciiString {
    AsciiString::from_bytes(unhex(text))
}

/// UTF-16 units from four hex digits each; zero units are skipped by `AssignCat`.
pub fn ext(text: &str) -> ExtendedString {
    let mut value = ExtendedString::default();
    if text == "-" {
        return value;
    }
    for index in 0..text.len() / 4 {
        let unit = u16::from_str_radix(&text[4 * index..4 * index + 4], 16).expect("hex unit");
        value.assign_cat(unit);
    }
    value
}

pub fn ascii_hex(text: &AsciiString) -> String {
    hex(text.as_bytes())
}

pub fn ext_hex(text: &ExtendedString) -> String {
    if text.is_empty() {
        return "-".into();
    }
    text.as_utf16().iter().map(|unit| format!("{unit:04x}")).collect()
}

/// `stor_split`: a trailing separator yields a trailing empty part.
pub fn split(text: &str, separator: char) -> Vec<String> {
    text.split(separator).map(str::to_string).collect()
}

pub fn ascii_list(items: &Sequence<AsciiString>) -> String {
    let parts: Vec<String> = items.iter().map(ascii_hex).collect();
    format!("[{}]", parts.join(";"))
}

pub fn ext_list(items: &Sequence<ExtendedString>) -> String {
    let parts: Vec<String> = items.iter().map(ext_hex).collect();
    format!("[{}]", parts.join(";"))
}

pub fn double(text: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(text, 16).expect("hex double"))
}

pub fn float(text: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(text, 16).expect("hex float"))
}

/// `!Class`, the native report of an exception.
pub fn thrown(failure: &Failure) -> String {
    format!("!{}", failure.exception_type())
}

pub fn mode(letter: char) -> OpenMode {
    match letter {
        'r' => OpenMode::Read,
        'w' => OpenMode::Write,
        'u' => OpenMode::ReadWrite,
        _ => OpenMode::None,
    }
}

/// The name every probe driver opens in its file system.
pub fn file_name() -> AsciiString {
    AsciiString::new("file")
}

pub fn driver(kind: char, files: &MemoryFileSystem) -> Box<dyn BaseDriver> {
    let files = Rc::new(files.clone());
    match kind {
        'b' => Box::new(BinaryFile::new(files)),
        'c' => Box::new(CmpFile::new(files)),
        _ => Box::new(File::new(files)),
    }
}

pub fn header_text(header: &Handle<HeaderData>) -> String {
    let header = header.get().expect("header");
    [
        header.error_status().ordinal().to_string(),
        ascii_hex(&header.error_status_extension()),
        header.number_of_objects().to_string(),
        ascii_hex(&header.storage_version()),
        ascii_hex(&header.creation_date()),
        ascii_hex(&header.schema_name()),
        ascii_hex(&header.schema_version()),
        ext_hex(&header.application_name()),
        ascii_hex(&header.application_version()),
        ext_hex(&header.data_type()),
        ascii_list(&header.user_info()),
        ext_list(&header.comments()),
    ]
    .join(",")
}

pub fn roots_text(roots: &Handle<RootData>) -> String {
    let roots = roots.get().expect("roots");
    let list = roots.roots();
    let entries: Vec<String> = list.get().expect("root list").sequence(|items| {
        items
            .iter()
            .filter_map(Handle::get)
            .map(|root| {
                format!(
                    "{}:{}:{}:{}",
                    ascii_hex(&root.name()),
                    root.reference(),
                    ascii_hex(&root.type_name()),
                    u8::from(root.object().is_some())
                )
            })
            .collect()
    });
    format!(
        "{},{},{},[{}]",
        roots.error_status().ordinal(),
        ascii_hex(&roots.error_status_extension()),
        roots.number_of_roots(),
        entries.join(";")
    )
}

pub fn types_text(types: &Handle<TypeData>) -> String {
    let types = types.get().expect("types");
    let names = types.types();
    let entries: Vec<String> = names.get().expect("type list").sequence(|items| {
        items
            .iter()
            .map(|name| format!("{}:{}", ascii_hex(name), types.type_number(name).unwrap_or(0)))
            .collect()
    });
    format!(
        "{},{},{},[{}]",
        types.error_status().ordinal(),
        ascii_hex(&types.error_status_extension()),
        types.number_of_types(),
        entries.join(";")
    )
}
