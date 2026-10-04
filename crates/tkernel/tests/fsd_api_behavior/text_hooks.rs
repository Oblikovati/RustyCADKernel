use super::fakes::with_file;
use rustycad_tkernel::fsd::{CmpFile, File, MemoryFileSystem};
use rustycad_tkernel::storage::{BaseDriver, OpenMode, StorageError};
use rustycad_tkernel::tcollection::ExtendedString;
use std::rc::Rc;

fn opened_file(bytes: &[u8]) -> File {
    let (_, filesystem) = with_file("hooks", bytes);
    let mut file = File::new(filesystem);
    file.open(&"hooks".into(), OpenMode::Read).unwrap();
    file
}

fn opened_cmp_file(bytes: &[u8]) -> CmpFile {
    let (_, filesystem) = with_file("hooks", bytes);
    let mut file = CmpFile::new(filesystem);
    file.open(&"hooks".into(), OpenMode::Read).unwrap();
    file
}

#[test]
fn base_read_char_word_line_and_string_primitives_are_callable() {
    let mut file = opened_file(b"abc\n  alpha beta\r\n  value\r\n");
    assert_eq!(file.read_char(3).as_bytes(), b"abc");
    assert_eq!(file.read_char(1).as_bytes(), b"\n");
    assert_eq!(file.read_word().as_bytes(), b"alpha");
    assert_eq!(file.read_line().as_bytes(), b"beta\r");
    assert_eq!(file.read_string().as_bytes(), b"value\r");
}

#[test]
fn cmp_overrides_trim_carriage_returns_for_line_and_string() {
    let mut file = opened_cmp_file(b"one\r\n  two\r\n");
    assert_eq!(file.read_line().as_bytes(), b"one");
    assert_eq!(file.read_string().as_bytes(), b"two");
}

#[test]
fn flush_end_of_line_uses_cmp_line_override() {
    let mut file = opened_cmp_file(b"discard\r\nkeep\r\n");
    file.flush_end_of_line();
    assert_eq!(file.read_line().as_bytes(), b"keep");
}

#[test]
fn find_tag_uses_virtual_string_reader_and_native_end_state() {
    let mut file = opened_cmp_file(b" skip\r\n BEGIN_INFO_SECTION\r\nbody\r\n");
    assert_eq!(file.find_tag("BEGIN_INFO_SECTION"), StorageError::Ok);
    assert_eq!(file.read_line().as_bytes(), b"body");
    let mut eof_tag = opened_file(b"END");
    assert_eq!(eof_tag.find_tag("END"), StorageError::SectionNotFound);
}

#[test]
fn find_tag_observes_a_c_string_tag_prefix() {
    let mut file = opened_file(b"BEGIN\nrest\n");
    assert_eq!(file.find_tag("BEGIN\0ignored"), StorageError::Ok);
}

#[test]
fn base_extended_line_round_trips_the_native_byte_pairs() {
    let files = MemoryFileSystem::default();
    let mut writer = File::new(Rc::new(files.clone()));
    writer.open(&"extended".into(), OpenMode::Write).unwrap();
    writer
        .write_extended_line(&ExtendedString::new("AB"))
        .unwrap();
    writer.close().unwrap();
    let bytes = files.contents(&"extended".into()).unwrap();
    assert_eq!(bytes, b"\0A\0B\0\n");
    let mut reader = opened_file(&bytes);
    let mut text = ExtendedString::new("old");
    reader.read_extended_line(&mut text).unwrap();
    assert_eq!(text.to_utf8(), "6566");
}

#[test]
fn cmp_extended_line_override_flushes_the_counted_line() {
    let mut file = opened_cmp_file(b"2 65 66 \ntail\r\n");
    let mut text = ExtendedString::default();
    file.read_extended_line(&mut text).unwrap();
    assert!(text.is_empty());
    assert_eq!(file.read_line().as_bytes(), b"tail");
}

#[test]
fn cmp_extended_line_override_writes_a_count_and_ascii_codes() {
    let files = MemoryFileSystem::default();
    let mut writer = CmpFile::new(Rc::new(files.clone()));
    writer
        .open(&"cmp-extended".into(), OpenMode::Write)
        .unwrap();
    writer
        .write_extended_line(&ExtendedString::new("AB"))
        .unwrap();
    writer.close().unwrap();
    assert_eq!(
        files.contents(&"cmp-extended".into()).unwrap(),
        b"2 65 66 \n"
    );
}
