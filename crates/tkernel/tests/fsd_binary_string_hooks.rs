//! Supplemental tests of the protected BinaryFile string API against pinned OCCT.
#[path = "fsd_api_behavior/fakes.rs"]
mod fakes;
use fakes::{RefusingFileSystem, with_file};
use rustycad_tkernel::fsd::{BinaryFile, MemoryFileSystem};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::{BaseDriver, IoStream, OpenMode};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use std::{fmt::Write, rc::Rc};

fn payload_driver(size: i32, bytes: &[u8]) -> BinaryFile {
    let mut payload = size.to_ne_bytes().to_vec();
    payload.extend_from_slice(bytes);
    let (_, files) = with_file("strings", &payload);
    let mut driver = BinaryFile::new(files);
    driver.open(&"strings".into(), OpenMode::Read).unwrap();
    driver
}

fn status_label(result: Result<(), Failure>) -> &'static str {
    result.err().map_or("ok", |error| error.exception_type())
}

fn append_ascii(output: &mut String, size: i32, bytes: &[u8]) {
    let mut driver = payload_driver(size, bytes);
    let mut text = AsciiString::new("keep");
    let status = status_label(driver.read_string(&mut text));
    writeln!(
        output,
        "{status} {} {} {} {}",
        text.length(),
        String::from_utf8_lossy(text.as_bytes()),
        driver.tell().unwrap(),
        i32::from(driver.is_end().unwrap())
    )
    .unwrap();
}

fn append_extended(output: &mut String, size: i32, bytes: &[u8]) {
    let mut driver = payload_driver(size, bytes);
    let mut text = ExtendedString::new("keep");
    let status = status_label(driver.read_extended_string(&mut text));
    write!(output, "{status} {}", text.length()).unwrap();
    for unit in text.as_utf16() {
        write!(output, " {unit}").unwrap();
    }
    writeln!(
        output,
        " {} {}",
        driver.tell().unwrap(),
        i32::from(driver.is_end().unwrap())
    )
    .unwrap();
}

#[test]
fn protected_binary_string_hooks_match_native_fixture() {
    let mut output = String::new();
    for (size, bytes) in [(3, b"a\0b".as_slice()), (0, b""), (-1, b""), (5, b"ab")] {
        append_ascii(&mut output, size, bytes);
    }
    let units = [0x20ac_u16, 0, 0x41];
    let bytes: Vec<u8> = units.iter().flat_map(|unit| unit.to_ne_bytes()).collect();
    append_extended(&mut output, 3, &bytes);
    for (size, bytes) in [(0, b"".as_slice()), (-1, b""), (3, b"a")] {
        append_extended(&mut output, size, bytes);
    }
    append_round_trip(&mut output);
    append_char_eof(&mut output);
    append_stream_counts(&mut output);
    assert_eq!(
        output,
        include_str!("../../../tools/oracle/tkernel_binary_string_hooks.expected")
    );
}

fn append_round_trip(output: &mut String) {
    let files = MemoryFileSystem::default();
    let mut driver = BinaryFile::new(Rc::new(files));
    let name = AsciiString::new("strings");
    driver.open(&name, OpenMode::Write).unwrap();
    driver.write_string(&"abc".into()).unwrap();
    driver
        .write_extended_string(&ExtendedString::from_utf16(&[0x20ac, 0xd83d, 0xde00]))
        .unwrap();
    writeln!(output, "written {}", driver.tell().unwrap()).unwrap();
    driver.close().unwrap();
    driver.open(&name, OpenMode::Read).unwrap();
    let mut text = AsciiString::default();
    let mut extended = ExtendedString::default();
    driver.read_string(&mut text).unwrap();
    driver.read_extended_string(&mut extended).unwrap();
    writeln!(
        output,
        "{} {} {}",
        String::from_utf8_lossy(text.as_bytes()),
        extended.length(),
        driver.tell().unwrap()
    )
    .unwrap();
}

fn append_char_eof(output: &mut String) {
    let (_, files) = with_file("chars", b"abc");
    let mut driver = BinaryFile::new(files);
    driver.open(&"chars".into(), OpenMode::Read).unwrap();
    let text = driver.read_char(5).unwrap();
    writeln!(
        output,
        "{} {} {}",
        String::from_utf8_lossy(text.as_bytes()),
        driver.tell().unwrap(),
        i32::from(driver.is_end().unwrap())
    )
    .unwrap();
    writeln!(output, "{}", driver.read_char(1).unwrap().length()).unwrap();
}

#[test]
fn protected_string_writers_report_refused_length_prefix() {
    let mut driver = BinaryFile::new(Rc::new(RefusingFileSystem));
    driver.open(&"refused".into(), OpenMode::Write).unwrap();
    assert_eq!(
        driver
            .write_string(&"abc".into())
            .unwrap_err()
            .exception_type(),
        "Storage_StreamWriteError"
    );
    assert_eq!(
        driver
            .write_extended_string(&ExtendedString::new("abc"))
            .unwrap_err()
            .exception_type(),
        "Storage_StreamWriteError"
    );
}

#[test]
fn protected_read_char_zero_request_consumes_no_bytes() {
    let (_, files) = with_file("chars", b"abc");
    let mut driver = BinaryFile::new(files);
    driver.open(&"chars".into(), OpenMode::Read).unwrap();
    assert!(driver.read_char(0).unwrap().is_empty());
    assert_eq!(driver.tell().unwrap(), 0);
    assert_eq!(driver.read_char(3).unwrap().as_bytes(), b"abc");
    assert!(!driver.is_end().unwrap());
}

fn append_stream_counts(output: &mut String) {
    let mut stream = IoStream::from_bytes(Vec::new());
    let ascii = AsciiString::new("abc");
    let extended = ExtendedString::from_utf16(&[0x20ac, 0xd83d, 0xde00]);
    let ascii_count = BinaryFile::write_string_stream(&mut stream, &ascii, true).unwrap();
    let extended_count =
        BinaryFile::write_extended_string_stream(&mut stream, &extended, true).unwrap();
    let counted_position = stream.tellp();
    let ascii_written = BinaryFile::write_string_stream(&mut stream, &ascii, false).unwrap();
    let extended_written =
        BinaryFile::write_extended_string_stream(&mut stream, &extended, false).unwrap();
    writeln!(
        output,
        "{ascii_count} {extended_count} {counted_position} {ascii_written} {extended_written} {}",
        stream.tellp()
    )
    .unwrap();
}
